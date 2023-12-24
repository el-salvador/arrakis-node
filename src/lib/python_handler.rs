use tokio::{fs::{self, File}, io::AsyncWriteExt, process::Command};
use crate::utils::CodeNote;
use async_trait::async_trait;

pub struct PythonCodeNote {
    pub code: String,
    pub output: Option<Result<String, String>>,
    pub input_note: String,
    pub user: String,
}

impl PythonCodeNote {
    async fn create_script_file(&self) -> Result<(), String> {
        // create a main.rs file inside /tmp/<user>/main.rs
        if let Ok(_) = fs::create_dir_all(format!("/tmp/{}", self.user)).await {
            if let Ok(mut file) = File::create(format!("/tmp/{}/script.py", self.user)).await {
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

    async fn format_script_file(&self) -> Result<(), String> {
        match Command::new("black")
            .arg(format!("/tmp/{}/script.py", self.user))
            .output()
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => Err(e.to_string()),
        }
    }

    async fn run_script_file(&self) -> Result<String, String> {
        match Command::new("python3")
            .arg(format!("/tmp/{}/script.py", self.user))
            .output()
            .await
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

    pub async fn create_script_output(&mut self) {
        match self.create_script_file().await {
            Ok(_) => match self.format_script_file().await {
                Ok(_) => self.output = Some(self.run_script_file().await), 
                Err(e) => self.output = Some(Err(e)),
            },
            Err(e) => self.output = Some(Err(e)),
        }
    }
}

#[async_trait]
impl CodeNote for PythonCodeNote {
    fn from_signed_note(signed_note: &nostro2::notes::SignedNote) -> Result<Self, String>
    where
        Self: Sized,
    {
        if signed_note.verify_signature() == false || signed_note.verify_content() == false {
            return Err("Verification failed".to_string());
        }

        let code = signed_note.get_content().to_string();
        let user = signed_note.get_pubkey().to_string();
        let input_note = signed_note.get_id().to_string();

        Ok(PythonCodeNote {
            code,
            output: None,
            input_note,
            user,
        })
    }

    async fn run(mut self) -> Self {
        self.create_script_output().await;
        self
    }

    fn create_output_note(
        &self,
        key_manager: crate::local_key_handler::LocalKeyManagerOpenssl,
    ) -> nostro2::notes::SignedNote
    where
        Self: Sized,
    {
        let content = match &self.output {
            Some(Ok(s)) => s.to_string(),
            Some(Err(e)) => e.to_string(),
            None => "No output".to_string(),
        };
        let mut note = nostro2::notes::Note::new(key_manager.get_public_key(), 301, &content);
        note.tag_note("a", self.input_note.as_str());
        let signed_note = key_manager.sign_nostr_event(note);
        signed_note
    }
}
