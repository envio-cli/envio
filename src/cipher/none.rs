use serde::{Deserialize, Serialize};
use std::any::Any;

use crate::{
    cipher::{Cipher, CipherKind},
    error::Result,
};

#[derive(Serialize, Deserialize, Clone)]
pub struct NONE;

impl Cipher for NONE {
    fn kind(&self) -> CipherKind {
        CipherKind::NONE
    }

    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        Ok(data.to_vec())
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        Ok(encrypted_data.to_vec())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
