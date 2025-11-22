use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::{
    cipher::{Cipher, CipherKind},
    env::EnvMap,
    error::{Error, Result},
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
    pub fn new(metadata: ProfileMetadata, cipher: Box<dyn Cipher>, envs: EnvMap) -> Profile {
        Profile {
            metadata,
            cipher,
            envs,
        }
    }

    pub fn from_file<P: AsRef<Path>>(file_path: P, mut cipher: Box<dyn Cipher>) -> Result<Profile> {
        let file_content = std::fs::read(&file_path)?;

        let mut serialized_profile: SerializedProfile = serde_json::from_slice(&file_content)?;

        let decrypted_envs_bytes = cipher.decrypt(&serialized_profile.content)?;

        let envs: EnvMap = bincode::deserialize(&decrypted_envs_bytes)?;

        if let Some(cipher_metadata) = &serialized_profile.metadata.cipher_metadata {
            cipher.load_metadata(cipher_metadata.clone())?;
        }

        // ensure file path when saving the profile is correct especially when importing
        serialized_profile.metadata.file_path = file_path.as_ref().into();

        Ok(Profile {
            metadata: serialized_profile.metadata,
            envs,
            cipher,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        let serialized_envs = bincode::serialize(&self.envs)?;

        let encrypted_envs = match self.cipher.encrypt(&serialized_envs) {
            Ok(data) => data,
            Err(e) => {
                return Err(Error::Cipher(e.to_string()));
            }
        };

        self.metadata.updated_at = Local::now();

        let serialized_profile = SerializedProfile {
            metadata: self.metadata.clone(),
            content: encrypted_envs,
        };

        let file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .truncate(true)
            .create(true)
            .open(&self.metadata.file_path)?;

        serde_json::to_writer_pretty(&file, &serialized_profile)?;

        Ok(())
    }
}
