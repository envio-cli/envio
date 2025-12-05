use chacha20poly1305::{
    Key, XChaCha20Poly1305,
    aead::stream::{DecryptorBE32, EncryptorBE32},
    aead::{AeadCore, KeyInit},
};

use argon2::{
    Argon2,
    password_hash::{SaltString, rand_core::OsRng},
};
use base64::{Engine, engine::general_purpose::STANDARD};

use serde::{Deserialize, Serialize};
use std::any::Any;

use crate::{
    cipher::{Cipher, CipherKind},
    error::{Error, Result},
};

const CHUNK_SIZE: usize = 1024; // 1KB

#[derive(Serialize, Deserialize, Default, Clone)]
struct Metadata {
    salt: String,
    nonce: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct PASSPHRASE {
    key: String,
    metadata: Metadata,
}

impl PASSPHRASE {
    pub fn new(key: String) -> Self {
        PASSPHRASE {
            key,
            metadata: Metadata::default(),
        }
    }

    pub fn set_key(&mut self, key: String) {
        self.key = key;
    }

    pub fn get_key(&self) -> String {
        self.key.clone()
    }
}

impl Cipher for PASSPHRASE {
    fn kind(&self) -> CipherKind {
        CipherKind::PASSPHRASE
    }

    fn encrypt(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        let salt = SaltString::generate(&mut OsRng);
        let mut output_key_material = [0u8; 32];

        Argon2::default()
            .hash_password_into(
                self.key.as_bytes(),
                salt.as_str().as_bytes(),
                &mut output_key_material,
            )
            .map_err(|e| Error::Cipher(e.to_string()))?;

        // the stream encryptor expects a 19-byte base nonce (cipher nonce minus 5 bytes for the counter + flag).
        // see: https://docs.rs/aead/0.5.2/aead/stream/struct.StreamBE32.html
        let nonce_bytes = &XChaCha20Poly1305::generate_nonce(&mut OsRng)[0..19];
        let mut encryptor = EncryptorBE32::<XChaCha20Poly1305>::from_aead(
            XChaCha20Poly1305::new(Key::from_slice(&output_key_material)),
            nonce_bytes.into(),
        );

        let mut encrypted_buffer = Vec::new();
        let mut offset = 0;

        while offset + CHUNK_SIZE < data.len() {
            let end = usize::min(offset + CHUNK_SIZE, data.len());
            let chunk = &data[offset..end];

            encrypted_buffer.extend(
                encryptor
                    .encrypt_next(chunk)
                    .map_err(|e| Error::Cipher(e.to_string()))?,
            );

            offset = end;
        }

        let last_chunk = &data[offset..];
        encrypted_buffer.extend(
            encryptor
                .encrypt_last(last_chunk)
                .map_err(|e| Error::Cipher(e.to_string()))?,
        );

        self.metadata.salt = salt.to_string(); // already encoded in base64
        self.metadata.nonce = STANDARD.encode(nonce_bytes);

        Ok(encrypted_buffer)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let mut output_key_material = [0u8; 32];

        Argon2::default()
            .hash_password_into(
                self.key.as_bytes(),
                self.metadata.salt.as_bytes(),
                &mut output_key_material,
            )
            .map_err(|e| Error::Cipher(e.to_string()))?;

        let nonce_bytes = STANDARD
            .decode(&self.metadata.nonce)
            .map_err(|e| Error::Cipher(e.to_string()))?;

        let cipher = XChaCha20Poly1305::new(Key::from_slice(&output_key_material));

        let mut decryptor =
            DecryptorBE32::<XChaCha20Poly1305>::from_aead(cipher, nonce_bytes.as_slice().into());

        let mut decrypted_buffer = Vec::new();
        let mut offset = 0;

        const BUFFER_LEN: usize = CHUNK_SIZE + 16; // 16 bytes for the tag
        while offset + BUFFER_LEN < encrypted_data.len() {
            let end = usize::min(offset + BUFFER_LEN, encrypted_data.len());
            let chunk = &encrypted_data[offset..end];

            decrypted_buffer.extend(
                decryptor
                    .decrypt_next(chunk)
                    .map_err(|e| Error::Cipher(e.to_string()))?,
            );

            offset = end;
        }

        let last_chunk = &encrypted_data[offset..];
        decrypted_buffer.extend(
            decryptor
                .decrypt_last(last_chunk)
                .map_err(|e| Error::Cipher(e.to_string()))?,
        );

        Ok(decrypted_buffer)
    }

    fn export_metadata(&self) -> Option<serde_json::Value> {
        serde_json::to_value(self.metadata.clone()).ok()
    }

    fn import_metadata(&mut self, data: serde_json::Value) -> Result<()> {
        self.metadata = serde_json::from_value(data)?;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
