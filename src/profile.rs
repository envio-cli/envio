use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::{
    cipher::{Cipher, CipherKind},
    env::EnvMap,
    error::Result,
    utils::save_serialized_profile,
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub file_path: PathBuf,
    pub cipher_kind: CipherKind,
    pub cipher_metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

#[derive(Clone)]
pub struct Profile {
    pub metadata: ProfileMetadata,
    pub envs: EnvMap,
    pub cipher: Box<dyn Cipher>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedProfile {
    pub metadata: ProfileMetadata,
    pub content: Vec<u8>,
}

impl Profile {
    pub fn new(
        name: String,
        description: Option<String>,
        file_path: PathBuf,
        envs: EnvMap,
        cipher: Box<dyn Cipher>,
    ) -> Profile {
        Profile {
            metadata: ProfileMetadata {
                name,
                version: env!("CARGO_PKG_VERSION").to_string(),
                description,
                file_path,
                cipher_kind: cipher.kind(),
                cipher_metadata: cipher.export_metadata(),
                created_at: Local::now(),
                updated_at: Local::now(),
            },
            envs,
            cipher,
        }
    }

    pub fn from_file<P: AsRef<Path>>(file_path: P, mut cipher: Box<dyn Cipher>) -> Result<Profile> {
        let file_content = std::fs::read(&file_path)?;

        let serialized_profile: SerializedProfile = serde_json::from_slice(&file_content)?;

        if let Some(cipher_metadata) = &serialized_profile.metadata.cipher_metadata {
            cipher.import_metadata(cipher_metadata.clone())?;
        }

        let decrypted_envs_bytes = cipher.decrypt(&serialized_profile.content)?;

        let (envs, _): (EnvMap, usize) =
            bincode::serde::decode_from_slice(&decrypted_envs_bytes, bincode::config::standard())?;

        Ok(Profile {
            metadata: serialized_profile.metadata,
            envs,
            cipher,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        let serialized_envs =
            bincode::serde::encode_to_vec(&self.envs, bincode::config::standard())?;

        let encrypted_envs = self.cipher.encrypt(&serialized_envs)?;

        self.metadata.updated_at = Local::now();
        self.metadata.cipher_metadata = self.cipher.export_metadata();

        let serialized_profile = SerializedProfile {
            metadata: self.metadata.clone(),
            content: encrypted_envs,
        };

        save_serialized_profile(&self.metadata.file_path, serialized_profile)?;

        Ok(())
    }
}
