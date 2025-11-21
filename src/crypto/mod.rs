pub mod gpg;
pub mod none;
pub mod passphrase;

use std::{any::Any, path::Path};

// re-export the cipher types
pub use gpg::GPG;
pub use none::NONE;
pub use passphrase::PASSPHRASE;

use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter, EnumString};

use crate::{error::Result, utils};

#[derive(Clone, PartialEq, Serialize, Deserialize, Display, EnumIter, EnumString)]
#[serde(rename_all = "lowercase")]
pub enum CipherKind {
    #[strum(ascii_case_insensitive, to_string = "none")]
    NONE,
    #[strum(ascii_case_insensitive, to_string = "passphrase")]
    PASSPHRASE,
    #[strum(ascii_case_insensitive, to_string = "gpg")]
    GPG,
}

pub trait Cipher: Any {
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>>;
    fn kind(&self) -> CipherKind;

    fn get_metadata(&self) -> Option<serde_json::Value> {
        None
    }

    #[allow(unused)]
    fn load_metadata(&mut self, data: serde_json::Value) -> Result<()> {
        Ok(())
    }

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub fn create_cipher(cipher_kind: CipherKind, key: Option<&str>) -> Result<Box<dyn Cipher>> {
    match cipher_kind {
        CipherKind::PASSPHRASE => Ok(Box::new(PASSPHRASE::new(key.unwrap_or_default().into()))),
        CipherKind::GPG => Ok(Box::new(GPG::new(key.unwrap_or_default().into()))),
        CipherKind::NONE => Ok(Box::new(NONE)),
    }
}

pub fn get_profile_cipher<P: AsRef<Path>>(profile_filepath: P) -> Result<Box<dyn Cipher>> {
    let serialized_profile = utils::get_serialized_profile(profile_filepath)?;

    Ok(create_cipher(
        serialized_profile.metadata.cipher_kind,
        None,
    )?)
}
