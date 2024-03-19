use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use crate::utils::get_configdir;

use crate::crypto::EncryptionType;
use crate::error::{Error, Result};

pub struct Profile {
    pub name: String,
    pub envs: HashMap<String, String>,
    pub profile_file_path: PathBuf,
    encryption_type: Box<dyn EncryptionType>,
}

impl Profile {
    pub fn load(name: &str, mut encryption_type: Box<dyn EncryptionType>) -> Result<Profile> {
        let profile_file_path = match Path::new(name).exists() {
            true => PathBuf::from(name),
            false => {
                if !Profile::does_exist(&name) {
                    return Err(Error::ProfileDoesNotExist(name.to_string()));
                }

                get_configdir()
                    .join("profiles")
                    .join(format!("{}.env", name))
            }
        };

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(&profile_file_path)?;

        let mut encrypted_contents = Vec::new();
        file.read_to_end(&mut encrypted_contents).unwrap();

        let content = match encryption_type.decrypt(&encrypted_contents) {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };

        let mut envs = HashMap::new();

        for line in content.lines() {
            if line.is_empty() {
                continue;
            }

            if !line.contains('=') {
                encryption_type.set_key(line.to_string());
                continue;
            }

            let mut parts = line.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                envs.insert(key.to_string(), value.to_string());
            }
        }

        Ok(Profile {
            name: name.to_string(),
            envs,
            profile_file_path,
            encryption_type,
        })
    }

    pub fn does_exist(name: &str) -> bool {
        let configdir = get_configdir();

        let profile_path = configdir.join("profiles").join(format!("{}.env", name));

        if profile_path.exists() {
            return true;
        }

        false
    }

    /*
    * Add a new environment variable to the profile

    @param env String
    @param env_value String
    */
    pub fn insert_env(&mut self, env: String, env_value: String) {
        self.envs.insert(env, env_value);
    }

    /*
    * Edit an existing environment variable of the profile
    * If the environment variable does not exists, it will print an error message

    @param env String
    @param new_value String
    */
    pub fn edit_env(&mut self, env: String, new_value: String) -> Result<()> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.envs.entry(env.clone()) {
            e.insert(new_value);
            Ok(())
        } else {
            Err(Error::EnvDoesNotExist(env.to_string()))
        }
    }

    /*
    * Remove an existing environment variable of the profile
    * If the environment variable does not exists, it will print an error message

    @param env &str
    */
    pub fn remove_env(&mut self, env: &str) -> Result<()> {
        if self.envs.contains_key(env) {
            self.envs.remove(env);
            Ok(())
        } else {
            return Err(Error::EnvDoesNotExist(env.to_string()));
        }
    }

    /*
    * Get the value of an environment variable of the profile
    * If the environment variable does not exists, it will return None

    @param env &str
    */
    pub fn get_env(&self, env: &str) -> Option<&String> {
        self.envs.get(env)
    }

    /*
     * Push the changes to the profile file
     */
    pub fn push_changes(&mut self) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .open(&self.profile_file_path)
            .unwrap();

        file.set_len(0)?;

        let mut buffer = String::from("");

        for key in self.envs.keys() {
            buffer = buffer + key + "=" + self.envs.get(key).unwrap() + "\n";
        }

        buffer = buffer + &self.encryption_type.get_key();

        let encrypted_data = match self.encryption_type.encrypt(&buffer) {
            Ok(data) => data,
            Err(e) => {
                return Err(e);
            }
        };

        file.write_all(encrypted_data.as_slice())?;

        file.flush()?;

        file.sync_all()?;

        Ok(())
    }
}
