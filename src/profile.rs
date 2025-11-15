use std::path::Path;
use std::{io::Write, path::PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::crypto::EncryptionType;
use crate::env::{Env, EnvVec};
use crate::error::{Error, Result};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub description: Option<String>,
    pub file_path: PathBuf,
    pub encryption_type_name: String,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

pub struct Profile {
    pub metadata: ProfileMetadata,
    pub envs: EnvVec,
    pub encryption_type: Option<Box<dyn EncryptionType>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedProfile {
    pub metadata: ProfileMetadata,
    pub content: Vec<u8>,
}

impl Profile {
    pub fn new(
        metadata: ProfileMetadata,
        encryption_type: Box<dyn EncryptionType>,
        envs: EnvVec,
    ) -> Profile {
        Profile {
            metadata,
            encryption_type: Some(encryption_type),
            envs,
        }
    }

    pub fn from_file<P: AsRef<Path>>(
        file_path: P,
        encryption_type: Box<dyn EncryptionType>,
    ) -> Result<Profile> {
        let file_content = std::fs::read(&file_path)?;

        let serialized_profile: SerializedProfile = serde_json::from_slice(&file_content)
            .map_err(|e| Error::Deserialization(format!("Failed to parse profile JSON: {}", e)))?;

        let decrypted_envs_bytes = encryption_type.decrypt(&serialized_profile.content)?;

        let envs: EnvVec = bincode::deserialize(&decrypted_envs_bytes)
            .map_err(|e| Error::Deserialization(format!("Failed to deserialize envs: {}", e)))?;

        Ok(Profile {
            metadata: serialized_profile.metadata,
            envs,
            encryption_type: Some(encryption_type),
        })
    }

    pub fn insert_env(&mut self, env: String, env_value: String) {
        self.envs.push(Env::from_key_value(env, env_value));
    }

    pub fn edit_env(&mut self, env: String, new_value: String) -> Result<()> {
        if self.envs.iter().any(|e| e.name == env) {
            for e in self.envs.iter_mut() {
                if e.name == env {
                    e.value = new_value;
                    return Ok(());
                }
            }
        }

        Err(Error::EnvDoesNotExist(env))
    }

    pub fn remove_env(&mut self, env: &str) -> Result<()> {
        if self.envs.iter().any(|e| e.name == env) {
            self.envs.retain(|e| e.name != env);
            return Ok(());
        }

        Err(Error::EnvDoesNotExist(env.to_string()))
    }

    pub fn get_env(&self, env: &str) -> Option<&String> {
        for e in self.envs.iter() {
            if e.name == env {
                return Some(&e.value);
            }
        }

        None
    }

    pub fn get_envs_hashmap(&self) -> std::collections::HashMap<String, String> {
        let mut envs = std::collections::HashMap::new();

        for e in self.envs.iter() {
            envs.insert(e.name.clone(), e.value.clone());
        }

        envs
    }

    pub fn save(&mut self) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(&self.metadata.file_path)?;

        file.set_len(0)?;

        let serialized_envs = match bincode::serialize(&self.envs) {
            Ok(data) => data,
            Err(e) => {
                return Err(Error::Serialization(e.to_string()));
            }
        };

        // unwrapping is completely safe since we always pass a encryption type when creating a new profile
        let encrypted_envs = match self
            .encryption_type
            .as_ref()
            .unwrap()
            .encrypt(&serialized_envs)
        {
            Ok(data) => data,
            Err(e) => {
                return Err(e);
            }
        };

        let serialized_profile = serde_json::json!(SerializedProfile {
            metadata: self.metadata.clone(),
            content: encrypted_envs,
        });

        file.write_all(serialized_profile.to_string().as_bytes())?;
        file.flush()?;
        file.sync_all()?;

        Ok(())
    }
}
