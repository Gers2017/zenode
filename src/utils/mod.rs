use p2panda_rs::identity::KeyPair;

use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Helper function to write a file.
pub fn write_file(path: &PathBuf, content: &str) {
    let mut file =
        File::create(path).unwrap_or_else(|_| panic!("Could not create file {:?}", path));
    write!(&mut file, "{}", content).unwrap();
}

/// Helper function to read a private key from a file, deriving a key pair from it. If it doesn't
/// exist yet, a new key pair will be generated automatically.
pub fn get_key_pair(path: Option<PathBuf>) -> KeyPair {
    let path = path.unwrap_or_else(|| PathBuf::from("key.txt"));

    // Read private key from file or generate a new one
    let private_key = if Path::exists(&path) {
        let key = read_to_string(path).expect("Couldn't read file!");
        key.replace('\n', "")
    } else {
        let key = hex::encode(KeyPair::new().private_key().to_bytes());
        write_file(&path, &key);
        key
    };

    // Derive key pair from private key
    KeyPair::from_private_key_str(&private_key).expect("Invalid private key")
}

/*
/// Transforms a StringTuple (name and value) to a json field
/// ### Example:
/// input: `(PI, 3.1416)` output: `"PI": 3.1416`
 pub fn field_to_json((name, value): &StringTuple) -> String {
    let value = (*value).to_string();

    if value == "true" || value == "false" {
        return format!(r#""{}": {}"#, name, value);
    }

    // For relation_list, pinned_relation and pinned_relation_list
    if value.starts_with('[') && value.ends_with(']') {
        return format!(r#""{}": {}"#, name, value);
    }

    if let Ok(x) = value.parse::<f64>() {
        if value.contains('.') {
            return format!(r#""{}": {:?}"#, name, x);
        } else {
            return format!(r#""{}": {}"#, name, x.round());
        }
    }

    format!(r#""{}": "{}""#, name, value)
} */
