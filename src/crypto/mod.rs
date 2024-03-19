mod age;
pub mod gpg;

// Re-export the encryption types so that users don't have to use envio::crypto::type::TYPE
pub use age::AGE;
pub use gpg::GPG;

use std::io::Read;
use std::path::Path;

use crate::error::{Error, Result};
use crate::utils::{contains_path_separator, get_configdir};

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
pub fn get_encryption_type(
    name: &str,
    get_key: Option<impl Fn() -> String>,
) -> Result<Box<dyn EncryptionType>> {
    let profile_file_path = if contains_path_separator(name) {
        Path::new(name).to_path_buf()
    } else {
        let config_dir = get_configdir();
        let profile_dir = config_dir.join("profiles");
        profile_dir.join(name.to_owned() + ".env")
    };

    let mut file = std::fs::File::open(profile_file_path)?;

    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).unwrap();

    let gpg_instance = match create_encryption_type("".to_string(), "gpg") {
        Ok(instance) => instance,
        Err(e) => {
            return Err(e);
        }
    };

    // If the file can be decrypted with GPG, then we use GPG, otherwise we use AGE
    if gpg_instance.decrypt(&file_contents).is_ok() {
        Ok(gpg_instance)
    } else {
        if get_key.is_none() {
            return Err(Error::Crypto(
                "No key provided for AGE encryption".to_string(),
            ));
        }

        create_encryption_type(get_key.unwrap()(), "age")
    }
}
