use std::io::Error;

use std::io::{Read, Write};

use age::secrecy::Secret;
use colored::Colorize;

use crate::crypto::EncryptionType;

/*
 * AGE encryption - its not a real encryption type, but a wrapper around the age crate to keep the
 * same interface as the other encryption types
*/
pub struct AGE {
    key: String,
}

impl EncryptionType for AGE {
    fn new(key: String) -> Self {
        AGE { key }
    }

    fn set_key(&mut self, key: String) {
        self.key = key;
    }

    fn get_key(&self) -> String {
        self.key.clone()
    }

    fn as_string(&self) -> &'static str {
        "age"
    }

    fn encrypt(&self, data: &str) -> Vec<u8> {
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(self.key.to_owned()));

        let mut encrypted = vec![];
        let mut writer = match encryptor.wrap_output(&mut encrypted) {
            Ok(writer) => writer,
            Err(e) => {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }
        };

        if let Err(e) = writer.write_all(data.as_bytes()) {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        };

        if let Err(e) = writer.finish() {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        };

        encrypted
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<String, Error> {
        let decryptor = match age::Decryptor::new(encrypted_data).unwrap() {
            age::Decryptor::Passphrase(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted = vec![];
        let mut reader = match decryptor.decrypt(&Secret::new(self.key.to_owned()), None) {
            Ok(reader) => reader,
            Err(e) => {
                return Err(Error::new(std::io::ErrorKind::Other, e));
            }
        };

        if let Err(e) = reader.read_to_end(&mut decrypted) {
            return Err(Error::new(std::io::ErrorKind::Other, e));
        }

        String::from_utf8(decrypted).map_err(|e| Error::new(std::io::ErrorKind::Other, e))
    }
}
