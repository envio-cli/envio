use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};

use colored::Colorize;

use crate::utils::get_configdir;

use crate::crypto::EncryptionType;

pub struct Profile {
    pub name: String,
    pub envs: HashMap<String, String>,
    pub profile_file_path: PathBuf,
    encryption_type: Box<dyn EncryptionType>,
}

impl Profile {
    pub fn load(name: &str, mut encryption_type: Box<dyn EncryptionType>) -> Option<Profile> {
        let profile_file_path = match Path::new(name).exists() {
            true => PathBuf::from(name),
            false => {
                if !Profile::does_exist(&name) {
                    println!("{}: Profile does not exist", "Error".red());
                    return None;
                }

                get_configdir()
                    .join("profiles")
                    .join(format!("{}.env", name))
            }
        };

        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .open(&profile_file_path)
            .unwrap();

        let mut encrypted_contents = Vec::new();
        file.read_to_end(&mut encrypted_contents).unwrap();

        let content = match encryption_type.decrypt(&encrypted_contents) {
            Ok(c) => c,
            Err(e) => {
                println!("{}: {}", "Error".red(), e);
                return None;
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

        Some(Profile {
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
    pub fn edit_env(&mut self, env: String, new_value: String) {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.envs.entry(env.clone()) {
            e.insert(new_value);
        } else {
            println!("{}: env `{}` does not exists", "Error".red(), env);
        }
    }

    /*
    * Remove an existing environment variable of the profile
    * If the environment variable does not exists, it will print an error message

    @param env &str
    */
    pub fn remove_env(&mut self, env: &str) {
        if self.envs.contains_key(env) {
            self.envs.remove(env);
        } else {
            println!("{}: env `{}` does not exists", "Error".red(), env);
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
    pub fn push_changes(&mut self) {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .open(&self.profile_file_path)
            .unwrap();

        if let Err(e) = file.set_len(0) {
            println!("{}: {}", "Error".red(), e);
        }

        let mut buffer = String::from("");

        for key in self.envs.keys() {
            buffer = buffer + key + "=" + self.envs.get(key).unwrap() + "\n";
        }

        buffer = buffer + &self.encryption_type.get_key();

        let encrypted_data = self.encryption_type.encrypt(&buffer);
        if let Err(e) = file.write_all(encrypted_data.as_slice()) {
            println!("{}: {}", "Error".red(), e);
        }

        if let Err(e) = file.flush() {
            println!("{}: {}", "Error".red(), e);
        }

        if let Err(e) = file.sync_all() {
            println!("{}: {}", "Error".red(), e);
        }
    }
}
