use std::path::{Path, PathBuf};

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{Cipher, CipherKind},
    env::{Env, EnvVec},
    error::{Error, Result},
};

#[derive(Clone, Serialize, Deserialize)]
pub struct ProfileMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub file_path: PathBuf,
    pub cipher_kind: CipherKind,
    pub created_at: DateTime<Local>,
    pub updated_at: DateTime<Local>,
}

pub struct Profile {
    pub metadata: ProfileMetadata,
    pub envs: EnvVec,
    pub cipher: Box<dyn Cipher>,
}

#[derive(Serialize, Deserialize)]
pub struct SerializedProfile {
    pub metadata: ProfileMetadata,
    pub content: Vec<u8>,
}

impl Profile {
    pub fn new(metadata: ProfileMetadata, cipher: Box<dyn Cipher>, envs: EnvVec) -> Profile {
        Profile {
            metadata,
            cipher,
            envs,
        }
    }

    pub fn from_file<P: AsRef<Path>>(file_path: P, cipher: Box<dyn Cipher>) -> Result<Profile> {
        let file_content = std::fs::read(&file_path)?;

        let serialized_profile: SerializedProfile = serde_json::from_slice(&file_content)?;

        let decrypted_envs_bytes = cipher.decrypt(&serialized_profile.content)?;

        let envs: EnvVec = bincode::deserialize(&decrypted_envs_bytes)?;

        Ok(Profile {
            metadata: serialized_profile.metadata,
            envs,
            cipher,
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
        let file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(&self.metadata.file_path)?;

        file.set_len(0)?;

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

        serde_json::to_writer_pretty(&file, &serialized_profile)?;

        Ok(())
    }
}
