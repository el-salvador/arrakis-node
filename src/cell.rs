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
                println!(
                    "Code Cell {}",
                    to_string_pretty(&s).unwrap()
                );
                let codecell = Code::new(s);
                tokio::spawn(async move {
                    let signed_note: SignedNote = codecell.run();
                    let url = var("URL").unwrap_or("ws://localhost:7001".to_string());
                    if let Ok(relay) = NostrRelay::new(&url).await {
                        relay.send_note(signed_note).await.expect("Failed to send note");
                    }
                });
            }
            CellFactory::Output(s) => {
                println!(
                    "Output Cell: {}",
                    to_string_pretty(&s).unwrap()
                );
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeContent {
    source: Vec<String>,
}

impl CodeContent {
    pub fn new(source: Vec<String>) -> Self {
        CodeContent { source }
    }
    // Expect the source to be a a vector of strings.
    pub fn get_source(self) -> Vec<String> {
        self.source
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
    pub fn create_output_note(self, stdout: String) -> SignedNote {
        let file_path = retrieve_pem_file_path(
            "private-diego-stash.pem".to_string()
        ).unwrap();
        let user_key_pair = LocalKeyManagerOpenssl::new(file_path);
        let mut note = Note::new(
            user_key_pair.get_public_key(),
            301,
            &stdout
        );
        note.tag_note("u", self.signed_note.get_pubkey());
        note.tag_note("a", self.signed_note.get_id());
        let signed_note = user_key_pair.sign_nostr_event(note);
        signed_note
    }
    pub fn run(self) -> SignedNote {
        // we will take the &self.signed_note, and create an output cell. The output_cell
        // should have the stdout with kind 301 ready to be sent to the relay.
        // The structure of the new cell can be made by 
        let code_content: CodeContent = serde_json::from_str(
            self.signed_note.get_content()
        ).unwrap();
        let stdout = create_rust_file_compile_and_run(
            code_content.get_source()
        );
        let signed_note = self.create_output_note(stdout.unwrap());
        signed_note
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
