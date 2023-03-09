use p2panda_rs::identity::KeyPair;
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_PATH: &str = "key.txt";

/// Helper function to read a private key from a file, deriving a key pair from it. If it doesn't
/// exist yet, a new key pair will be generated automatically.
pub fn get_key_pair(path: Option<PathBuf>) -> KeyPair {
    let path = path.unwrap_or_else(|| PathBuf::from(DEFAULT_PATH));

    // Read private key from file or generate a new one
    let private_key = if Path::exists(&path) {
        let key = fs::read_to_string(path).expect("Couldn't read key");
        key.trim().replace('\n', "")
    } else {
        let key = hex::encode(KeyPair::new().private_key().to_bytes());
        fs::write(&path, &key).expect("Couldn't write key");
        key
    };

    // Derive key pair from private key
    KeyPair::from_private_key_str(&private_key).expect("Invalid private key")
}
