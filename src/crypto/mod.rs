pub mod age;
pub mod gpg;

use std::path::Path;

// re-export the cipher types
pub use age::AGE;
pub use gpg::GPG;
use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumIter};

use crate::{error::Result, utils};

#[derive(Clone, PartialEq, Serialize, Deserialize, Display, EnumIter)]
#[serde(rename_all = "lowercase")]
pub enum CipherKind {
    Age,
    Gpg,
}

pub trait Cipher {
    fn new(key: String) -> Self
    where
        Self: Sized;

    fn set_key(&mut self, key: String);
    fn get_key(&self) -> String;
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>>;
    fn kind(&self) -> CipherKind;
}

pub fn create_cipher(cipher_kind: CipherKind, key: Option<&str>) -> Result<Box<dyn Cipher>> {
    match cipher_kind {
        CipherKind::Age => Ok(Box::new(AGE::new(key.unwrap_or_default().into()))),
        CipherKind::Gpg => Ok(Box::new(GPG::new(key.unwrap_or_default().into()))),
    }
}

pub fn get_profile_cipher<P: AsRef<Path>>(profile_filepath: P) -> Result<Box<dyn Cipher>> {
    let serialized_profile = utils::get_serialized_profile(profile_filepath)?;

    Ok(create_cipher(
        serialized_profile.metadata.cipher_kind,
        None,
    )?)
}
