// This file will have a function in which will take rust string.
// We will have a detector in which will determine if the string is rust.
// If it is rust, we will determine if the string is surrounded by
// a main function. If it is not, we will add a main function.
// We will then create a file with the string and compile it.
// We will then run the file and return the output.
use regex::Regex;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use hex::encode;
use rand::Rng;
use sha256::digest;

pub fn generate_random_hash_string() -> Result<String, Box<dyn Error>> {
    let mut rng = rand::thread_rng();
    let random_string: String = rng.gen::<u32>().to_string();
    let hash = digest(random_string.as_bytes());
    let hash_string = encode(hash);
    Ok(hash_string)
}

pub fn create_file_with_content(new_code: String) -> Result<String, Box<dyn Error>> {
    // Create a hash string
    let hash: String = generate_random_hash_string().unwrap();
    // create a main.rs file inside /tmp/<hash>/main.rs
    fs::create_dir_all(format!("/tmp/{}", hash))?;
    let mut file = File::create(format!("/tmp/{}/main.rs", hash))?;
    // write the code to the file.
    file.write_all(new_code.as_bytes())?;
    // Check if the file exists.
    if Path::new(&format!("/tmp/{}/main.rs", hash)).exists() {
        // Assign the output command to retrieve the script stdout.
        let output = Command::new("rust-script")
            .arg(format!("/tmp/{}/main.rs", hash))
            .output()
            .expect("failed to execute process");

        Ok(output.stdout.to_vec()[..]
            .to_vec()
            .into_iter()
            .map(|x| x as char)
            .collect::<String>())
    } else {
        Ok("File does not exists".to_string())
    }
}

// I would like to iterate through every string in a vector
// and capture if there is a comment in that vector.
// If there is no comment then return the vector as is.
// If there is a comment on the vector, I would like to pop
// that comment and return the vector without the comment.
pub fn remove_comments(s: Vec<String>) -> Vec<String> {
    let mut new_vec: Vec<String> = Vec::new();
    for i in s {
        if i.contains("//") {
            let mut split = i.split("//");
            let first = split.next().unwrap();
            new_vec.push(first.to_string());
        } else {
            new_vec.push(i);
        }
    }
    new_vec
}

pub fn ensure_main_function(s: Vec<String>) -> String {
    // double check that comments are gone
    let s = remove_comments(s);
    let re = Regex::new(r"(?s)fn main\s*\(\s*\)\s*\{.*\}").unwrap();
    let new_string = s.join(" ");
    if re.is_match(&new_string) {
        new_string.to_string()
    } else {
        format!(r#"fn main() {{ {} }}"#, new_string)
    }
}

pub fn create_rust_file_compile_and_run(content: Vec<String>) -> Result<String, Box<dyn Error>> {
    // Remove comments
    let content = remove_comments(content);
    // Check that code is wrapped by main function
    let new_code = ensure_main_function(content);
    // Check with clippy
    let result = create_file_with_content(new_code);
    result
}
