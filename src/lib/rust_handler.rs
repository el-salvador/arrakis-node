use crate::{local_key_handler::LocalKeyManagerOpenssl, utils::CodeNote};
use async_trait::async_trait;
use nostro2::notes::{Note, SignedNote};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
    process::Command,
};
use tokio::time::{sleep, Duration};

#[derive(Debug, Deserialize, Serialize)]
pub struct RustCodeNote {
    code: String,
    code_vector: Vec<String>,
    output: Option<Result<String, String>>,
    input_note: String,
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

    fn ensure_main_function(&mut self) {
        // double check that comments are gone
        self.remove_comments();
        let re = Regex::new(r"(?s)fn main\s*\(\s*\)\s*\{.*\}").unwrap();
        let new_string = self.code_vector.join(" ");
        if re.is_match(&new_string) {
            self.code = new_string.to_string()
        } else {
            self.code = format!(r#"fn main() {{ {} }}"#, new_string)
        }
    }

    async fn create_script_file(&self) -> Result<(), String> {
        // create a main.rs file inside /tmp/<user>/main.rs
        if let Ok(_) = fs::create_dir_all(format!("/tmp/{}", self.user)).await {
            if let Ok(mut file) = File::create(format!("/tmp/{}/main.rs", self.user)).await {
                if file.write_all(self.code.as_bytes()).await.is_err() {
                    return Err("Could not write to file.".to_string());
                } else {
                    Ok(())
                }
            } else {
                Err("Could not create main file.".to_string())
            }
        } else {
            Err("Could not create directory.".to_string())
        }
    }

    async fn run_script_file(&mut self) {
        sleep(Duration::from_secs(4)).await;
        match Command::new("rust-script")
            .arg(format!("/tmp/{}/main.rs", self.user))
            .output()
            .await
        {
            Ok(output) => {
                if output.stdout.is_empty() {
                    // If stdout is empty, return stderr as the error message
                    self.output = Some(Err(String::from_utf8_lossy(&output.stderr).to_string()));
                } else {
                    // Otherwise, return stdout as the successful output
                    self.output = Some(Ok(String::from_utf8_lossy(&output.stdout).to_string()));
                }
            }
            Err(e) => {
                // Handle any errors that occur when running the command
                self.output = Some(Err(e.to_string()));
            }
        }
    }
}

#[async_trait]
impl CodeNote for RustCodeNote {
    fn from_signed_note(signed_note: &SignedNote) -> Result<Self, String> {
        if signed_note.verify_signature() == false || signed_note.verify_content() == false {
            return Err("Verification failed".to_string());
        }

        let code = signed_note.get_content().to_string();
        let user = signed_note.get_pubkey().to_string();
        let input_note = signed_note.get_id().to_string();
        Ok(RustCodeNote {
            code,
            code_vector: Vec::new(),
            output: None,
            input_note,
            user,
        })
    }

    async fn run(mut self) -> Self {
        self.create_vector_by_splitting_from_regex();
        self.remove_comments();
        self.ensure_main_function();
        match self.create_script_file().await {
            Ok(_) => self.run_script_file().await,
            Err(e) => self.output = Some(Err(e)),
        }
        self
    }

    fn create_output_note(&self, key_manager: LocalKeyManagerOpenssl) -> SignedNote {
        let content = match &self.output {
            Some(Ok(s)) => s.to_string(),
            Some(Err(e)) => e.to_string(),
            None => "No output".to_string(),
        };
        let mut note = Note::new(key_manager.get_public_key(), 301, &content);
        note.tag_note("a", &self.input_note);
        let signed_note = key_manager.sign_nostr_event(note);
        signed_note
    }
}
