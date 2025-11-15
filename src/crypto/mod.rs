pub mod age;
pub mod gpg;

// Re-export the cipher types so that users don't have to use envio::crypto::type::TYPE
use std::path::Path;

pub use age::AGE;
pub use gpg::GPG;
use serde::{Deserialize, Serialize};

use crate::{error::Result, utils};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
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

pub fn create_cipher(key: String, cipher_kind: CipherKind) -> Result<Box<dyn Cipher>> {
    match cipher_kind {
        CipherKind::Age => Ok(Box::new(AGE::new(key))),
        CipherKind::Gpg => Ok(Box::new(GPG::new(key))),
    }
}

pub fn get_cipher<P: AsRef<Path>>(profile_filepath: P) -> Result<Box<dyn Cipher>> {
    let serialized_profile = utils::get_serialized_profile(profile_filepath)?;

    let cipher = create_cipher(String::new(), serialized_profile.metadata.cipher_kind)?;

    Ok(cipher)
}
