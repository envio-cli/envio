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
use zeroize::Zeroize;

use crate::{
    error::{Error, Result},
    metadata_struct,
};

pub const CHUNK_SIZE: usize = 1024;

metadata_struct!(V1, {
    salt: String,
    nonce: String,
});

pub fn encrypt(key: &str, data: &[u8]) -> Result<(Vec<u8>, MetadataV1)> {
    let salt = SaltString::generate(&mut OsRng);
    let mut output_key_material = [0u8; 32];

    Argon2::default()
        .hash_password_into(
            key.as_bytes(),
            salt.as_str().as_bytes(),
            &mut output_key_material,
        )
        .map_err(|e| Error::Cipher(e.to_string()))?;

    let nonce_bytes = &XChaCha20Poly1305::generate_nonce(&mut OsRng)[0..19];
    let mut encryptor = EncryptorBE32::<XChaCha20Poly1305>::from_aead(
        XChaCha20Poly1305::new(Key::from_slice(&output_key_material)),
        nonce_bytes.into(),
    );
    
    output_key_material.zeroize();

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

    let metadata = MetadataV1 {
        salt: salt.to_string(),
        nonce: STANDARD.encode(nonce_bytes),
    };

    Ok((encrypted_buffer, metadata))
}

pub fn decrypt(key: &str, metadata: &MetadataV1, encrypted_data: &[u8]) -> Result<Vec<u8>> {
    let mut output_key_material = [0u8; 32];

    Argon2::default()
        .hash_password_into(
            key.as_bytes(),
            metadata.salt.as_bytes(),
            &mut output_key_material,
        )
        .map_err(|e| Error::Cipher(e.to_string()))?;

    let nonce_bytes = STANDARD
        .decode(&metadata.nonce)
        .map_err(|e| Error::Cipher(e.to_string()))?;

    let cipher = XChaCha20Poly1305::new(Key::from_slice(&output_key_material));
    output_key_material.zeroize();

    let mut decryptor =
        DecryptorBE32::<XChaCha20Poly1305>::from_aead(cipher, nonce_bytes.as_slice().into());

    let mut decrypted_buffer = Vec::new();
    let mut offset = 0;

    const BUFFER_LEN: usize = CHUNK_SIZE + 16;
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
