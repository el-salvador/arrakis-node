// I would like to use the openssl library for rust to
// write a struct with implementations that will
// do the following
// 1. Retrieve a local public key from a pem file defined
//   by a yaml configuration file. The pem file is generated
//   by the openssl secpk26k1 algorithm cli which creates a pem file.
// 2. Use Arc<str> to store the public key in memory
// 3. Destroys all evidence of the location of the key in memory
//   after the key is used.
// 4. Makes sure that errors are defined for this specific struct,
//   along with being able to adapt the error messages to other
//   monitoring systems.

use nostro2::notes::{Note, SignedNote};
use nostro2::userkeys::UserKeys;
use openssl::ec::EcKey;

pub struct LocalKeyManagerOpenssl {
    keypair: UserKeys,
}

impl LocalKeyManagerOpenssl {
    pub fn new(path: String) -> LocalKeyManagerOpenssl {
        // Assign the path to the pem file to a variable
        let file = std::fs::read(path).expect("Unable to read file");
        // Obtain the public key from the pem file in buffer format
        let buffer = EcKey::private_key_from_pem(&file);

        let keypair = UserKeys::new(&buffer.unwrap().private_key().to_hex_str().unwrap());
        LocalKeyManagerOpenssl {
            keypair: keypair.unwrap(),
        }
    }
    pub fn get_public_key(&self) -> String {
        self.keypair.get_public_key()
    }
    pub fn sign_nostr_event(&self, note: Note) -> SignedNote {
        self.keypair.sign_nostr_event(note)
    }
}
