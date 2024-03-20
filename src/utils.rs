/// A bunch of utility functions that are used in various places in the codebase.
/// These functions are not meant to be used by the end user, but rather to be used by the library itself.
///
/// The CLI also has its own utility functions, but they are located in the `bin/envio` directory inside the `utils.rs` file.
/// There might be a few functions that are used in both the CLI and the library, but they are kept separate since the library does not expose these utility functions to the end user. They are only used internally.
use std::{
    io::Read,
    path::{Path, PathBuf},
};

use crate::{
    error::{Error, Result},
    Profile,
};

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

pub fn get_profile_filepath(name: &str) -> Result<PathBuf> {
    match Path::new(name).exists() {
        true => return Ok(PathBuf::from(name)),
        false => {
            if !Profile::does_exist(&name) {
                return Err(Error::ProfileDoesNotExist(name.to_string()));
            }

            return Ok(get_configdir()
                .join("profiles")
                .join(format!("{}.env", name)));
        }
    };
}

/// Reads the encrypted content of a profile and returns it
/// 
/// # Parameters
/// - `name`: &str - the name of the profile
/// 
/// `name` can either be the name of the profile or the absolute path to the profile file.
/// 
/// # Returns
/// - `Result<Vec<u8>>`: the encrypted content of the profile
pub fn get_profile_content(name: &str) -> Result<Vec<u8>> {
    let profile_file_path = get_profile_filepath(name)?;

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&profile_file_path)?;

    let mut encrypted_contents = Vec::new();
    file.read_to_end(&mut encrypted_contents).unwrap();

    Ok(encrypted_contents)
}

pub fn get_configdir() -> PathBuf {
    let homedir = dirs::home_dir().unwrap();
    homedir.join(".envio")
}

pub fn truncate_identity_bytes(encrypted_contents: &Vec<u8>) -> Vec<u8> {
    let mut truncated_contents = encrypted_contents.clone();

    truncated_contents.truncate(encrypted_contents.len() - 28);

    truncated_contents
}
