pub mod age;
pub mod gpg;

// Re-export the cipher types so that users don't have to use envio::crypto::type::TYPE
pub use age::AGE;
pub use gpg::GPG;

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{
    error::{Error, Result},
    utils,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CipherKind {
    Age,
    Gpg,
}

impl CipherKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            CipherKind::Age => "age",
            CipherKind::Gpg => "gpg",
        }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            s if s.eq_ignore_ascii_case("age") => Ok(CipherKind::Age),
            s if s.eq_ignore_ascii_case("gpg") => Ok(CipherKind::Gpg),
            _ => Err(Error::InvalidCipherType(s.to_string())),
        }
    }
}

impl std::fmt::Display for CipherKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
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
