use std::{error::Error, sync::Mutex, fs, io};
use std::path::PathBuf;
use lazy_static::lazy_static;

const TOKEN_FILE: &str = ".gitlab_token";

lazy_static! {
    static ref TOKEN: Mutex<Option<String>> = Mutex::new(None);
}

pub fn login(token: &str) -> Result<(), Box<dyn Error>> {
    fs::write(TOKEN_FILE, token)?;
    println!("Token was stored successfully!");
    Ok(())
}

pub fn fetch_stored_token() -> Result<Option<String>, Box<dyn Error>> {
    match fs::read_to_string(TOKEN_FILE) {
        Ok(token) if !token.trim().is_empty() => Ok(Some(token.trim().to_string())),
        Ok(_) => Ok(None), // File exists but is empty
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            Err(Box::from("Please login first using 'devopscli login --token' command")) // Custom error message
        }
        Err(e) => Err(Box::new(e)), // Other errors wrapped in Box
    }
}
