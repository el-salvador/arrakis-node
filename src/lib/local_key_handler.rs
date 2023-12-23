use std::env;

use nostro2::notes::{Note, SignedNote};
use nostro2::userkeys::UserKeys;
use openssl::ec::EcKey;

#[derive(Debug)]
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
