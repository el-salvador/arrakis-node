use chrono::prelude::Utc;
use nostro2::notes::SignedNote;
use serde_json::Value;

use crate::{local_key_handler::LocalKeyManagerOpenssl, rust_handler::RustCodeNote};

pub fn unix_stamp_from_the_beginning_of_today() -> u64 {
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
    Python,
    None
}

impl CodeLanguage {
    pub fn from_signed_note(s: &SignedNote) -> Self {
        if let Some(tag) = s.get_tags_by_id("l").first() {
            match tag.as_str() {
                "rust" => CodeLanguage::Rust(RustCodeNote::from_signed_note(s).unwrap()),
                "python" => CodeLanguage::Python,
                _ => CodeLanguage::None,
            }
        } else {
            CodeLanguage::None
        }
    }
}

pub trait CodeNote {
    fn from_signed_note(signed_note: &SignedNote) -> Result<Self, String>
    where
        Self: Sized;

    fn run(self) -> Self
    where
        Self: Sized;

    fn create_output_note(&self, key_manager: &LocalKeyManagerOpenssl) -> SignedNote
    where
        Self: Sized;
}
