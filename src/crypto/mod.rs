pub mod age;
pub mod gpg;

// Re-export the encryption types so that users don't have to use envio::crypto::type::TYPE
pub use age::AGE;
pub use gpg::GPG;

use crate::{
    error::{Error, Result},
    utils,
};

/// Trait for encryption types
///
/// Used to define the methods that an encryption type must implement.
#[typetag::serde(tag = "type")]
pub trait EncryptionType {
    fn new(key: String) -> Self
    where
        Self: Sized;

    /// Set the key used for encryption/decryption
    ///
    /// The key is the fingerprint of your gpg key or the password for the age
    ///
    /// # Parameters
    /// - `key`: String - the key to use for encryption/decryption
    fn set_key(&mut self, key: String);

    /// Get the key used for encryption/decryption
    ///
    /// # Returns
    /// - `String`: the key
    fn get_key(&self) -> String;

    /// Encrypt data
    ///
    /// # Parameters
    /// - `data`: &[u8] - the data to encrypt
    ///
    /// # Returns
    /// - `Result<Vec<u8>>`: the encrypted data
    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>>;

    /// Decrypt data
    ///
    /// # Parameters
    /// - `encrypted_data`: &[u8] - the encrypted data
    ///
    /// # Returns
    /// - `Result<Vec<u8>>`: the decrypted data
    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>>;

    /// Get the string representation of the encryption type
    ///
    /// # Returns
    /// - `&'static str`: the string representation of the encryption type
    fn as_string(&self) -> &'static str;

    /// Check if the encrypted data was encrypted using this encryption type
    ///
    /// # Parameters
    /// - `encrypted_data`: &[u8] - the encrypted data
    ///
    /// # Returns
    /// - `bool`: true if the encrypted data was encrypted using this encryption
    ///   type
    fn is_this_type(encrypted_data: &[u8]) -> bool
    where
        Self: Sized;
}

/// Create an encryption type based on the `str` provided to
/// `encryption_type_str` argument
///
/// # Parameters
/// - `key` - the key to use for encryption/decryption for gpg it's the
///   fingerprint of your key
/// - `encryption_type_str` - the encryption type string
///
/// # Returns
/// - `Result<Box<dyn EncryptionType>>`: the encryption type
///
/// # Example
///
/// ```rust
/// use envio::crypto::create_encryption_type;
///
/// let key = "my_key".to_string();
///
/// let encryption_type = create_encryption_type(key, "age").unwrap();
/// ```
pub fn create_encryption_type(
    key: String,
    encryption_type_str: &str,
) -> Result<Box<dyn EncryptionType>> {
    match encryption_type_str {
        "age" => Ok(Box::new(AGE::new(key))),
        "gpg" => Ok(Box::new(GPG::new(key))),
        _ => Err(Error::InvalidEncryptionType(
            encryption_type_str.to_string(),
        )),
    }
}

/// Get the encryption type used to encrypt a profile
///
/// The user first needs to get the encrypted content of the profile and then
/// pass it to this function to get the encryption type used. A
/// [utility](crate::utils::get_profile_content) function is provided to get the
/// encrypted content of a profile.
///
/// # Parameters
/// - `encrypted_content`: &Vec<u8> - the encrypted content of the profile
///
/// # Returns
/// - `Result<Box<dyn EncryptionType>>`: the encryption type
///
/// # Example
///
/// ```rust
/// use envio::utils::get_profile_content;
/// use envio::crypto::get_encryption_type;
///
/// let encrypted_content = get_profile_content("my_profile").unwrap();
///
/// let encryption_type = get_encryption_type(&encrypted_content).unwrap();
///
/// println!("{}", encryption_type.as_string());
/// ```
pub fn get_encryption_type(profile_name: &str) -> Result<Box<dyn EncryptionType>> {
    let encrypted_content = utils::get_profile_content(profile_name)?;

    let e_type =
        if GPG::is_this_type(&encrypted_content) || GPG::is_this_type_fallback(profile_name)? {
            "gpg"
        } else {
            "age"
        };

    create_encryption_type("".to_string(), e_type)
}
