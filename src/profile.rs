use std::{collections::HashMap, io::Write, path::PathBuf};

use crate::utils::{get_configdir, get_profile_filepath, truncate_identity_bytes};

use crate::crypto::EncryptionType;
use crate::error::{Error, Result};

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
pub struct Profile {
    pub name: String,
    pub envs: HashMap<String, String>,
    pub profile_file_path: PathBuf,
    encryption_type: Box<dyn EncryptionType>,
}

impl Profile {
    /// Create a new profile object from the encrypted contents of a stored
    /// profile file
    ///  
    ///
    /// This method is not meant to be used by the end user. It is recommended
    /// to use the [load_profile](crate::load_profile) macro to load a profile.
    ///
    /// # Parameters
    /// - `name` - The name of the profile
    /// - `encrypted_content` - The encrypted contents of the profile file
    /// - `encryption_type` - The encryption type used to encrypt the profile
    ///
    /// `name` can either be the name of the profile or the absolute path to the
    /// profile file.
    ///
    /// # Returns
    /// - `Result<Profile>`: the profile object if the operation was successful or an error if it was not
    ///
    /// # Examples
    /// ```
    /// use envio::Profile;
    ///
    /// let encrypted_content = envio::utils::get_profile_content("my-profile").unwrap();
    ///
    /// let mut profile = match Profile::from_content("my-profile", &encrypted_content, envio::crypto::get_encryption_type(&encrypted_content).unwrap()) {
    ///     Ok(p) => p,
    ///     Err(e) => {
    ///         eprintln!("An error occurred: {}", e);
    ///         return;
    ///     }
    ///  };
    /// ```
    pub fn from_content(
        name: &str,
        encrypted_content: &Vec<u8>,
        mut encryption_type: Box<dyn EncryptionType>,
    ) -> Result<Profile> {
        let truncated_content = truncate_identity_bytes(&encrypted_content);

        let content = match encryption_type.decrypt(&truncated_content) {
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

        let profile_file_path = get_profile_filepath(name)?;

        Ok(Profile {
            name: name.to_string(),
            envs,
            profile_file_path,
            encryption_type,
        })
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
        self.envs.insert(env, env_value);
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
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.envs.entry(env.clone()) {
            e.insert(new_value);
            Ok(())
        } else {
            Err(Error::EnvDoesNotExist(env.to_string()))
        }
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
        if self.envs.contains_key(env) {
            self.envs.remove(env);
            Ok(())
        } else {
            return Err(Error::EnvDoesNotExist(env.to_string()));
        }
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
        self.envs.get(env)
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

            let encrypted_content = utils::get_profile_content($name)?;
            let mut encryption_type;

            match crypto::get_encryption_type(&encrypted_content) {
                Ok(t) => encryption_type = t,
                Err(e) => return Err(e.into()),
            }

            if encryption_type.as_string() == "age" {
                $(
                    let key = $get_key();
                    encryption_type.set_key(key);
                )?
            }

            match Profile::from_content($name, &encrypted_content, encryption_type) {
                Ok(profile) => return Ok(profile),
                Err(e) => return Err(e.into()),
            }
         })()
    };
}
