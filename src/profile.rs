use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::{
    cipher::{Cipher, CipherKind, EncryptedContent},
    env::EnvMap,
    error::Result,
    utils::{get_serialized_profile, save_serialized_profile},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub file_path: PathBuf,
    pub cipher_kind: CipherKind,
    #[serde(skip_serializing_if = "Option::is_none")]
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
    pub content: EncryptedContent,
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
        let serialized_profile = get_serialized_profile(&file_path)?;

        if let Some(cipher_metadata) = &serialized_profile.metadata.cipher_metadata {
            cipher.import_metadata(cipher_metadata.clone())?;
        }

        Ok(Profile {
            metadata: serialized_profile.metadata,
            envs: cipher.decrypt(&serialized_profile.content)?,
            cipher,
        })
    }

    pub fn save(&mut self) -> Result<()> {
        let encrypted_envs = self.cipher.encrypt(&self.envs)?;

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
