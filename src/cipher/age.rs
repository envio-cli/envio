use std::{
    any::Any,
    io::{Cursor, Read, Write},
    iter,
};

use age::{Decryptor, Encryptor, scrypt::Identity, secrecy::SecretString};
use zeroize::Zeroizing;

use crate::{
    EnvMap,
    cipher::{Cipher, CipherKind, EncryptedContent},
    error::{Error, Result},
};

#[derive(Clone)]
pub struct AGE {
    key: Zeroizing<String>,
}

impl AGE {
    pub fn new(key: Zeroizing<String>) -> Self {
        AGE { key }
    }

    pub fn set_key(&mut self, key: Zeroizing<String>) {
        self.key = key;
    }
}

impl Cipher for AGE {
    fn kind(&self) -> CipherKind {
        CipherKind::AGE
    }

    fn encrypt(&mut self, envs: &EnvMap) -> Result<EncryptedContent> {
        let data = envs.as_bytes()?;

        let encryptor = Encryptor::with_user_passphrase(SecretString::from(self.key.as_str()));

        let mut encrypted = vec![];
        let mut writer = encryptor
            .wrap_output(&mut encrypted)
            .map_err(|e| Error::Cipher(e.to_string()))?;

        writer.write_all(&data)?;
        writer.finish()?;

        Ok(EncryptedContent::Bytes(encrypted))
    }

    fn decrypt(&self, encrypted_data: &EncryptedContent) -> Result<EnvMap> {
        let decryptor = Decryptor::new(Cursor::new(encrypted_data.as_bytes()?))
            .map_err(|e| Error::Cipher(e.to_string()))?;

        let mut decrypted = vec![];
        let mut reader = decryptor
            .decrypt(iter::once(
                &Identity::new(SecretString::from(self.key.as_str())) as _,
            ))
            .map_err(|e| Error::Cipher(e.to_string()))?;

        reader.read_to_end(&mut decrypted)?;

        Ok(decrypted.into())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
