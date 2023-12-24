use arrakis_node::consts::{DEFAULT_RELAY_URL, NODE_PEM, NOTEBOOK_PEM, USER_PEM};
use arrakis_node::local_key_handler::LocalKeyManagerOpenssl;
use arrakis_node::utils::{unix_stamp_utc_now, CodeLanguage, CodeNote};
use nostro2::notes::Note;
use nostro2::relays::{NostrRelay, RelayEvents};
use serde_json::{json, to_string_pretty, Value};
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use tracing::{error, info};

fn create_code_content() -> String {
    // send a rust string that retrieves returns all pid processes into a info
    let source: String = String::from(
        r#"
        use std::process::Command;
        let output = Command::new("whoami")
            .output()
            .expect("failed to execute process");
        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        "#,
    );
    source
}

fn create_python_script_content() -> String {
    // Create a Rust string that contains a basic Python script
    let source: String = String::from(
        r#"

def greet(name):
        if name:
              return "Hello, " + name + "!"
        else:
              return "Hello, World!"

names = ["Alice", "Bob", "", "Charlie"]

for name in names:
    greeting = greet(name)
    print(greeting)
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
            let mut rust_note =
                Note::new(user_key_pair.get_public_key(), 300, &create_code_content());
            rust_note.tag_note("l", "rust");
            rust_note.tag_note("N", &notebook_key_pair.get_public_key());
            let rust_signed_note = user_key_pair.sign_nostr_event(rust_note);
            let mut python_note = Note::new(
                user_key_pair.get_public_key(),
                300,
                &create_python_script_content(),
            );
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

#[derive(Debug)]
enum HandlerErrors {
    NoRelay(String),
    NoNodeKeyPair(String),
}

impl std::fmt::Display for HandlerErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HandlerErrors::NoRelay(msg) => write!(f, "{}", msg),
            HandlerErrors::NoNodeKeyPair(msg) => write!(f, "{}", msg),
        }
    }
}

async fn code_notes_handler() -> Result<(), HandlerErrors> {
    let url = env::var("RELAY").unwrap_or(DEFAULT_RELAY_URL.to_string());

    let relay = NostrRelay::new(&url)
        .await
        .map_err(|_| HandlerErrors::NoRelay("Could not create relay".to_string()))?;
    let relay = Arc::new(Mutex::new(relay)); // Wrap in Arc and Mutex

    let rust_filter: Value = json!({
        "kinds":[300],
        "since": unix_stamp_utc_now(),
        "#l": ["rust", "python"]
    });

    relay
        .lock()
        .await
        .subscribe(rust_filter)
        .await
        .expect("Could not subscribe");

    while let Some(Ok(message)) = relay.lock().await.read_from_relay().await {
        match message {
            RelayEvents::EVENT(_event, _id, signed_note) => {
                info!(
                    "Received event: {}",
                    to_string_pretty(&signed_note).unwrap()
                );
                let relay_clone = relay.clone();
                tokio::spawn(async move {
                    let node_key_pair = LocalKeyManagerOpenssl::new_from_pem(NODE_PEM.to_string())
                        .map_err(|_| {
                            HandlerErrors::NoNodeKeyPair(
                                "Could not create node key pair".to_string(),
                            )
                        });
                    let output_note =
                        CodeLanguage::identify_and_execute(&signed_note, node_key_pair.unwrap()).await;
                    info!("Output note: {}", to_string_pretty(&output_note).unwrap());
                    let _ = relay_clone.lock().await.send_note(output_note).await;
                });
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
