pub mod age;
pub mod gpg;

// Re-export the encryption types so that users don't have to use envio::crypto::type::TYPE
pub use age::AGE;
pub use gpg::GPG;

use std::path::Path;

use crate::{
    error::{Error, Result},
    utils,
};

pub const ENCRYPTION_TYPE_AGE: &str = "age";
pub const ENCRYPTION_TYPE_GPG: &str = "gpg";

#[typetag::serde(tag = "type")]
pub trait EncryptionType {
    fn new(key: String) -> Self
    where
        Self: Sized;

    fn set_key(&mut self, key: String);
    fn get_key(&self) -> String;
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;
    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>>;
    fn as_string(&self) -> &'static str;
}

pub fn create_encryption_type(
    key: String,
    encryption_type_str: &str,
) -> Result<Box<dyn EncryptionType>> {
    match encryption_type_str {
        ENCRYPTION_TYPE_AGE => Ok(Box::new(AGE::new(key))),
        ENCRYPTION_TYPE_GPG => Ok(Box::new(GPG::new(key))),
        _ => Err(Error::InvalidEncryptionType(
            encryption_type_str.to_string(),
        )),
    }
}

pub fn get_encryption_type<P: AsRef<Path>>(profile_filepath: P) -> Result<Box<dyn EncryptionType>> {
    let serialized_profile = utils::get_serialized_profile(profile_filepath)?;
    
    let encryption_type = create_encryption_type(
        String::new(),
        &serialized_profile.metadata.encryption_type_name,
    )?;

    Ok(encryption_type)
}
