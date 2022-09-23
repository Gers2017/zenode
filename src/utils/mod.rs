use p2panda_rs::identity::KeyPair;

use crate::StrTuple;

use std::fs::{read_to_string, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Utility function to sort `Vec<StrTuple>` in alphabetical order
/// p2panda requires the fields in alphabetical order
pub fn sort_fields<'a>(fields: &mut Vec<StrTuple<'a>>) {
    fields.sort_by(|a, b| a.0.cmp(b.0))
}

/// Helper function to write a file.
pub fn write_file(path: &PathBuf, content: &str) {
    let mut file =
        File::create(path).unwrap_or_else(|_| panic!("Could not create file {:?}", path));
    write!(&mut file, "{}", content).unwrap();
}

/// Helper function to read a private key from a file, deriving a key pair from it. If it doesn't
/// exist yet, a new key pair will be generated automatically.
pub fn get_key_pair(path: Option<PathBuf>) -> KeyPair {
    let path = path.unwrap_or(PathBuf::from("key.txt"));

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

/// Utility function to map a `Vec<StrTuple>` to `Vec<String>`
/// The resulting string has the shape: `"a": "b"` or `"a": b` if b is a number or boolean
pub fn fields_to_json_fields<'a>(fields: &Vec<StrTuple<'a>>) -> Vec<String> {
    fields
        .iter()
        .map(|(name, value)| -> String {
            let value = (**value).to_string();

            if value == "true" || value == "false" {
                return format!(r#""{}": {}"#, name, value);
            }

            if let Ok(x) = value.parse::<f64>() {
                return format!(r#""{}": {}"#, name, x);
            }

            return format!(r#""{}": "{}""#, name, value);
        })
        .collect()
}
