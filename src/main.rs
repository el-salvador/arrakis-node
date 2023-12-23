use arrakis_node::consts::{DEFAULT_RELAY_URL, NOTEBOOK_PEM, NODE_PEM, USER_PEM};
use arrakis_node::utils::{CodeLanguage, CodeNote};
use arrakis_node::local_key_handler::LocalKeyManagerOpenssl;
use arrakis_node::utils::unix_stamp_from_the_beginning_of_today;
use nostro2::notes::Note;
use nostro2::relays::{NostrRelay, RelayEvents};
use serde_json::{json, to_string_pretty, Value};
use std::env;
use tokio::time::{sleep, Duration};

fn create_code_content() -> String {
    // send a rust string that retrieves returns all pid processes into a println
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
            let mut note = Note::new(user_key_pair.get_public_key(), 300, &create_code_content());
            note.tag_note("l", "rust");
            note.tag_note("N", &notebook_key_pair.get_public_key());
            let signed_note = user_key_pair.sign_nostr_event(note);
            let url = env::var("RELAY").unwrap_or(DEFAULT_RELAY_URL.to_string());
            if let Ok(relay) = NostrRelay::new(&url).await {
                relay
                    .send_note(signed_note)
                    .await
                    .expect("Failed to send note");
            }
        }
    }
}

async fn code_notes_handler() {
    let url = env::var("RELAY").unwrap_or(DEFAULT_RELAY_URL.to_string());
    if let Ok(node_key_pair) = LocalKeyManagerOpenssl::new_from_pem(NODE_PEM.to_string()) {
        if let Ok(relay) = NostrRelay::new(&url).await {
            let rust_filter: Value = json!({
                "kinds":[300,301],
                "since": unix_stamp_from_the_beginning_of_today(),
                "limit": 100,
                "#l": ["rust"]
            });
            relay
                .subscribe(rust_filter)
                .await
                .expect("Could not subscribe");

            let python_filter: Value = json!({
                "kinds":[300,301],
                "since": unix_stamp_from_the_beginning_of_today(),
                "limit": 100,
                "#l":["python"]
            });
            relay
                .subscribe(python_filter)
                .await
                .expect("Could not subscribe");

            while let Some(Ok(message)) = relay.read_from_relay().await {
                match message {
                    RelayEvents::EVENT(_event, _id, signed_note) => {
                        match CodeLanguage::from_signed_note(&signed_note) {
                            CodeLanguage::Rust(rust_note) => {
                                let rust_output_note =
                                    rust_note.run().create_output_note(&node_key_pair);
                                let _ = relay.send_note(rust_output_note).await;
                            }
                            CodeLanguage::Python => {}
                            CodeLanguage::None => {
                                println!("Received a note that is not a code note");
                            }
                        }
                    }
                    RelayEvents::OK(_event, id, success, _msg) => {
                        println!("Received Ok: {} {} {}", id, success, _msg);
                    }
                    _ => {
                        println!("Received something else");
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let _ = tokio::join!(code_notes_handler(), create_code_cells());
    Ok(())
}
