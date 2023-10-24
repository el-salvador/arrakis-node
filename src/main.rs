use nostro2::notes::{Note, SignedNote};
use nostro2::userkeys::{UserKeys};
use nostro2::relays::{NostrRelay, RelayEvents};
use serde_json::{json, Value, to_string_pretty, from_str};
mod utils;
mod cell;
mod local_key_manager_openssl;
mod run_rust;
use crate::cell::{CellFactory};
use crate::utils::{
    unix_stamp_from_the_beginning_of_today,
    append_key_value_to_json_string
};
use std::env;
use tokio::time::{sleep, Duration};
use crate::local_key_manager_openssl::LocalKeyManagerOpenssl;
use crate::utils::retrieve_pem_file_path;

fn create_code_content() -> String {
    // let json_string = json!({
        // "source": ["println!(\"Hello World!\");"]
    // });
    // json_string.to_string()
    let source: String = String::from("fn main(){\tprintln!(\"Hello World!\");\t}");
    source

}
async fn create_code_cells(){
    sleep(Duration::from_secs(5)).await;
    let notebook_pem_file_path = retrieve_pem_file_path("notebook.pem".to_string()).unwrap();
    let notebook_key_pair = LocalKeyManagerOpenssl::new(notebook_pem_file_path);
    let user_pem_file_path = retrieve_pem_file_path("private-diego-stash.pem".to_string()).unwrap();
    let user_key_pair = LocalKeyManagerOpenssl::new(user_pem_file_path);
    let mut note = Note::new(
        user_key_pair.get_public_key(),
        300,
        &create_code_content()
    );
    note.tag_note("l", "rust");
    note.tag_note("N", &notebook_key_pair.get_public_key());
    let signed_note = user_key_pair.sign_nostr_event(note);
    let cell: CellFactory = CellFactory::identify_note_type(signed_note);
    cell.execute();
}

async fn create_from_signed_note() {
    let url = env::var("URL").unwrap_or("ws://localhost:7001".to_string());
    if let Ok(relay) = NostrRelay::new(&url).await {
    let filter: Value = json!({
        "kinds":[300,301],
        "since": unix_stamp_from_the_beginning_of_today(),
        "limit": 100,
        "l":"rust"
    });
    relay.subscribe(filter).await.expect("Could not subscribe");
    while let Some(Ok(message)) = relay.read_from_relay().await {
            match message {
                RelayEvents::EVENT(_event,_id, signed_note) => {
                    println!(
                        "Received event: {}",
                        to_string_pretty(&signed_note).unwrap()
                    );
                    let cell: CellFactory = CellFactory::identify_note_type(
                        signed_note
                    );
                    cell.execute();

                },
                RelayEvents::OK(_event, id, success, _msg) =>{
                    println!("Received Ok: {} {} {}", id, success, _msg);
                },
                _ => {
                    println!("Received something else");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(),String> {
    let _ = tokio::join!(
        create_code_cells(),
        create_from_signed_note()
    );
    Ok(())
}
