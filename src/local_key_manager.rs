use ed25519_dalek::{
  VerifyingKey,
  Signature,
  SigningKey
};
use ed25519_dalek::pkcs8::{EncodePrivateKey, DecodePrivateKey, DecodePublicKey};
use nostro2::userkeys::{UserKeys};
use std::fs::File;
use std::io::BufReader;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::str::{from_utf8_mut};

use rand::{thread_rng, Rng, rngs::OsRng};

// I would like to create a struct with implementations
// in which will help the user generate and assign private 
// public key pairs to a session from the agents local machine.
// These keys should only exists to encrypt internal messages at 
// the local layer. When the note is sent out, the note should be
// left to be interpreted in the manner the user intended. 
// We will be using the ed25519_dalek library to generate the keys,
// and to sign the messages.
// These manager should also have a cleanup function that will remove
// the files from the local machine. Considering that the only intention to 
// have these keys is for internal traffic and encryption over the wire in case
// microservices live in different realms.

pub struct LocalKeyManager {
  pub private_key: [u8; 32],
  pub public_key:[u8; 32],
  pub pem_file_location: String,
  pub pem_file_name: String,
}

#[derive(Debug)]
pub enum LocalKeyManagerError {
  IoError(std::io::Error),
  ParseError,
  CustomError(String),
}

impl Error for LocalKeyManagerError {}

impl std::fmt::Display for LocalKeyManagerError {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    match self {
      LocalKeyManagerError::IoError(e) => write!(f, "IO Error: {}", e),
      LocalKeyManagerError::ParseError => write!(f, "Parse Error"),
      LocalKeyManagerError::CustomError(e) => write!(f, "Custom Error: {}", e),
    }
  }
}

impl From<std::io::Error> for LocalKeyManagerError {
  fn from(e: std::io::Error) -> Self {
    LocalKeyManagerError::IoError(e)
  }
}

impl std::fmt::Display for LocalKeyManager {
  fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
    write!(f, "Private Key: {}\nPublic Key: {}\nPem File Location: {}\nPem File Name: {}", self.private_key, self.public_key, self.pem_file_location, self.pem_file_name)
  }
}

impl LocalKeyManager {
  // Generate the full path of the file location
  fn generate_file_name(self) -> Result<String, LocalKeyManagerError> {
    let mut rng = thread_rng();
    let mut file_name = String::new();
    for _ in 0..10 {
      let random_number: char = rng.gen();
      file_name.push(random_number);
    }
    Ok(file_name)
  }
  fn pem_file_location(self) -> Result<String, LocalKeyManagerError> {
    let file_name = self.generate_file_name().unwrap();
    let file_root = String::from("~/.ssh/arrakis_keys/");
    let pem_file_location = file_root + file_name.as_str() + ".pem";
    Ok(pem_file_location)

  }
  fn extract_keys_from_pem(self, pem_file: String) -> Result<([u8;32],[u8;32]), LocalKeyManagerError> {
    let pem_file = String::from_utf8_lossy(&std::fs::read(pem_file).unwrap());
    let keypair = SigningKey::from_pkcs8_pem(&pem_file).unwrap();
    let signing_key = keypair.to_bytes();
    let verifying_key = keypair.verifying_key().to_bytes();
    Ok((signing_key, verifying_key))
  }
  // Use for testing as of now, the ssh keys should be generated by the
  // arrakis node host, or if the user wants to generate their own keys
  // they can do so and pass them in as a parameter.
  fn generate_pem_file(self) -> Result<(String,String), LocalKeyManagerError> {
    // Step 1 - Choose a location and name of the *.pem file. The name
    // should be generated by the the thread processing the request + unixtimestamp all hashed together.
    let file_name = String::from(self.generate_file_name().unwrap().as_str()) + ".pem";
    let full_path = String::from(self.pem_file_location().unwrap().as_str()) + file_name.as_str();
    let mut csprng = OsRng;
    let signing_key: SigningKey = SigningKey::generate(&mut csprng);
    let pem_string = EncodePrivateKey::to_pkcs8_pem(&signing_key, );
    let mut file = std::fs::File::create(&full_path)?;
    file.write_all(pem_string.as_bytes())?;
    Ok((full_path, file_name))
  }
  pub fn get_public_key(self) -> Result<[u8;32], LocalKeyManagerError> {
    Ok(self.public_key)
  }
  // The new function will generate a new key pair into a pem file,
  // and store them in a designated directory.
  // The goal is to be able to eventually store these keys in a 
  // hardware wallet or hashicorp vault.
  pub fn new() -> Self {
    let initial = LocalKeyManager {
      private_key: [0;32],
      public_key: [0;32],
      pem_file_location: String::new(),
      pem_file_name: String::new(),
    };
    let (file_location, file_name) = initial.generate_pem_file().unwrap();
    let file_location = format!("{}/{}", file_location, file_name);
    let (private_key_pem, public_key_pem) = initial.extract_keys_from_pem(file_location).unwrap();
    Self {
      private_key: private_key_pem,
      public_key: public_key_pem,
      pem_file_location: file_location,
      pem_file_name: file_name,
    }
  }
}