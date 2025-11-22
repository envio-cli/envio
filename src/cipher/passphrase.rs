use std::{
    any::Any,
    io::{Read, Write},
};

use age::secrecy::Secret;
use serde::{Deserialize, Serialize};

use crate::{
    cipher::{Cipher, CipherKind},
    error::{Error, Result},
};

#[derive(Serialize, Deserialize, Clone)]
pub struct PASSPHRASE {
    key: String,
}

impl PASSPHRASE {
    pub fn new(key: String) -> Self {
        PASSPHRASE { key }
    }

    pub fn set_key(&mut self, key: String) {
        self.key = key;
    }

    pub fn get_key(&self) -> String {
        self.key.clone()
    }
}

impl Cipher for PASSPHRASE {
    fn kind(&self) -> CipherKind {
        CipherKind::PASSPHRASE
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(self.key.to_owned()));

        let mut encrypted = vec![];
        let mut writer = match encryptor.wrap_output(&mut encrypted) {
            Ok(writer) => writer,
            Err(e) => {
                return Err(Error::Cipher(e.to_string()));
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
                return Err(Error::Cipher(e.to_string()));
            }
        };

        reader.read_to_end(&mut decrypted)?;

        Ok(decrypted)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
