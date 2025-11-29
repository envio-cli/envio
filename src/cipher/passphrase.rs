use std::{
    any::Any,
    io::{Read, Write},
    iter,
};

use age::{Decryptor, Encryptor, scrypt::Identity, secrecy::SecretString};
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
        let encryptor = Encryptor::with_user_passphrase(SecretString::from(self.key.to_owned()));

        let mut encrypted = vec![];
        let mut writer = encryptor
            .wrap_output(&mut encrypted)
            .map_err(|e| Error::Cipher(e.to_string()))?;

        writer.write_all(data)?;
        writer.finish()?;

        Ok(encrypted)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let decryptor = Decryptor::new(encrypted_data).map_err(|e| Error::Cipher(e.to_string()))?;

        let mut decrypted = vec![];
        let mut reader = decryptor
            .decrypt(iter::once(
                &Identity::new(SecretString::from(self.key.to_owned())) as _,
            ))
            .map_err(|e| Error::Cipher(e.to_string()))?;

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
