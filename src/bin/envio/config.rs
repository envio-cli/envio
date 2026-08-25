use std::path::PathBuf;

use envio::profile::{ProfileMetadata, SerializedProfile};

use crate::{
    error::{AppError, AppResult},
    utils::get_cwd,
};

/// Resolve the envio root directory: a local `.envio` in the current directory
/// if present, otherwise a global one from `ENVIO_HOME` (or `~/.envio`).
///
/// This lets a profile created in the home folder be used from any directory,
/// without re-initializing envio in every project. The local store always wins
/// when present, so per-project profiles keep overriding global ones.
/// `envio init` is not affected: it always creates a local `.envio` in the
/// current directory.
pub fn get_envio_dir() -> AppResult<PathBuf> {
    let cwd_envio = get_cwd().join(".envio");
    if cwd_envio.is_dir() {
        return Ok(canonicalize(cwd_envio));
    }

    if let Some(home) = std::env::var_os("ENVIO_HOME").filter(|v| !v.is_empty()) {
        let dir = PathBuf::from(home);
        if !dir.is_dir() {
            return Err(AppError::Msg(format!(
                "ENVIO_HOME directory does not exist: {}",
                dir.display()
            )));
        }
        return Ok(canonicalize(dir));
    }

    if let Some(home_dir) = dirs::home_dir() {
        let dir = home_dir.join(".envio");
        if dir.is_dir() {
            return Ok(canonicalize(dir));
        }
    }

    Err(AppError::Msg(
        "No .envio folder found in the current directory or home folder. \
         Run `envio init` first, or set ENVIO_HOME to your envio root."
            .to_string(),
    ))
}

/// Canonicalize a path, falling back to the original if canonicalization fails.
fn canonicalize(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

/// Print a one-line warning when a global store is being used, so writes
/// outside a project directory don't surprise the user. Call from mutations
/// only (create/set/unset/delete/edit/import/rotate-key), not from reads.
pub fn warn_if_global_store() {
    let Ok(dir) = get_envio_dir() else {
        return;
    };
    let local = get_cwd().join(".envio");
    if dir != canonicalize(local) {
        eprintln!("warning: using global envio store at {}", dir.display());
    }
}

pub fn get_profile_dir() -> AppResult<PathBuf> {
    Ok(get_envio_dir()?.join("profiles"))
}

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

/// returns the path for a profile that does **not** exist yet
pub fn build_profile_path(profile_name: &str) -> AppResult<PathBuf> {
    Ok(get_profile_dir()?.join(format!("{profile_name}.envio")))
}

/// returns the path for a profile that **must exist**
pub fn get_profile_path(profile_name: &str) -> AppResult<PathBuf> {
    let path = build_profile_path(profile_name)?;

    if !path.exists() {
        return Err(AppError::ProfileDoesNotExist(profile_name.to_string()));
    }

    Ok(path)
}

pub fn get_profile_metadata(profile_name: &str) -> AppResult<ProfileMetadata> {
    let path = get_profile_path(profile_name)?;
    let serialized_profile: SerializedProfile = envio::utils::get_serialized_profile(path)?;
    Ok(serialized_profile.metadata)
}

pub fn collect_profile_names() -> AppResult<Vec<String>> {
    let profile_dir = get_profile_dir()?;
    let mut profiles = Vec::new();

    if !profile_dir.exists() {
        return Ok(profiles);
    }

    for entry in std::fs::read_dir(&profile_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("envio") {
            continue;
        }

        if let Some(name) = path.file_stem().and_then(|s| s.to_str())
            && !name.starts_with('.')
        {
            profiles.push(name.to_owned());
        }
    }

    Ok(profiles)
}
