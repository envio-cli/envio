pub mod crypto;
pub mod env;
pub mod error;
pub mod profile;
pub mod utils;

pub use env::{Env, EnvVec};
pub use profile::{Profile, ProfileMetadata};

use std::path::Path;
use crate::crypto::{get_encryption_type, ENCRYPTION_TYPE_AGE};
use crate::error::{Error, Result};


pub fn get_profile<P, F>(file_path: P, get_key: Option<F>) -> Result<Profile>
where
    P: AsRef<Path>,
    F: FnOnce() -> String,
{
    let mut encryption_type = get_encryption_type(&file_path)?;

    if encryption_type.as_string() == ENCRYPTION_TYPE_AGE {
        if let Some(key_provider) = get_key {
            encryption_type.set_key(key_provider());
        } else {
            return Err(Error::Msg(
                "Key provider is required for age-encrypted profiles".to_string(),
            ));
        }
    }

    Profile::from_file(file_path, encryption_type)
}

pub fn load_profile<P, F>(file_path: P, get_key: Option<F>) -> Result<Profile>
where
    P: AsRef<Path>,
    F: FnOnce() -> String,
{
    let profile = get_profile(file_path, get_key)?;

    for env in &profile.envs {
        std::env::set_var(&env.name, &env.value);
    }

    Ok(profile)
}
