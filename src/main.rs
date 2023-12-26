use arrakis_node::consts::{DEFAULT_RELAY_URL, NODE_PEM};
use arrakis_node::local_key_handler::LocalKeyManagerOpenssl;
use arrakis_node::utils::{unix_stamp_utc_now, CodeLanguage, HandlerErrors};
use nostro2::relays::{NostrRelay, RelayEvents};
use serde_json::{json, Value};
use std::env;
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let relay_task = tokio::spawn(async move {
        let relay = env::var("RELAY").unwrap_or(DEFAULT_RELAY_URL.to_string());
        if let Err(e) = relay_connection(&relay).await {
            error!("Relay connection failed: {}", e);
        }
    });

    let _ = tokio::join!(relay_task);
}

async fn relay_connection(relay_url: &str) -> Result<(), HandlerErrors> {
    let relay = Arc::new(
        NostrRelay::new(relay_url)
            .await
            .map_err(|_| HandlerErrors::RelayError("Could not create relay".to_string()))?,
    );

    let code_note_filter: Value = json!({
        "kinds":[300],
        "since": unix_stamp_utc_now(),
        "#l": ["rust", "python"]
    });

    relay
        .subscribe(code_note_filter)
        .await
        .map_err(|_| HandlerErrors::RelayError("Could not subscribe to relay".to_string()))?;

    while let Some(Ok(message)) = relay.read_from_relay().await {
        match message {
            RelayEvents::EVENT(_event, _id, signed_note) => {
                info!("Node event:");
                let relay_clone = Arc::clone(&relay);
                tokio::spawn(async move {
                    let node_key_pair = LocalKeyManagerOpenssl::new_from_pem(NODE_PEM.to_string())
                        .map_err(|_| {
                            HandlerErrors::KeyPairError(
                                "Could not create node key pair".to_string(),
                            )
                        });
                    let output_note =
                        CodeLanguage::identify_and_execute(&signed_note, node_key_pair.unwrap())
                            .await;
                    let _ = relay_clone.send_note(output_note).await.map_err(|_| {
                        HandlerErrors::RelayError("Could not send note to relay".to_string())
                    });
                    info!("Node sent output note");
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

#[cfg(test)]
mod tests {
    use super::*;
    use arrakis_node::utils::{create_python_code_note, create_rust_code_note};

    async fn send_rust_code_notes() -> Result<(), HandlerErrors> {
        let relay = NostrRelay::new(DEFAULT_RELAY_URL)
            .await
            .map_err(|_| HandlerErrors::RelayError("Could not create relay".to_string()))?;

        let output_filter = json!({
            "kinds":[301],
            "since": unix_stamp_utc_now()- 1,
        });

        relay
            .subscribe(output_filter)
            .await
            .map_err(|_| HandlerErrors::RelayError("Could not subscribe to relay".to_string()))?;

        let mut message_count = 0;

        while let Some(Ok(message)) = relay.read_from_relay().await {
            match message {
                RelayEvents::EVENT(_event, _id, _signed_note) => {
                    message_count += 1;
                    if message_count == 10 {
                        return Ok(());
                    } else {
                        let signed_code_note = create_rust_code_note(message_count);
                        relay.send_note(signed_code_note).await.map_err(|_| {
                            HandlerErrors::RelayError("Could not send note to relay".to_string())
                        })?;
                    }
                }
                RelayEvents::OK(_, _, _, _) => {}
                RelayEvents::NOTICE(_, _) => {}
                RelayEvents::EOSE(_, _) => {
                    let signed_code_note = create_rust_code_note(0);
                    relay.send_note(signed_code_note).await.map_err(|_| {
                        HandlerErrors::RelayError("Could not send note to relay".to_string())
                    })?;
                }
            }
        }
        Ok(())
    }

    async fn send_python_notes() -> Result<(), HandlerErrors> {
        let relay = NostrRelay::new(DEFAULT_RELAY_URL)
            .await
            .map_err(|_| HandlerErrors::RelayError("Could not create relay".to_string()))?;

        let output_filter = json!({
            "kinds":[301],
            "since": unix_stamp_utc_now()- 1,
        });

        relay
            .subscribe(output_filter)
            .await
            .map_err(|_| HandlerErrors::RelayError("Could not subscribe to relay".to_string()))?;

        let mut message_count = 0;

        while let Some(Ok(message)) = relay.read_from_relay().await {
            match message {
                RelayEvents::EVENT(_event, _id, _signed_note) => {
                    message_count += 1;
                    if message_count == 10 {
                        return Ok(());
                    } else {
                        let signed_code_note = create_python_code_note(message_count);
                        relay.send_note(signed_code_note).await.map_err(|_| {
                            HandlerErrors::RelayError("Could not send note to relay".to_string())
                        })?;
                    }
                }
                RelayEvents::OK(_, _, _, _) => {}
                RelayEvents::NOTICE(_, _) => {}
                RelayEvents::EOSE(_, _) => {
                    let signed_code_note = create_python_code_note(0);
                    relay.send_note(signed_code_note).await.map_err(|_| {
                        HandlerErrors::RelayError("Could not send note to relay".to_string())
                    })?;
                }
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_sending_notes() {
        let relay_task = tokio::spawn(async move {
            let relay = env::var("RELAY").unwrap_or(DEFAULT_RELAY_URL.to_string());
            if let Err(e) = relay_connection(&relay).await {
                panic!("Relay connection failed: {}", e);
            }
        });

        let mut sender_tasks = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
            let rust_task = tokio::spawn(send_rust_code_notes());
            let python_task = tokio::spawn(send_python_notes());
            let _ = tokio::join!(rust_task, python_task);
        });

        let sender_result = tokio::select! {
            _ = relay_task => None,
            result = &mut sender_tasks => Some(result),
        };

        match sender_result {
            Some(Ok(_)) => assert!(true, "Sender tasks completed successfully"),
            Some(Err(e)) => assert!(false, "Sender tasks failed: {:?}", e),
            None => assert!(false, "Relay task finished unexpectedly"),
        }
    }
}
