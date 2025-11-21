pub mod cipher;
pub mod env;
pub mod error;
pub mod profile;
pub mod utils;

use std::path::Path;

pub use env::{Env, EnvMap};
pub use profile::{Profile, ProfileMetadata};

use crate::{
    cipher::{get_profile_cipher, CipherKind, PASSPHRASE},
    error::{Error, Result},
};

pub fn get_profile<P, F>(file_path: P, key_provider: Option<F>) -> Result<Profile>
where
    P: AsRef<Path>,
    F: FnOnce() -> String,
{
    let mut cipher = get_profile_cipher(&file_path)?;

    match cipher.kind() {
        CipherKind::PASSPHRASE => {
            let key = key_provider.ok_or_else(|| {
                Error::Msg("Key provider is required for passphrase-encrypted profiles".into())
            })?;

            cipher
                .as_any_mut()
                .downcast_mut::<PASSPHRASE>()
                .unwrap()
                .set_key(key());
        }
        _ => {}
    }

    Profile::from_file(file_path, cipher)
}

pub fn load_profile<P, F>(file_path: P, key_provider: Option<F>) -> Result<Profile>
where
    P: AsRef<Path>,
    F: FnOnce() -> String,
{
    let profile = get_profile(file_path, key_provider)?;

    for env in &profile.envs {
        std::env::set_var(&env.name, &env.value);
    }

    Ok(profile)
}
