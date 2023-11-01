// As a developer, I would like to test my src/run_rust.rs
// functions.

use getmessages_ms::run_rust::{
    generate_random_hash_string,
    ensure_main_function,
    create_file_with_content
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_rust_code() {
        let hash_string = generate_random_hash_string();
        // the length of the hash_string should be 128
        // because a rng.gen of u32 is converted to a string
        // and then hashed with sha256 and then encoded with hex.
        // 128 is the length of the hex encoded sha256 hash.
        assert_eq!(&hash_string.unwrap().len(), &128);
    }

    #[test]
    fn test_ensure_main_function_without_main() {
        let code = vec![
            "println!(\"Hello, world!\");".to_string()
        ];
        let result = ensure_main_function(code);
        assert_eq!(
            result,
            "fn main() { println!(\"Hello, world!\"); }".to_string()
        );
    }

    #[test]
    fn test_ensure_main_function_with_main(){
        let code = vec![
            "fn main() {".to_string(),
            "println!(\"Hello, world!\");".to_string(),
            "}".to_string()
        ];
        let result = ensure_main_function(code);
        assert_eq!(
            result,
            "fn main() { println!(\"Hello, world!\"); }".to_string()
        );
    }

    #[test]
    fn test_create_file_with_content() {
        let code = vec![
            "// This is a comment".to_string(),
            "fn main() {".to_string(),
            "println!(\"Hello, world!\");".to_string(),
            "}".to_string()
        ];
        let code_ensured = ensure_main_function(code);
        let result = create_file_with_content(code_ensured);
        assert_eq!(
            result.unwrap(),
            "Hello, world!\n".to_string()
        );
    }
    #[test]
    fn test_create_file_with_comment_and_no_main() {
        let code = vec![
            "// This is a comment".to_string(),
            "println!(\"Hello, world!\");".to_string(),
        ];
        let code_ensured = ensure_main_function(code);
        let result = create_file_with_content(code_ensured);
        assert_eq!(
            result.unwrap(),
            "Hello, world!\n".to_string()
        );
    }
}

