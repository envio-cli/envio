use std::fs::File;
/// Utility functions used throughout the binary crate
use std::io::Write;
use std::path::PathBuf;

use envio::error::{Error, Result};
use envio::{Env, EnvVec};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

/// Get the home directory
///
/// # Returns
/// - `PathBuf`: the home directory
pub fn get_homedir() -> Result<PathBuf> {
    match dirs::home_dir() {
        Some(home) => Ok(home),
        None => Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        ))),
    }
}

/// Get the config directory
///
/// # Returns
/// - `PathBuf`: the config directory
pub fn get_configdir() -> Result<PathBuf> {
    Ok(get_homedir()?.join(".envio"))
}

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

pub fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

/// Parse environment variables from a string
///
/// # Parameters
/// - `buffer`: &str - the buffer to parse
///
/// # Returns
/// - `Result<HashMap<String, String>>`: the parsed environment variables
pub fn parse_envs_from_string(buffer: &str) -> Result<EnvVec> {
    let mut envs_vec = EnvVec::new();

    for buf in buffer.lines() {
        if buf.is_empty() || !buf.contains('=') {
            continue;
        }

        let mut split = buf.split('=');

        let key = split.next();
        let mut value = split.next();

        if key.is_none() {
            return Err(Error::Msg("Can not parse key from buffer".to_string()));
        }

        if value.is_none() {
            value = Some("");
        }

        envs_vec.push(Env::new(
            key.unwrap().to_string(),
            value.unwrap().to_string(),
        ));
    }

    Ok(envs_vec)
}

/// Download a file from a url with a progress bar
///
/// # Parameters
/// - `url`: &str - the url to download the file from
/// - `file_name`: &str - the name of the file to save the downloaded file to
///
/// # Returns
/// - `Result<()>`: an empty result
pub async fn download_file(url: &str, file_name: &str) -> Result<()> {
    let client = Client::new();
    let mut resp = if let Err(e) = client.get(url).send().await {
        return Err(Error::Msg(e.to_string()));
    } else {
        client.get(url).send().await.unwrap()
    };

    let mut file = File::create(file_name)?;

    let mut content_length = if resp.content_length().is_none() {
        return Err(Error::Msg("Content length is not available".to_string()));
    } else {
        resp.content_length().unwrap()
    };

    let pb = ProgressBar::new(content_length);

    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    while let Some(chunk) = resp.chunk().await.unwrap() {
        let chunk_size = chunk.len();
        file.write_all(&chunk)?;

        pb.inc(chunk_size as u64);
        content_length -= chunk_size as u64;
    }

    pb.finish();
    Ok(())
}

/// Unix specific code
/// Returns the shell that is being used
///
/// # Returns
/// - `Result<&'static str>`: the shell that is being used
#[cfg(target_family = "unix")]
pub fn get_shell_config() -> Result<&'static str> {
    // Gets your default shell
    // This is used to determine which shell config file to edit
    let shell_env_value = if let Ok(e) = std::env::var("SHELL") {
        e
    } else {
        return Err(Error::Msg("Failed to get shell".to_string()));
    };

    let shell_as_vec = shell_env_value.split('/').collect::<Vec<&str>>();
    let shell = shell_as_vec[shell_as_vec.len() - 1];

    let mut shell_config = "";
    if shell.contains("bash") {
        shell_config = ".bashrc";
    } else if shell.contains("zsh") {
        shell_config = ".zshrc";
    } else if shell.contains("fish") {
        shell_config = ".config/fish/config.fish"
    }

    Ok(shell_config)
}
