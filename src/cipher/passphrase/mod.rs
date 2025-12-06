#[macro_use]
mod metadata;
mod v1;

use serde::{Deserialize, Serialize};
use std::any::Any;

use crate::{
    cipher::{Cipher, CipherKind},
    error::Result,
};

use metadata::VersionedMetadata;

include!(concat!(
    env!("OUT_DIR"),
    "/passphrase_decrypt_match_generated.rs"
));
include!(concat!(env!("OUT_DIR"), "/passphrase_encrypt_generated.rs"));

#[derive(Serialize, Deserialize, Clone)]
pub struct PASSPHRASE {
    key: String,
    metadata: VersionedMetadata,
}

impl PASSPHRASE {
    pub fn new(key: String) -> Self {
        PASSPHRASE {
            key,
            metadata: VersionedMetadata::default(),
        }
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

    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let (encrypted, metadata) = encrypt_latest(&self.key, data)?;
        self.metadata = metadata;
        Ok(encrypted)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        decrypt_match!(self, encrypted_data)
    }

    fn export_metadata(&self) -> Option<serde_json::Value> {
        serde_json::to_value(self.metadata.clone()).ok()
    }

    fn import_metadata(&mut self, data: serde_json::Value) -> Result<()> {
        self.metadata = serde_json::from_value(data)?;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
