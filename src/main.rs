use arrakis_node::consts::{DEFAULT_RELAY_URL, NODE_PEM, NOTEBOOK_PEM, USER_PEM};
use arrakis_node::local_key_handler::LocalKeyManagerOpenssl;
use arrakis_node::utils::{CodeLanguage, CodeNote, unix_stamp_utc_now};
use nostro2::notes::Note;
use nostro2::relays::{NostrRelay, RelayEvents};
use serde_json::{json, to_string_pretty, Value};
use std::env;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

fn create_code_content() -> String {
    // send a rust string that retrieves returns all pid processes into a info
    let source: String = String::from(
        r#"
        use std::process::Command;
        let output = Command::new("ps")
            .arg("aux")
            .output()
            .expect("failed to execute process");
        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        "#,
    );
    source
}

async fn create_code_cells() {
    sleep(Duration::from_secs(2)).await;
    let notebook_key_pair = LocalKeyManagerOpenssl::new_from_pem(NOTEBOOK_PEM.to_string());
    let user_key_pair = LocalKeyManagerOpenssl::new_from_pem(USER_PEM.to_string());

    if let Ok(notebook_key_pair) = notebook_key_pair {
        if let Ok(user_key_pair) = user_key_pair {
            let mut rust_note = Note::new(user_key_pair.get_public_key(), 300, &create_code_content());
            rust_note.tag_note("l", "rust");
            rust_note.tag_note("N", &notebook_key_pair.get_public_key());
            let rust_signed_note = user_key_pair.sign_nostr_event(rust_note);
            let mut python_note = Note::new(user_key_pair.get_public_key(), 300, "hello world");
            python_note.tag_note("l", "python");
            python_note.tag_note("N", &notebook_key_pair.get_public_key());
            let python_signed_note = user_key_pair.sign_nostr_event(python_note);
            let url = env::var("RELAY").unwrap_or(DEFAULT_RELAY_URL.to_string());
            if let Ok(relay) = NostrRelay::new(&url).await {
                relay
                    .send_note(rust_signed_note)
                    .await
                    .expect("Failed to send note");
                relay
                    .send_note(python_signed_note)
                    .await
                    .expect("Failed to send note");
                info!("Sent code notes");
            }
        }
    }
}

enum HandlerErrors {
    NoRelay(String),
    NoNodeKeyPair(String),
}

impl std::fmt::Display for HandlerErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HandlerErrors::NoRelay(msg) => write!(f,"{}", msg),
            HandlerErrors::NoNodeKeyPair(msg) => write!(f, "{}", msg),
        }
    }
}

async fn code_notes_handler() -> Result<(), HandlerErrors> {
    let url = env::var("RELAY").unwrap_or(DEFAULT_RELAY_URL.to_string());
    if let Ok(node_key_pair) = LocalKeyManagerOpenssl::new_from_pem(NODE_PEM.to_string()) {
        if let Ok(relay) = NostrRelay::new(&url).await {
            let rust_filter: Value = json!({
                "kinds":[300],
                "since": unix_stamp_utc_now(),
                "#l": ["rust", "python"]
            });
            relay
                .subscribe(rust_filter)
                .await
                .expect("Could not subscribe");

            while let Some(Ok(message)) = relay.read_from_relay().await {
                match message {
                    RelayEvents::EVENT(_event, _id, signed_note) => {
                        match CodeLanguage::from_signed_note(&signed_note) {
                            CodeLanguage::Rust(rust_note) => {
                                info!("Rust code note incoming {}", &to_string_pretty(&signed_note).unwrap());
                                let rust_output_note =
                                    rust_note.run().create_output_note(&node_key_pair);
                                info!("Rust code note output created {}", to_string_pretty(&rust_output_note).unwrap());
                                let _ = relay.send_note(rust_output_note).await;
                                info!("Rust code note handled");
                            }
                            CodeLanguage::Python => {
                                info!("Python code note handled");
                            }
                            CodeLanguage::None => {
                                error!("Received unsupported code note");
                            }
                        }
                    }
                    RelayEvents::OK(_event, id, success, _msg) => {
                        info!("Message: {} was sent correctly: {} {}", id, success, _msg);
                    }
                    RelayEvents::NOTICE(e, msg) => {
                        info!("Received notice: {} {}", e, msg);
                    }
                    RelayEvents::EOSE(_, msg) => {
                        info!("End of subscription {}, waiting for more.", msg);
                    }
                }
            }
        } else {
            return Err(HandlerErrors::NoRelay("Could not create relay".to_string()));
        }
    } else {
        return Err(HandlerErrors::NoNodeKeyPair(
            "Could not create node key pair".to_string(),
        ));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), String> {
    tracing_subscriber::fmt::init();

    let code_handler = tokio::spawn(async move {
        if let Err(e) = code_notes_handler().await {
            error!("Code notes handler error: {}", e);
        }
    });

    let code_cells = tokio::spawn(async move {
        create_code_cells().await;
    });

    let _ = code_handler.await;
    let _ = code_cells.await;
    Ok(())
}
