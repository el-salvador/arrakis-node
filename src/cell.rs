use crate::local_key_manager_openssl::LocalKeyManagerOpenssl;
use crate::utils::retrieve_pem_file_path;
use nostro2::notes::{Note, SignedNote};
use nostro2::relays::{NostrRelay, RelayEvents};
use nostro2::userkeys::UserKeys;
use serde::{Deserialize, Serialize};
use serde_json::{json, to_string_pretty, Value};
use std::collections::HashMap;
use std::env::var;
use crate::run_rust::{create_rust_file_compile_and_run};

// I would like to create a cell struct. The Cell struct will store content like Jupyter Notebook
// cells. The Implemenetation will have the ability to create a signed note using nostro2. There
// will also be delegation logic to determine what format the content of the note should look like
// along with auto tagging in the Note struct using the tag method.

// Pass the signed note inside the enum.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "cell_type")]
pub enum Cell {
    Code(CodeCell),
    Output(OutputCell),
}

//Factory that build cells from signed_notes.
#[derive(Debug, Serialize, Deserialize)]
pub enum CellFactory {
    Code(SignedNote),
    Output(SignedNote),
}

impl CellFactory {
    pub fn identify_note_type(s: SignedNote) -> Self {
        match s.get_kind() {
            300 => CellFactory::Code(s),
            301 => CellFactory::Output(s),
            _ => panic!("Invalid Note Type"),
        }
    }
    pub fn execute(self) {
        match self {
            CellFactory::Code(s) => {
                let codecell = Code::new(s);
                tokio::spawn(async move {
                     codecell.run().await;
                });
            }
            CellFactory::Output(s) => {
                let output_cell: OutputCell = serde_json::from_str(
                    s.get_content()
                ).unwrap();
                println!(
                    "Output Cell: {:?}",
                    to_string_pretty(&output_cell).unwrap()
                );
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Code {
    signed_note: SignedNote,
}

impl Code {
    pub fn new(s: SignedNote) -> Self {
        Code { signed_note: s }
    }
    pub async fn run(self) {
        // we will take the &self.signed_note, and create an output cell. The output_cell
        // should have the stdout with kind 301 ready to be sent to the relay.
        // The structure of the new cell can be made by 
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Output{
    signed_note: SignedNote,
}

impl Output {
    pub fn new(s: SignedNote) -> Self {
        Output { signed_note: s }
    }
    pub fn hello_world(&self) {
        println!("Hello World!");
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeCell {
    source: Vec<String>,
    language: String,
}

impl CodeCell {
    pub fn new(source: Vec<String>, language: String) -> Self {
        CodeCell { source, language }
    }
    pub fn create_cell_content(self) -> String {
        // I want to be able to take the code and language and create a json string that will be used in the code not.
        let json_string = json!({
          "language": self.language,
          "source": self.source,
          "cell_type": "Code"
        });
        json_string.to_string()
    }
    pub fn create_code_note(self) -> SignedNote {
        let file_path = retrieve_pem_file_path(
            "private-diego-stash.pem".to_string()
        ).unwrap();
        let user_key_pair = LocalKeyManagerOpenssl::new(file_path);
        let mut note = Note::new(
            user_key_pair.get_public_key(),
            300,
            &self.create_cell_content(),
        );
        note.tag_note("l", "rust");
        note.tag_note("i", "cellID");
        let signed_note = user_key_pair.sign_nostr_event(note);
        signed_note
    }
    pub fn get_source(self) -> Vec<String> {
        self.source
    }
    // I would like to have a run function that will use the
    // methods inside this implementation to run the the create_code_note
    // function and send out the note to the relay.
    pub async fn run(self) {
        let signed_code_note = self.create_code_note();
        println!(
            "Signed Code Note: {}",
            to_string_pretty(&signed_code_note).unwrap()
        );
        let url = var("URL").unwrap_or("ws://localhost:7001".to_string());
        if let Ok(relay) = NostrRelay::new(&url).await {
            relay.send_note(signed_code_note).await.expect("Failed to send note");
        }
    }
    pub async fn create_output_request(self) {
        println!("Create Output Request {}", to_string_pretty(&self).unwrap());
        let signed_code_note = self.create_code_note();
        let current_cell: CodeCell = serde_json::from_str(
            &signed_code_note.get_content()
        ).unwrap();
        let stdout = create_rust_file_compile_and_run(
            current_cell.get_source()
        ).unwrap();
        let output_cell = OutputCell::new(stdout);
        output_cell.run().await;

    }
}

// The OutputCell will have the structure of the stdout
// this stdout will be the result of the code cell
// The stdout will be a string
// the new note created will have the tags activeCell and publickey
// of the user that created the code cell
#[derive(Serialize, Deserialize, Debug)]
pub struct OutputCell {
    stdout: String,
}

impl OutputCell {
    pub fn new(stdout: String) -> Self {
        OutputCell { stdout }
    }
    pub fn create_output_content(self) -> String {
        let json_string = json!({
          "stdout": self.stdout,
          "cell_type": "Output"
        });
        json_string.to_string()
    }
    pub fn create_output_note(self) -> SignedNote {
        let file_path = retrieve_pem_file_path(
            "private-diego-stash.pem".to_string()
        ).unwrap();
        let user_key_pair = LocalKeyManagerOpenssl::new(file_path);
        let mut note = Note::new(
            user_key_pair.get_public_key(),
            301,
            &self.create_output_content(),
        );
        note.tag_note("l", "rust");
        note.tag_note("i", "0:cellId");
        note.tag_note("a", "activeCellID");
        let signed_note = user_key_pair.sign_nostr_event(note);
        signed_note
    }
    pub async fn run(self) {
        let signed_output_note = self.create_output_note();
        println!(
            "Signed Output Note: {}",
            to_string_pretty(&signed_output_note).unwrap()
        );
        let url = var("URL").unwrap_or("ws://localhost:7001".to_string());
        if let Ok(ws_connection) = NostrRelay::new(&url).await {
            let _ = ws_connection.send_note(signed_output_note).await;
        }
    }
}
