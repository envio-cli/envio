use std::io::{Read, Write};

use age::secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{Cipher, CipherKind},
    error::{Error, Result},
};

/// Represents the password-based cipher that `envio` provides
#[derive(Serialize, Deserialize)]
pub struct AGE {
    key: String,
}

impl Cipher for AGE {
    fn new(key: String) -> Self {
        AGE { key }
    }

    fn set_key(&mut self, key: String) {
        self.key = key;
    }

    fn get_key(&self) -> String {
        self.key.clone()
    }

    fn kind(&self) -> CipherKind {
        CipherKind::Age
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(self.key.to_owned()));

        let mut encrypted = vec![];
        let mut writer = match encryptor.wrap_output(&mut encrypted) {
            Ok(writer) => writer,
            Err(e) => {
                return Err(Error::Crypto(e.to_string()));
            }
        };

        writer.write_all(data)?;
        writer.finish()?;

        Ok(encrypted)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
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

        Ok(decrypted)
    }
}
