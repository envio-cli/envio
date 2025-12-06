pub mod cipher;
pub mod env;
pub mod error;
pub mod profile;
pub mod utils;

use std::path::Path;
use zeroize::Zeroizing;

pub use env::{Env, EnvMap};
pub use profile::{Profile, ProfileMetadata};

use crate::{
    cipher::{AGE, CipherKind, PASSPHRASE, get_profile_cipher},
    error::{Error, Result},
};

pub fn get_profile<P, F>(file_path: P, key_provider: Option<F>) -> Result<Profile>
where
    P: AsRef<Path>,
    F: FnOnce() -> Zeroizing<String>,
{
    let mut cipher = get_profile_cipher(&file_path)?;

    if matches!(cipher.kind(), CipherKind::PASSPHRASE | CipherKind::AGE) {
        let key = key_provider.ok_or_else(|| {
            Error::Msg(
                "Key provider is required for profiles using passphrase or AGE encryption".into(),
            )
        })?;

        match cipher.kind() {
            CipherKind::PASSPHRASE => cipher
                .as_any_mut()
                .downcast_mut::<PASSPHRASE>()
                .expect("Failed to cast to PASSPHRASE")
                .set_key(key()),
            CipherKind::AGE => cipher
                .as_any_mut()
                .downcast_mut::<AGE>()
                .expect("Failed to cast to AGE")
                .set_key(key()),
            _ => {}
        }
    }

    Profile::from_file(file_path, cipher)
}

pub fn load_profile<P, F>(file_path: P, key_provider: Option<F>) -> Result<Profile>
where
    P: AsRef<Path>,
    F: FnOnce() -> Zeroizing<String>,
{
    let profile = get_profile(file_path, key_provider)?;

    for env in &profile.envs {
        unsafe { std::env::set_var(&env.key, &env.value) };
    }

    Ok(profile)
}
