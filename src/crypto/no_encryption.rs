use serde::{Deserialize, Serialize};

use crate::crypto::EncryptionType;
use crate::error::Result;

// Bytes that identify the file as being stored without encryption
pub const IDENTITY_BYTES: &[u8] = b"-----PLAIN TEXT FILE-----";

/// NoEncryption is a pass-through encryption type that stores data in plain text
///
/// **Security Warning**: This option stores environment variables without encryption.
/// Only use this for non-sensitive data or in secure development environments.
#[derive(Serialize, Deserialize)]
pub struct NoEncryption {
    key: String,
}

#[typetag::serde]
impl EncryptionType for NoEncryption {
    fn new(key: String) -> Self {
        NoEncryption { key }
    }

    fn set_key(&mut self, key: String) {
        self.key = key;
    }

    fn get_key(&self) -> String {
        self.key.clone()
    }

    fn as_string(&self) -> &'static str {
        "none"
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Simply append the identity bytes to the data without encryption
        let mut result = data.to_vec();
        result.extend_from_slice(IDENTITY_BYTES);
        Ok(result)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        // Remove the identity bytes from the end if present
        if encrypted_data.len() >= IDENTITY_BYTES.len()
            && &encrypted_data[encrypted_data.len() - IDENTITY_BYTES.len()..] == IDENTITY_BYTES
        {
            Ok(encrypted_data[..encrypted_data.len() - IDENTITY_BYTES.len()].to_vec())
        } else {
            Ok(encrypted_data.to_vec())
        }
    }

    fn is_this_type(encrypted_data: &[u8]) -> bool {
        encrypted_data.len() >= IDENTITY_BYTES.len()
            && &encrypted_data[encrypted_data.len() - IDENTITY_BYTES.len()..] == IDENTITY_BYTES
    }
}
