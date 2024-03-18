/// A bunch of utility functions that are used in various places in the codebase.
/// These functions are not meant to be used by the end user, but rather to be used by the library itself.
///
/// The CLI also has its own utility functions, but they are located in the `bin/envio` directory inside the `utils.rs` file.
/// There might be a few functions that are used in both the CLI and the library, but they are kept separate since the library does not expose these utility functions to the end user. They are only used internally.
use std::path::PathBuf;

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

pub fn get_configdir() -> PathBuf {
    let homedir = dirs::home_dir().unwrap();
    homedir.join(".envio")
}
