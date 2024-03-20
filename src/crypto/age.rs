use std::io::{Read, Write};

use age::secrecy::Secret;

use crate::crypto::EncryptionType;
use crate::error::{Error, Result};

// Bytes that identify the file as being encrypted using the `age` method
pub const IDENTITY_BYTES: &[u8] = b"-----AGE ENCRYPTED FILE-----";

/// AGE is not a real encryption type, but rather a wrapper around the `age` crate
/// It is supposed to represent the password-based encryption method that `envio` provides
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

    fn encrypt(&self, data: &str) -> Result<Vec<u8>> {
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(self.key.to_owned()));

        let mut encrypted = vec![];
        let mut writer = match encryptor.wrap_output(&mut encrypted) {
            Ok(writer) => writer,
            Err(e) => {
                return Err(Error::Crypto(e.to_string()));
            }
        };

        writer.write_all(data.as_bytes())?;
        writer.finish()?;

        encrypted.extend_from_slice(IDENTITY_BYTES);

        Ok(encrypted)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<String> {
        let decryptor = match age::Decryptor::new(encrypted_data).unwrap() {
            age::Decryptor::Passphrase(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted = vec![];
        let mut reader = match decryptor.decrypt(&Secret::new(self.key.to_owned()), None) {
            Ok(reader) => reader,
            Err(e) => {
                return Err(Error::Crypto(e.to_string()));
            }
        };

        reader.read_to_end(&mut decrypted)?;

        String::from_utf8(decrypted).map_err(|e| Error::Utf8Error(e.utf8_error()))
    }

    fn is_this_type(encrypted_data: &[u8]) -> bool {
        encrypted_data.len() >= IDENTITY_BYTES.len()
            && &encrypted_data[encrypted_data.len() - IDENTITY_BYTES.len()..] == IDENTITY_BYTES
    }
}
