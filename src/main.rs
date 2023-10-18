use nostro2::notes::{Note, SignedNote};
use nostro2::userkeys::{UserKeys};
use nostro2::relays::{NostrRelay, RelayEvents};
use serde_json::{json, Value, to_string_pretty, from_str};
mod utils;
mod cell;
mod local_key_manager_openssl;
mod run_rust;
use crate::cell::{Cell};
use crate::utils::{
    unix_stamp_from_the_beginning_of_today,
    append_key_value_to_json_string
};
use std::env;
use tokio::time::{sleep, Duration};

async fn create_code_cells(){
    sleep(Duration::from_secs(5)).await;
    let code_cell_example: Cell = serde_json::from_str(
        r#"{
          "cell_type":"Code",
          "source":["println!(\"Hello World!\");"],
          "language":"rust"
      }"#
    ).unwrap();
    //
    match code_cell_example {
        Cell::Code(code_cell) => code_cell.run().await,
        _ => panic!("Not a code cell")
    };
}

async fn listen_to_relay(){
    let url = env::var("URL").unwrap_or("ws://localhost:7001".to_string());
    if let Ok(relay) = NostrRelay::new(&url).await {
    let filter: Value = json!({
        "kinds":[300,301],
        // unix timespamp since the beginning of today
        "since": unix_stamp_from_the_beginning_of_today(),
        "limit": 100,
    });
        relay.subscribe(filter).await.expect("Could not subscribe");
        // let _ = ws_connection.send_note(signed_code_note).await;
        while let Some(Ok(message)) = relay.read_from_relay().await {
                match message {
                    RelayEvents::EVENT(_event,_id, signed_note) => {
                        println!(
                            "Received event: {}",
                            to_string_pretty(&signed_note).unwrap()
                        );
                        ////////////////////////////
                        // 1) Retrieve a rust code note
                        // 2) Send to create_output_request
                        ////////////////////////////
                        let current_cell: Cell = serde_json::from_str(
                            signed_note.get_content()
                        ).unwrap();
                        let rust_file_output = match current_cell {
                            Cell::Code(c) => c.create_output_request().await,
                            Cell::Output(o) => println!(
                                "Output Cell Matched: {}",
                                to_string_pretty(&signed_note).unwrap()
                            ),
                            _ => panic!("Not a code cell"),
                        };
                        ////////////////////////////
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
    let _ = tokio::join!(create_code_cells(), listen_to_relay());
    Ok(())
}
