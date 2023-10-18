use chrono::prelude::Utc;
use serde_json::Value;
use std::env;
// I want to write a function that will retrieve the
// private key from the microservice's directory.
// I want to make sure that this private key is not retrievalble by anyone
// other than the microservice.
pub fn retrieve_pem_file_path() -> Result<String, Box<dyn std::error::Error>> {
    let pem_file = env::var("PEM_PATH").unwrap_or(format!("{}", "private-diego-stash.pem"));
    let file_path = format!("./{}", pem_file);
    Ok(file_path)
}

pub fn unix_stamp_from_the_beginning_of_today() -> u64 {
    let now = Utc::now();
    now.timestamp() as u64
}

pub fn append_key_value_to_json_string(json_string: String, key: String, value: String) -> String {
    let mut json: Value = serde_json::from_str(&json_string).unwrap();
    json[key] = serde_json::Value::String(value);
    json.to_string()
}
