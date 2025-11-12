use std::collections::HashMap;
use std::{io::Write, path::PathBuf};

use chrono::NaiveDate;
use colored::Colorize;
use inquire::Confirm;
use serde::{Deserialize, Serialize};

use crate::utils::{self, get_configdir, truncate_identity_bytes};

use crate::crypto::EncryptionType;
use crate::error::{Error, Result};

/// Representation of an environment variable
#[derive(Serialize, Deserialize, Clone)]
pub struct Env {
    pub name: String,
    pub value: String,
    pub comment: Option<String>,
    pub expiration_date: Option<NaiveDate>,
}

impl Env {
    pub fn new(
        name: String,
        value: String,
        comment: Option<String>,
        expiration_date: Option<NaiveDate>,
    ) -> Env {
        Env {
            name,
            value,
            comment,
            expiration_date,
        }
    }

    pub fn from_key_value(key: String, value: String) -> Env {
        Env {
            name: key,
            value,
            comment: None,
            expiration_date: None,
        }
    }
}

/// Wrapper around a vector of `Env`
#[derive(Serialize, Deserialize, Clone)]
pub struct EnvVec {
    envs: Vec<Env>,
}

/// Build a `EnvVec` from a `Vec<Env>` or a `HashMap<String, String>`
impl From<Vec<Env>> for EnvVec {
    fn from(envs: Vec<Env>) -> Self {
        EnvVec { envs }
    }
}

impl From<HashMap<String, String>> for EnvVec {
    fn from(envs: HashMap<String, String>) -> Self {
        let mut env_vec = EnvVec::new();

        for (key, value) in envs {
            env_vec.push(Env::from_key_value(key, value));
        }

        env_vec
    }
}

/// Convert a `EnvVec` to a `Vec<Env>` or a `HashMap<String, String>`
impl From<EnvVec> for Vec<Env> {
    fn from(val: EnvVec) -> Self {
        val.envs
    }
}

impl From<EnvVec> for HashMap<String, String> {
    fn from(val: EnvVec) -> Self {
        let mut envs = HashMap::new();

        for e in val.envs {
            envs.insert(e.name, e.value);
        }

        envs
    }
}

impl Default for EnvVec {
    fn default() -> Self {
        Self::new()
    }
}

impl EnvVec {
    pub fn new() -> EnvVec {
        EnvVec { envs: Vec::new() }
    }

    /// Add a new environment variable to the `EnvVec`
    ///
    /// # Parameters
    /// - `env` - The environment variable to add. Has to be an instance of the
    ///   [Env](crate::Env) struct
    ///
    /// # Examples
    /// ```
    /// use envio::EnvVec;
    ///
    /// let mut envs = EnvVec::new();
    ///
    /// envs.push(envio::Env::new("NEW_ENV".to_string(), "NEW_VALUE".to_string()));
    ///
    /// ```
    pub fn push(&mut self, env: Env) {
        self.envs.push(env);
    }

    /// Remove an environment variable from the `EnvVec`
    ///
    /// # Parameters
    /// - `env` - The name of the environment variable to remove. Has to be an
    ///   instance of the [Env](crate::Env) struct
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::EnvVec;
    ///
    /// let mut envs = EnvVec::new();
    ///
    /// envs.push(envio::Env::new("NEW_ENV".to_string(), "NEW_VALUE".to_string()));
    ///
    /// envs.remove("NEW_ENV");
    ///
    /// ```
    pub fn remove(&mut self, env: &str) {
        self.envs.retain(|e| e.name != env);
    }

    /// Return an iterator over the `EnvVec`
    pub fn iter(&self) -> std::slice::Iter<Env> {
        self.envs.iter()
    }

    /// Return a mutable iterator over the `EnvVec`
    pub fn iter_mut(&mut self) -> std::slice::IterMut<Env> {
        self.envs.iter_mut()
    }

    /// Return a vector of all the keys in the `EnvVec`
    ///
    /// # Returns
    /// - `Vec<String>`: A vector of all the keys in the `EnvVec`
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::EnvVec;
    ///
    /// let mut envs = EnvVec::new();
    ///
    /// envs.push(envio::Env::new("NEW_ENV".to_string(), "NEW_VALUE".to_string()));
    ///
    /// let keys = envs.keys();
    ///
    /// for key in keys {
    ///    println!("{}", key);
    /// }
    /// ```
    pub fn keys(&self) -> Vec<String> {
        self.envs.iter().map(|e| e.name.clone()).collect()
    }

    /// Check to see if an environment variable with the given key exists in the
    /// `EnvVec`
    ///
    /// # Parameters
    /// - `key` - The key to check for
    ///
    /// # Returns
    /// - `bool`: indicating whether the key exists or not
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::EnvVec;
    ///
    /// let mut envs = EnvVec::new();
    ///
    /// envs.push(envio::Env::new("NEW_ENV".to_string(), "NEW_VALUE".to_string()));
    ///
    /// let exists = envs.contains_key("NEW_ENV");
    ///
    /// if exists {
    ///   println!("The key exists");
    ///
    /// } else {
    ///  println!("The key does not exist");
    /// }
    /// ```
    pub fn contains_key(&self, key: &str) -> bool {
        self.envs.iter().any(|e| e.name == key)
    }

    /// Get the value of an environment variable with the given key
    ///
    /// # Parameters
    /// - `key` - The key of the environment variable
    ///
    /// # Returns
    /// - `Option<&String>`: The value of the environment variable if it exists
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::EnvVec;
    ///
    /// let mut envs = EnvVec::new();
    ///
    /// envs.push(envio::Env::new("NEW_ENV".to_string(), "NEW_VALUE".to_string()));
    ///
    /// let value = envs.get("NEW_ENV");
    ///
    /// if let Some(v) = value {
    ///     println!("The value of the environment variable is: {}", v);
    /// } else {
    ///     println!("The environment variable does not exist");
    /// }
    /// ```
    pub fn get(&self, key: &str) -> Option<&String> {
        for e in self.envs.iter() {
            if e.name == key {
                return Some(&e.value);
            }
        }

        None
    }

    /// Check to see if the `EnvVec` is empty
    ///
    /// # Returns
    /// - `bool`: indicating whether the `EnvVec` is empty or not
    pub fn is_empty(&self) -> bool {
        self.envs.is_empty()
    }

    /// Get the number of environment variables in the `EnvVec`
    ///
    /// # Returns
    /// - `usize`: The number of environment variables in the `EnvVec`
    pub fn len(&self) -> usize {
        self.envs.len()
    }

    /// Retain only the environment variables that satisfy the given predicate
    ///
    /// # Parameters
    /// - `f` - The predicate to use
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::EnvVec;
    ///
    /// let mut envs = EnvVec::new();
    ///
    /// envs.push(envio::Env::new("NEW_ENV".to_string(), "NEW_VALUE".to_string()));
    /// envs.push(envio::Env::new("NEW_ENV_2".to_string(), "NEW_VALUE_2".to_string()));
    ///
    /// envs.retain(|e| e.name == "NEW_ENV"); // Only keep the environment variable with the key "NEW_ENV"
    /// ```
    pub fn retain<F>(&mut self, f: F)
    where
        F: FnMut(&Env) -> bool,
    {
        self.envs.retain(f);
    }
}

/// Allow users to iterate over the `EnvVec` struct
impl IntoIterator for EnvVec {
    type Item = Env;
    type IntoIter = std::vec::IntoIter<Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.into_iter()
    }
}

impl<'a> IntoIterator for &'a EnvVec {
    type Item = &'a Env;
    type IntoIter = std::slice::Iter<'a, Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.iter()
    }
}

impl<'a> IntoIterator for &'a mut EnvVec {
    type Item = &'a mut Env;
    type IntoIter = std::slice::IterMut<'a, Env>;

    fn into_iter(self) -> Self::IntoIter {
        self.envs.iter_mut()
    }
}

/// This struct is a representation of a profile env file stored on the system.
/// For more information on what a profile is, see the
/// [profile](https://envio-cli.github.io/profiles) page on the official
/// website.
///
/// The way that envio loads a profile is it first reads the encrypted contents
/// of the profile file and then passes that to the
/// [from_content](Profile::from_content) method of the `Profile` struct along
/// with the name of the profile and the encryption type used to encrypt the
/// profile.
///
/// The encryption type can be deduced using the
/// [get_encryption_type](crate::crypto::get_encryption_type) function from the
/// `crypto` module. It is recommended to first take a look at the documentation
/// for the [crypto](crate::crypto) module before proceeding. Since depending on
/// the encryption type used, the user might have to provide a key to decrypt
/// the profile.
///
/// This method is pretty *technical* and is not meant to be used by the end
/// user. It is recommended to use the [load_profile](crate::load_profile) macro to
/// load a profile. It is a lot more user friendly and handles all the technical
/// details for you.
///
/// After successfully loading a profile users can then use the methods of the
/// `Profile` struct to interact with the environment variables stored in the
/// profile.
///
/// After the user has made the changes to the profile, they can then call the
/// [push_changes](Profile::push_changes) method to push the changes to the
/// profile file.
///
/// # Examples
///
/// ```
///   // Error handling is omitted for brevity
///   use envio::Profile;
///    
///   // Read the contents of the profile file
///   // A utility function is provided to do this or you can load the contents yourself
///   let encrypted_content = envio::utils::get_profile_content("my-profile").unwrap();
///
///   // Load the profile assuming the encryption type is `age`
///   let mut profile = Profile::from_content("my-profile", &encrypted_content, envio::crypto::get_encryption_type(&encrypted_content).unwrap()).unwrap();
///   
///   // Or use the load_profile macro
///   let mut profile = envio::load_profile!("my-profile").unwrap();
///
///   for (key, value) in profile.envs.iter() {
///    println!("{}={}", key, value);
///   }
///
///   // Add a new environment variable to the profile
///   profile.insert_env("NEW_ENV".to_string(), "new_value".to_string());
///   
///   // Push the changes to the profile file
///   profile.push_changes().unwrap();
/// ```
#[derive(Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub envs: EnvVec,
    pub profile_file_path: PathBuf,
    encryption_type: Box<dyn EncryptionType>,
}

impl Profile {
    pub fn new(
        name: String,
        envs: EnvVec,
        profile_file_path: PathBuf,
        encryption_type: Box<dyn EncryptionType>,
    ) -> Profile {
        Profile {
            name,
            envs,
            profile_file_path,
            encryption_type,
        }
    }

    /// Create a new profile object from a profile file stored on the system
    ///
    /// This method is not meant to be used by the end user. It is recommended to
    /// use the [load_profile](crate::load_profile) macro to load a profile.
    ///
    /// # Parameters
    /// - `profile_name` - The name of the profile
    /// - `encryption_type` - The encryption type used to encrypt the profile
    ///
    /// `profile_name` can either be the name of the profile or the absolute path
    /// to the profile file.
    ///
    /// # Returns
    /// - `Result<Profile>`: the profile object if the operation was successful or an error if it was not
    ///
    ///   /// # Examples
    /// ```
    /// use envio::Profile;
    ///
    /// let profile_name = "my-profile";
    /// let mut profile = match Profile::from(profile_name, envio::crypto::get_encryption_type(profile_name).unwrap()) {
    ///     Ok(p) => p,
    ///     Err(e) => {
    ///         eprintln!("An error occurred: {}", e);
    ///         return;
    ///     }
    ///  };
    /// ```
    pub fn from(
        profile_name: &str,
        mut encryption_type: Box<dyn EncryptionType>,
    ) -> Result<Profile> {
        let profile_file_path = utils::get_profile_filepath(profile_name)?;
        let encrypted_content = std::fs::read(&profile_file_path)?;

        let truncated_content = truncate_identity_bytes(&encrypted_content);

        let content = match encryption_type.decrypt(&truncated_content) {
            Ok(c) => c,
            Err(e) => {
                return Err(e);
            }
        };

        match bincode::deserialize(&content) {
            Ok(profile) => Ok(profile),
            Err(_) => {
                // Profiles created with older versions of envio are not serialized using bincode
                println!(
                    "{}",
                    format!(
                        "{}: Unable to deserialize the profile content\n\
                    \n\
                    This may indicate:\n\
                     - The file has been tampered with\n\
                     - It was created with an older version of the tool\n",
                        "Warning".yellow().bold()
                    )
                );

                let prompt =
                    Confirm::new("Do you want to fallback to the old way of reading the profile?")
                        .with_default(false)
                        .with_help_message("If the file has been tampered with, then falling back to the old way of reading the profile will not work")
                        .prompt();

                let fallback = match prompt {
                    Ok(f) => f,
                    Err(e) => return Err(Error::Msg(e.to_string())),
                };

                if !fallback {
                    return Err(Error::Deserialization(
                        "Unable to deserialize the profile content".to_string(),
                    ));
                }

                let mut envs = HashMap::new();
                let string_content = String::from_utf8_lossy(&content);
                for line in string_content.lines() {
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

                let mut profile = Profile::new(
                    profile_name.to_owned(),
                    envs.into(),
                    profile_file_path,
                    encryption_type,
                );

                profile.push_changes()?; // Update the profile file with the new format

                println!("{}", "Fallback successful!".green().bold());

                return Ok(profile);
            }
        }
    }

    /// Check to see if a profile with the given name exists on the system
    ///
    /// # Parameters
    /// - `name` - The name of the profile
    ///
    /// # Returns
    /// - `bool`: indicating whether the profile exists or not
    ///
    /// # Examples
    /// ```
    /// use envio::Profile;
    ///
    /// let exists = Profile::does_exist("my-profile");
    ///
    /// if exists {
    ///    println!("The profile exists");
    /// } else {
    ///   println!("The profile does not exist");
    /// }
    pub fn does_exist(name: &str) -> bool {
        let configdir = get_configdir();

        let profile_path = configdir.join("profiles").join(format!("{}.env", name));

        if profile_path.exists() {
            return true;
        }

        false
    }

    /// Add a new environment variable to the profile
    ///
    /// # Parameters
    /// - `env` - The name of the environment variable
    /// - `env_value` - The value of the environment variable
    ///
    /// # Examples
    /// ```
    /// use envio::load_profile;
    ///
    /// let mut profile = load_profile!("my-profile").unwrap();
    ///
    /// profile.insert_env("NEW_ENV".to_string(), "new_value".to_string());
    ///
    /// profile.push_changes().unwrap();
    ///
    /// ```
    pub fn insert_env(&mut self, env: String, env_value: String) {
        self.envs.push(Env::from_key_value(env, env_value));
    }

    /// Edit an existing environment variable of the profile
    ///
    /// # Parameters
    /// - `env` - The name of the environment variable
    /// - `new_value` - The new value of the environment variable
    ///
    /// # Returns
    /// - `Result<()>`: indicating whether the operation was successful or not
    ///
    /// # Examples
    /// ```
    /// use envio::load_profile;
    ///
    /// let mut profile = load_profile!("my-profile").unwrap();
    ///
    /// profile.edit_env("NEW_ENV".to_string(), "new_value".to_string());
    ///
    /// profile.push_changes().unwrap();
    ///
    /// ```
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

    /// Remove an environment variable from the profile
    ///
    /// # Parameters
    /// - `env` - The name of the environment variable
    ///
    /// # Returns
    /// - `Result<()>`: indicating whether the operation was successful or not
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::load_profile;
    ///  
    /// let mut profile = load_profile!("my-profile").unwrap();
    ///
    /// profile.remove_env("NEW_ENV");
    ///
    /// profile.push_changes().unwrap();
    ///
    /// ```
    pub fn remove_env(&mut self, env: &str) -> Result<()> {
        if self.envs.iter().any(|e| e.name == env) {
            self.envs.retain(|e| e.name != env);
            return Ok(());
        }

        Err(Error::EnvDoesNotExist(env.to_string()))
    }

    /// Get the value of an environment variable from the profile
    ///
    /// # Parameters
    /// - `env` - The name of the environment variable
    ///
    /// # Returns
    /// - `Option<&String>`: The value of the environment variable if it exists
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::load_profile;
    ///
    /// let profile = load_profile!("my-profile").unwrap();
    ///
    /// let value = profile.get_env("NEW_ENV");
    ///
    /// if let Some(v) = value {
    ///    println!("The value of the environment variable is: {}", v);
    /// } else {
    ///   println!("The environment variable does not exist");
    /// }
    /// ```
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

    /// Push the changes made to the profile object to the profile file
    ///
    /// # Returns
    /// - `Result<()>`: indicating whether the operation was successful or not
    ///
    /// # Examples
    ///
    /// ```
    /// use envio::load_profile;
    ///
    /// let mut profile = load_profile!("my-profile").unwrap();
    ///
    /// profile.insert_env("NEW_ENV".to_string(), "new_value".to_string());
    ///
    /// profile.push_changes().unwrap();
    ///
    /// ```
    pub fn push_changes(&mut self) -> Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .create(true)
            .open(&self.profile_file_path)?;

        file.set_len(0)?;

        let serialized_data = match bincode::serialize(&self) {
            Ok(data) => data,
            Err(e) => {
                return Err(Error::Serialization(e.to_string()));
            }
        };

        let encrypted_data = match self.encryption_type.encrypt(&serialized_data) {
            Ok(data) => data,
            Err(e) => {
                return Err(e);
            }
        };

        file.write_all(&encrypted_data)?;

        file.flush()?;

        file.sync_all()?;

        Ok(())
    }
}

/// A macro which simplifies the process of loading a profile from the system
///
/// # Parameters
/// - `name` - The name of the profile
/// - `get_key` - A closure which returns the key used to decrypt the profile.
///   It is only required if the profile is encrypted using the `age` encryption
///   type. You can omit this parameter if the profile is encrypted using the
///  `gpg` encryption type. To figure out which encryption type is used, you can
/// use the [get_encryption_type](crate::crypto::get_encryption_type) function
/// from the `crypto` module.
///
/// `name` can either be the name of the profile or the absolute path to the
/// profile file.
///
/// <div class="warning">Please note that it is not recommended to hardcode the key in the closure. It is recommended to use a password manager to store the key and then retrieve it here or prompt the user to enter the key.</div>
///
/// # Returns
/// - `Result<Profile>`: the profile object if the operation was successful or
///   an error if it was not
///
/// # Examples
///
/// If the profile is encrypted using the `gpg` encryption type:
/// ```
/// use envio::load_profile;
///
/// let profile = match load_profile!("my-profile") {
///    Ok(p) => p,
///    Err(e) => {
///     eprintln!("An error occurred: {}", e);
///     return;
///    }
/// };
///
/// for (key, value) in profile.envs.iter() {
///   println!("{}={}", key, value);
/// }
///
/// ```
///
/// If the profile is encrypted using the `age` encryption type:
/// ```
/// use envio::load_profile;
///
/// let profile = match load_profile!("my-profile", || {
///    // This closure should return the key used to decrypt the profile
///    // It is recommended to use a password manager to store the key and then retrieve it here
///    // Or prompt the user to enter the key
///    "my-key".to_string()
/// }) {
///    Ok(p) => p,
///    Err(e) => {
///        eprintln!("An error occurred: {}", e);
///        return;
///    }
/// };
///
/// for (key, value) in profile.envs.iter() {
///   println!("{}={}", key, value);
/// }
///
/// ```
#[macro_export]
macro_rules! load_profile {
    ($name:expr $(, $get_key:expr)?) => {
        (||->envio::error::Result<envio::Profile> {
            use envio::Profile;
            use envio::crypto;
            use envio::utils;

            let mut encryption_type;

            match crypto::get_encryption_type($name) {
                Ok(t) => encryption_type = t,
                Err(e) => return Err(e.into()),
            }

            if encryption_type.as_string() == "age" {
                $(
                    let key = $get_key();
                    encryption_type.set_key(key);
                )?
            } else if encryption_type.as_string() == "none" {
                // No key required for unencrypted profiles
            }

            match Profile::from($name, encryption_type) {
                Ok(profile) => return Ok(profile),
                Err(e) => return Err(e.into()),
            }
         })()
    };
}
