use crate::{local_key_handler::LocalKeyManagerOpenssl, utils::CodeNote};
use nostro2::notes::{Note, SignedNote};
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::Command;

use hex::encode;
use rand::Rng;
use sha256::digest;

use serde::{Deserialize, Serialize};


#[derive(Debug, Deserialize, Serialize)]
pub struct RustCodeNote {
    code: String,
    code_vector: Vec<String>,
    output: Option<Result<String, String>>,
    notebook: String,
    user: String,
}

impl RustCodeNote {
    fn create_vector_by_splitting_from_regex(&mut self) {
        // The regex shoul split the string into a vector
        // everytime it sees "\t" or "\n"
        let regex = r"[\t]";
        for s in self.code.split(regex) {
            self.code_vector.push(s.to_string());
        }
    }

    fn generate_random_hash_string(&self) -> String {
        let mut rng = rand::thread_rng();
        let random_string: String = rng.gen::<u32>().to_string();
        let hash = digest(random_string.as_bytes());
        let hash_string = encode(hash);
        hash_string
    }

    fn create_script_file(&self, code: String) -> Result<String, String> {
        // Create a hash string
        let hash: String = self.generate_random_hash_string();
        // create a main.rs file inside /tmp/<hash>/main.rs
        if let Ok(_) = fs::create_dir_all(format!("/tmp/{}", hash)) {
            if let Ok(mut file) = File::create(format!("/tmp/{}/main.rs", hash)) {
                if file.write_all(code.as_bytes()).is_err() {
                    return Err("Could not write to file.".to_string());
                } else {
                    Ok(hash)
                }
            } else {
                Err("Could not create main file.".to_string())
            }
        } else {
            Err("Could not create directory.".to_string())
        }
    }

    fn remove_comments(&mut self) {
        let mut new_vec: Vec<String> = Vec::new();
        self.code_vector.iter().for_each(|i| {
            if i.contains("//") {
                let mut split = i.split("//");
                let first = split.next().unwrap();
                new_vec.push(first.to_string());
            } else {
                new_vec.push(i.to_string());
            }
        });
        self.code_vector = new_vec;
    }

    fn ensure_main_function(&mut self) -> String {
        // double check that comments are gone
        self.remove_comments();
        let re = Regex::new(r"(?s)fn main\s*\(\s*\)\s*\{.*\}").unwrap();
        let new_string = self.code_vector.join(" ");
        if re.is_match(&new_string) {
            new_string.to_string()
        } else {
            format!(r#"fn main() {{ {} }}"#, new_string)
        }
    }

    fn run_script_file(&self, hash: String) -> Result<String, String> {
        match Command::new("rust-script")
            .arg(format!("/tmp/{}/main.rs", hash))
            .output()
        {
            Ok(output) => {
                if output.stdout.is_empty() {
                    // If stdout is empty, return stderr as the error message
                    Err(String::from_utf8_lossy(&output.stderr).to_string())
                } else {
                    // Otherwise, return stdout as the successful output
                    Ok(String::from_utf8_lossy(&output.stdout).to_string())
                }
            }
            Err(e) => {
                // Handle any errors that occur when running the command
                Err(e.to_string())
            }
        }
    }

    pub fn create_rust_file_compile_and_run(&mut self) -> Result<String, String> {
        self.create_vector_by_splitting_from_regex();
        self.remove_comments();
        let new_code = self.ensure_main_function();
        match self.create_script_file(new_code) {
            Ok(hash) => match self.run_script_file(hash) {
                Ok(output) => Ok(output),
                Err(e) => Err(e),
            },
            Err(e) => Err(e),
        }
    }
}

impl CodeNote for RustCodeNote {
    fn from_signed_note(signed_note: &SignedNote) -> Result<Self, String> {
        if signed_note.verify_signature() == false || signed_note.verify_content() == false {
            return Err("Verification failed".to_string());
        }

        let code = signed_note.get_content().to_string();
        let notebook = signed_note.get_tags_by_id("N").first().unwrap().to_string();
        let user = signed_note.get_pubkey().to_string();
        Ok(RustCodeNote {
            code,
            code_vector: Vec::new(),
            output: None,
            notebook,
            user,
        })
    }

    fn run(mut self) -> Self {
        self.output = Some(self.create_rust_file_compile_and_run());
        self
    }

    fn create_output_note(&self, key_manager: &LocalKeyManagerOpenssl) -> SignedNote {
        let content = match &self.output {
            Some(Ok(s)) => s.to_string(),
            Some(Err(e)) => e.to_string(),
            None => "No output".to_string(),
        };
        let mut note = Note::new(key_manager.get_public_key(), 301, &content);
        note.tag_note("N", &self.notebook);
        note.tag_note("u", &self.user);
        let signed_note = key_manager.sign_nostr_event(note);
        signed_note
    }
}
