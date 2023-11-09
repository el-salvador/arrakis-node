// As a developer, I would like to create test for my local key manager
// so that I can verify that it works correctly.


use getmessages_ms::local_key_manager_openssl::{LocalKeyManagerOpenssl};
use getmessages_ms::utils::{retrieve_pem_file_path};
use nostro2::notes::{Note, SignedNote};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_that_public_key_can_be_retrieved() {
        let file_path = retrieve_pem_file_path("private-diego-stash.pem".to_string()).unwrap();
        let key_manager = LocalKeyManagerOpenssl::new(file_path);
        let public_key = key_manager.get_public_key();
        assert_eq!(public_key.len(), 64);

    }

    #[test]
    fn test_that_given_a_note_a_signed_note_struct_is_returned() {
        let file_path = retrieve_pem_file_path("private-diego-stash.pem".to_string()).unwrap();
        let key_manager = LocalKeyManagerOpenssl::new(file_path);
        let mut note = Note::new(
            key_manager.get_public_key(),
            100,
            "test"
        );
        let signed_note = key_manager.sign_nostr_event(note);
        assert_eq!(signed_note.verify_signature(), true);
    }

}
