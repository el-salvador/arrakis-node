use std::env;

use nostro2::notes::{Note, SignedNote};
use nostro2::userkeys::UserKeys;
use openssl::ec::EcKey;

#[derive(Clone, Debug)]
pub enum KeyManagerError {
    KeyNotFound,
    KeyNotValid,
    KeyNotLoaded,
    KeyNotDestroyed,
}

pub struct LocalKeyManagerOpenssl {
    keypair: UserKeys,
}

impl LocalKeyManagerOpenssl {

    pub fn new_from_pem(pem: String) -> Result<Self, KeyManagerError> {
        let pem_file_path = env::var("PEM_FILE").unwrap_or(pem);
        if let Ok(pem_file) = std::fs::read(pem_file_path) {
            if let Ok(buffer) = EcKey::private_key_from_pem(&pem_file) {
                let keypair = UserKeys::new(&buffer.private_key().to_hex_str().unwrap());
                Ok(LocalKeyManagerOpenssl {
                    keypair: keypair.unwrap(),
                })
            } else {
                Err(KeyManagerError::KeyNotValid)
            }
        } else {
            Err(KeyManagerError::KeyNotFound)
        }
    }
    pub fn get_public_key(&self) -> String {
        self.keypair.get_public_key()
    }
    pub fn sign_nostr_event(&self, note: Note) -> SignedNote {
        self.keypair.sign_nostr_event(note)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_manager_creation() {
        let key_manager = LocalKeyManagerOpenssl::new_from_pem("node_key.pem".to_string()).unwrap();
        let public_key = key_manager.get_public_key();
        assert_eq!(public_key.len(), 64);

    }

    #[test]
    fn test_key_manager_signing() {
        let key_manager = LocalKeyManagerOpenssl::new_from_pem("node_key.pem".to_string()).unwrap();
        let note = Note::new(
            key_manager.get_public_key(),
            100,
            "test"
        );
        let signed_note = key_manager.sign_nostr_event(note);
        assert_eq!(signed_note.verify_signature(), true);
        assert_eq!(signed_note.verify_content(), true);
    }

}
