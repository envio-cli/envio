use std::{fs::File, io::Write, path::PathBuf};

use envio::{
    profile::{ProfileMetadata, SerializedProfile},
    Env, EnvMap,
};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;

use crate::error::{AppError, AppResult};

pub fn get_configdir() -> PathBuf {
    dirs::home_dir().unwrap().join(".envio")
}

pub fn get_profile_dir() -> PathBuf {
    get_configdir().join("profiles")
}

#[cfg(target_family = "unix")]
pub fn get_shellscript_path() -> PathBuf {
    get_configdir().join("setenv.sh")
}

pub fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

/// returns the path for a profile that does **not** exist yet
pub fn build_profile_path(profile_name: &str) -> PathBuf {
    get_profile_dir().join(format!("{}.env", profile_name))
}

/// returns the path for a profile that **must exist**
pub fn get_profile_path(profile_name: &str) -> AppResult<PathBuf> {
    let path = build_profile_path(profile_name);
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

pub fn parse_envs_from_string(buffer: &str) -> AppResult<EnvMap> {
    let mut envs_vec = EnvMap::new();

    for line in buffer.lines() {
        if line.is_empty() || !line.contains('=') {
            continue;
        }

        let mut split = line.splitn(2, '=');
        let key = split
            .next()
            .ok_or_else(|| AppError::Msg("Cannot parse key from buffer".to_string()))?;
        let value = split
            .next()
            .ok_or_else(|| AppError::Msg(format!("Cannot parse value for key: `{}`", key)))?;

        envs_vec.insert(Env::from_key_value(key.to_string(), value.to_string()));
    }

    Ok(envs_vec)
}

pub async fn download_file(url: &str, file_name: &str) -> AppResult<()> {
    let client = Client::new();

    let pb = ProgressBar::new_spinner();
    pb.set_message("Connecting...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{msg} {spinner:.green} [{elapsed_precise}]")
            .unwrap(),
    );

    let mut resp = client.get(url).send().await?;
    let mut file = File::create(file_name)?;

    let content_length = resp
        .content_length()
        .ok_or_else(|| AppError::Msg("Content length not available".to_string()))?;
    pb.set_length(content_length);

    let style = ProgressStyle::default_bar()
        .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .with_key(
            "eta",
            |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap();
            },
        )
        .progress_chars("#>-");
    pb.set_style(style);

    let mut downloaded = 0u64;
    while let Some(chunk) = resp.chunk().await? {
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish();
    Ok(())
}

#[cfg(target_family = "unix")]
pub fn get_shell_config_path(full_path: bool) -> AppResult<PathBuf> {
    let shell_env_value = std::env::var("SHELL")
        .map_err(|_| AppError::Msg("Failed to get SHELL environment variable".into()))?;

    let shell = shell_env_value.rsplit('/').next().unwrap_or("");

    let shell_config_path = if shell.contains("bash") {
        ".bashrc"
    } else if shell.contains("zsh") {
        ".zshrc"
    } else if shell.contains("fish") {
        ".config/fish/config.fish"
    } else {
        return Err(AppError::UnsupportedShell(shell.to_string()));
    };

    if full_path {
        Ok(dirs::home_dir().unwrap().join(shell_config_path))
    } else {
        Ok(PathBuf::from(shell_config_path))
    }
}
