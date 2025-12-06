use std::any::Any;

use crate::{
    EnvMap,
    cipher::{Cipher, CipherKind, EncryptedContent},
    error::{Error, Result},
};

#[derive(Clone)]
pub struct NONE;

impl Cipher for NONE {
    fn kind(&self) -> CipherKind {
        CipherKind::NONE
    }

    fn encrypt(&mut self, envs: &EnvMap) -> Result<EncryptedContent> {
        Ok(EncryptedContent::Json(serde_json::to_value(envs)?))
    }

    fn decrypt(&self, encrypted_data: &EncryptedContent) -> Result<EnvMap> {
        match encrypted_data {
            EncryptedContent::Json(value) => Ok(serde_json::from_value(value.clone())?),
            _ => Err(Error::Cipher(
                "Encrypted data is not a JSON object".to_string(),
            )),
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
