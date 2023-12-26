use chrono::prelude::Utc;
use nostro2::notes::{Note, SignedNote};
use serde_json::Value;

use crate::{
    consts::USER_PEM, local_key_handler::LocalKeyManagerOpenssl, python_handler::PythonCodeNote,
    rust_handler::RustCodeNote,
};
use async_trait::async_trait;

pub fn unix_stamp_utc_now() -> u64 {
    let now = Utc::now();
    now.timestamp() as u64
}

pub fn append_key_value_to_json_string(json_string: String, key: String, value: String) -> String {
    let mut json: Value = serde_json::from_str(&json_string).unwrap();
    json[key] = serde_json::Value::String(value);
    json.to_string()
}

pub fn create_rust_code_note(execution_count: u32) -> SignedNote {
    let source = format!("println!(\"Hello world from Rust times {}!\");", execution_count);
    let student_key = LocalKeyManagerOpenssl::new_from_pem(USER_PEM.to_string()).unwrap();
    let mut note = Note::new(student_key.get_public_key(), 300, &source);
    note.tag_note("l", "rust");
    let signed_note = student_key.sign_nostr_event(note);
    signed_note
}

pub fn create_python_code_note(execution_count: u32) -> SignedNote {
    let source = format!("print(\"Hello world from Python times {}!\")", execution_count);
    let student_key = LocalKeyManagerOpenssl::new_from_pem(USER_PEM.to_string()).unwrap();
    let mut note = Note::new(student_key.get_public_key(), 300, &source); // Assuming 302 is the kind for Python
    note.tag_note("l", "python");
    let signed_note = student_key.sign_nostr_event(note);
    signed_note
}

#[async_trait]
pub trait CodeNote {
    fn from_signed_note(signed_note: &SignedNote) -> Result<Self, String>
    where
        Self: Sized;

    async fn run(self) -> Self;

    fn create_output_note(&self, key_manager: LocalKeyManagerOpenssl) -> SignedNote
    where
        Self: Sized;
}

pub enum CodeLanguage {
    Rust(RustCodeNote),
    Python(PythonCodeNote),
    Unverified,
    Unsupported,
}

impl CodeLanguage {
    fn identify_language(s: &SignedNote) -> Self {
        if s.verify_signature() == false || s.verify_content() == false {
            return CodeLanguage::Unverified;
        };

        if let Some(tag) = s.get_tags_by_id("l").first() {
            match tag.as_str() {
                "rust" => CodeLanguage::Rust(RustCodeNote::from_signed_note(s).unwrap()),
                "python" => CodeLanguage::Python(PythonCodeNote::from_signed_note(s).unwrap()),
                _ => CodeLanguage::Unsupported,
            }
        } else {
            CodeLanguage::Unsupported
        }
    }

    pub async fn identify_and_execute(
        input_note: &SignedNote,
        key_manager: LocalKeyManagerOpenssl,
    ) -> SignedNote {
        match CodeLanguage::identify_language(input_note) {
            CodeLanguage::Rust(rust_note) => rust_note.run().await.create_output_note(key_manager),
            CodeLanguage::Python(python_note) => {
                python_note.run().await.create_output_note(key_manager)
            }
            CodeLanguage::Unsupported => key_manager.sign_nostr_event(Note::new(
                key_manager.get_public_key(),
                301,
                "No language support",
            )),
            CodeLanguage::Unverified => key_manager.sign_nostr_event(Note::new(
                key_manager.get_public_key(),
                301,
                "Invalid note",
            )),
        }
    }
}

#[derive(Debug)]
pub enum HandlerErrors {
    RelayError(String),
    KeyPairError(String),
    TestingError(String),
}

impl std::fmt::Display for HandlerErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            HandlerErrors::RelayError(msg) => write!(f, "{}", msg),
            HandlerErrors::KeyPairError(msg) => write!(f, "{}", msg),
            HandlerErrors::TestingError(msg) => write!(f, "{}", msg),
        }
    }
}
