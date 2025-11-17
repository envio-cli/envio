pub mod crypto;
pub mod env;
pub mod error;
pub mod profile;
pub mod utils;

use std::path::Path;

pub use env::{Env, EnvVec};
pub use profile::{Profile, ProfileMetadata};

use crate::{
    crypto::{get_profile_cipher, CipherKind},
    error::{Error, Result},
};

pub fn get_profile<P, F>(file_path: P, get_key: Option<F>) -> Result<Profile>
where
    P: AsRef<Path>,
    F: FnOnce() -> String,
{
    let mut cipher = get_profile_cipher(&file_path)?;

    if cipher.kind() == CipherKind::Age {
        if let Some(key_provider) = get_key {
            cipher.set_key(key_provider());
        } else {
            return Err(Error::Msg(
                "Key provider is required for age-encrypted profiles".to_string(),
            ));
        }
    }

    Profile::from_file(file_path, cipher)
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
