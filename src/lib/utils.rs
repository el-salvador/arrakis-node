use chrono::prelude::Utc;
use nostro2::notes::{SignedNote, Note};
use serde_json::Value;

use crate::{local_key_handler::LocalKeyManagerOpenssl, rust_handler::RustCodeNote, python_handler::PythonCodeNote};
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


pub enum CodeLanguage {
    Rust(RustCodeNote),
    Python(PythonCodeNote),
    None
}

impl CodeLanguage {
    fn identify_language(s: &SignedNote) -> Self {
        if let Some(tag) = s.get_tags_by_id("l").first() {
            match tag.as_str() {
                "rust" => CodeLanguage::Rust(RustCodeNote::from_signed_note(s).unwrap()),
                "python" => CodeLanguage::Python(PythonCodeNote::from_signed_note(s).unwrap()),
                _ => CodeLanguage::None,
            }
        } else {
            CodeLanguage::None
        }
    }

    pub async fn identify_and_execute(
        input_note: &SignedNote,
        key_manager: LocalKeyManagerOpenssl,
    ) -> SignedNote {
        match CodeLanguage::identify_language(input_note) {
            CodeLanguage::Rust(rust_note) => rust_note.run().await.create_output_note(key_manager),
            CodeLanguage::Python(python_note) => python_note.run().await.create_output_note(key_manager),
            CodeLanguage::None => key_manager.sign_nostr_event(Note::new(key_manager.get_public_key(), 301, "No language support")),
        }
    }

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
