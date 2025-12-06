#[macro_use]
mod metadata;
mod v1;

use std::any::Any;
use zeroize::Zeroizing;

use crate::{
    EnvMap,
    cipher::{Cipher, CipherKind, EncryptedContent},
    error::Result,
};

use metadata::VersionedMetadata;

include!(concat!(
    env!("OUT_DIR"),
    "/passphrase_decrypt_match_generated.rs"
));
include!(concat!(env!("OUT_DIR"), "/passphrase_encrypt_generated.rs"));

#[derive(Clone)]
pub struct PASSPHRASE {
    key: Zeroizing<String>,
    metadata: VersionedMetadata,
}

impl PASSPHRASE {
    pub fn new(key: Zeroizing<String>) -> Self {
        PASSPHRASE {
            key,
            metadata: VersionedMetadata::default(),
        }
    }

    pub fn set_key(&mut self, key: Zeroizing<String>) {
        self.key = key;
    }
}

impl Cipher for PASSPHRASE {
    fn kind(&self) -> CipherKind {
        CipherKind::PASSPHRASE
    }

    fn encrypt(&mut self, envs: &EnvMap) -> Result<EncryptedContent> {
        let data = envs.as_bytes()?;
        let (encrypted, metadata) = encrypt_latest(&self.key, &data)?;
        self.metadata = metadata;

        Ok(EncryptedContent::Bytes(encrypted))
    }

    fn decrypt(&self, encrypted_data: &EncryptedContent) -> Result<EnvMap> {
        Ok(decrypt_match!(self, &encrypted_data.as_bytes()?)?.into())
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
