pub mod age;
pub mod gpg;

// Re-export the encryption types so that users don't have to use envio::crypto::type::TYPE
pub use age::AGE;
pub use gpg::GPG;

use crate::error::{Error, Result};

/*
 * EncryptionType trait
 * This trait is used to implement different encryption methods
 * Note: AGE is not a real encryption method, but a wrapper around the age crate to use the same interface as the other encryption methods
*/
pub trait EncryptionType {
    fn new(key: String) -> Self
    where
        Self: Sized;
    /*
    * Set the key to use for encryption/decryption

    * @param key: key to use - for GPG it's the fingerprint of the key
    */
    fn set_key(&mut self, key: String);
    /*
    * Get the key used for encryption/decryption

    * @return String
    */
    fn get_key(&self) -> String;
    /*
    * Encrypt data

    * @param data: data to encrypt
    * @return encrypted data
    */
    fn encrypt(&self, data: &str) -> Result<Vec<u8>>;
    /*
     * Decrypt data
     */
    fn decrypt(&self, encrypted_data: &[u8]) -> Result<String>;
    /*
     * Return the name of the encryption type
     */
    fn as_string(&self) -> &'static str;

    fn is_this_type(encrypted_data: &[u8]) -> bool
    where
        Self: Sized;
}

/*
 * Create an encryption type based on the string passed

 * @param key: String - the key to use for encryption/decryption (for GPG this is the fingerprint)
 * @param encryption_type_str: &str - the string to match against
 * @return Box<dyn EncryptionType>: the encryption type
*/
pub fn create_encryption_type(
    key: String,
    encryption_type_str: &str,
) -> Result<Box<dyn EncryptionType>> {
    match encryption_type_str {
        "age" => Ok(Box::new(AGE::new(key))),
        "gpg" => Ok(Box::new(GPG::new(key))),
        _ => {
            return Err(Error::InvalidEncryptionType(
                encryption_type_str.to_string(),
            ));
        }
    }
}

/*
 * Get the encryption type for a profile
 * This is used to get the encryption type for a profile when decrypting a file, so we know which
 * encryption type to use

 * @param name: String - the name of the profile or absolute path to the profile file
 * @param get_key: FnOnce() -> String - a function that returns the key to use for encryption/decryption this is only used for AGE encryption
 * @return Box<dyn EncryptionType>: the encryption type
*/
pub fn get_encryption_type(encrypted_content: &Vec<u8>) -> Result<Box<dyn EncryptionType>> {
    let e_type;
    if GPG::is_this_type(&encrypted_content) {
        e_type = "gpg";
    } else {
        e_type = "age";
    }

    create_encryption_type("".to_string(), e_type)
}
