use aead::stream::{DecryptorBE32, EncryptorBE32};
use aes_gcm::{
    aead::{AeadCore, KeyInit},
    Aes256Gcm, Key, Nonce,
};
use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::{Deserialize, Serialize};
use std::any::Any;

use crate::{
    cipher::{Cipher, CipherKind},
    error::{Error, Result},
};

const CHUNK_SIZE: usize = 1024;

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
        if data.len() < CHUNK_SIZE {
            return Err(Error::Cipher("Data is too small".to_string()));
        }

        let salt = SaltString::generate(&mut OsRng);
        let mut output_key_material = [0u8; 32];

        Argon2::default()
            .hash_password_into(
                self.key.as_bytes(),
                salt.as_str().as_bytes(),
                &mut output_key_material,
            )
            .map_err(|e| Error::Cipher(e.to_string()))?;

        // the stream encryptor expects a 7-byte base nonce (cipher nonce minus 5 bytes for the counter + flag).
        // see: https://docs.rs/aead/0.5.2/aead/stream/struct.StreamBE32.html
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng)[0..7].to_vec();
        let nonce = Nonce::from_slice(&nonce);
        let mut encryptor = EncryptorBE32::<Aes256Gcm>::from_aead(
            Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&output_key_material)),
            nonce,
        );

        let mut encrypted_buffer = Vec::new();
        let mut chunks = data.chunks(CHUNK_SIZE);

        if let Some(mut last_chunk) = chunks.next() {
            for chunk in chunks.by_ref() {
                encryptor
                    .encrypt_next_in_place(last_chunk, &mut encrypted_buffer)
                    .map_err(|e| Error::Cipher(e.to_string()))?;

                last_chunk = chunk;
            }

            encryptor
                .encrypt_last_in_place(last_chunk, &mut encrypted_buffer)
                .map_err(|e| Error::Cipher(e.to_string()))?;
        }

        self.metadata.salt = salt.to_string(); // already encoded in base64
        self.metadata.nonce = STANDARD.encode(nonce.as_slice());

        Ok(encrypted_buffer)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        let mut output_key_material = [0u8; 32];

        Argon2::default()
            .hash_password_into(
                self.key.as_bytes(),
                SaltString::from_b64(&self.metadata.salt)
                    .map_err(|e| Error::Cipher(e.to_string()))?
                    .as_str()
                    .as_bytes(),
                &mut output_key_material,
            )
            .map_err(|e| Error::Cipher(e.to_string()))?;

        let nonce_bytes = STANDARD
            .decode(&self.metadata.nonce)
            .map_err(|e| Error::Cipher(e.to_string()))?;

        let cipher = Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&output_key_material));

        let mut decryptor =
            DecryptorBE32::<Aes256Gcm>::from_aead(cipher, Nonce::from_slice(&nonce_bytes));

        let mut decrypted_buffer = Vec::new();
        let mut chunks = encrypted_data.chunks(CHUNK_SIZE);

        if let Some(mut last_chunk) = chunks.next() {
            for chunk in chunks.by_ref() {
                decryptor
                    .decrypt_next_in_place(last_chunk, &mut decrypted_buffer)
                    .map_err(|e| Error::Cipher(e.to_string()))?;

                last_chunk = chunk;
            }

            decryptor
                .decrypt_last_in_place(last_chunk, &mut decrypted_buffer)
                .map_err(|e| Error::Cipher(e.to_string()))?;
        }

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
