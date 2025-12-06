pub mod age;
pub mod gpg;
pub mod none;
pub mod passphrase;

// re-export the cipher types
pub use age::AGE;
use colored::Colorize;
pub use gpg::GPG;
pub use none::NONE;
pub use passphrase::PASSPHRASE;

use dyn_clone::DynClone;
use serde::{Deserialize, Serialize};
use serde_with::{base64::Base64, serde_as};
use std::{any::Any, path::Path};
use strum_macros::{AsRefStr, EnumIter, EnumString};
use zeroize::Zeroizing;

use crate::{env::EnvMap, error::Result, utils};

#[derive(Copy, Clone, PartialEq, Serialize, Deserialize, EnumIter, EnumString, AsRefStr)]
#[serde(rename_all = "lowercase")]
pub enum CipherKind {
    #[strum(ascii_case_insensitive, to_string = "none")]
    NONE,
    #[strum(ascii_case_insensitive, to_string = "passphrase")]
    PASSPHRASE,
    #[strum(ascii_case_insensitive, to_string = "age", props(label = "BETA"))]
    AGE,
    #[strum(ascii_case_insensitive, to_string = "gpg")]
    GPG,
}

impl std::fmt::Display for CipherKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CipherKind::AGE => write!(
                f,
                "{} {}",
                self.as_ref(),
                "[BETA] https://crates.io/crates/age".bold().yellow()
            ),
            _ => write!(f, "{}", self.as_ref()),
        }
    }
}

#[serde_as]
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum EncryptedContent {
    Bytes(#[serde_as(as = "Base64")] Vec<u8>),
    Json(serde_json::Value),
}

impl EncryptedContent {
    pub fn as_bytes(&self) -> Result<Vec<u8>> {
        match self {
            EncryptedContent::Bytes(b) => Ok(b.clone()),
            EncryptedContent::Json(value) => Ok(serde_json::to_vec(value)?),
        }
    }
}

pub trait Cipher: Any + Send + DynClone {
    fn encrypt(&mut self, envs: &EnvMap) -> Result<EncryptedContent>;
    fn decrypt(&self, encrypted_data: &EncryptedContent) -> Result<EnvMap>;
    fn kind(&self) -> CipherKind;

    fn export_metadata(&self) -> Option<serde_json::Value> {
        None
    }

    #[allow(unused)]
    fn import_metadata(&mut self, data: serde_json::Value) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl Clone for Box<dyn Cipher> {
    fn clone(&self) -> Box<dyn Cipher> {
        dyn_clone::clone_box(self.as_ref())
    }
}

pub fn create_cipher(
    cipher_kind: CipherKind,
    key: Option<Zeroizing<String>>,
) -> Result<Box<dyn Cipher>> {
    match cipher_kind {
        CipherKind::NONE => Ok(Box::new(NONE)),
        CipherKind::PASSPHRASE => Ok(Box::new(PASSPHRASE::new(key.unwrap_or_default()))),
        CipherKind::AGE => Ok(Box::new(AGE::new(key.unwrap_or_default()))),
        CipherKind::GPG => Ok(Box::new(GPG::new(key.unwrap_or_default().to_string()))),
    }
}

pub fn get_profile_cipher<P: AsRef<Path>>(profile_filepath: P) -> Result<Box<dyn Cipher>> {
    let serialized_profile = utils::get_serialized_profile(profile_filepath)?;

    create_cipher(serialized_profile.metadata.cipher_kind, None)
}
