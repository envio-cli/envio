/// Utility functions used throughout the binary crate
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use envio::error::{Error, Result};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

/*
* Get the home directory of the user

@return PathBuf
*/
pub fn get_homedir() -> Result<PathBuf> {
    match dirs::home_dir() {
        Some(home) => Ok(home),
        None => Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find home directory",
        ))),
    }
}

/*
* Get the config directory which is located in the home directory
* The directory is called .envio

@return PathBuf
*/
pub fn get_configdir() -> Result<PathBuf> {
    Ok(get_homedir()?.join(".envio"))
}

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

pub fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

/*
* Parse the environment variables from a string
* The string should be in the format of KEY=VALUE

* @param buffer &str
* @return HashMap<String, String>
*/
pub fn parse_envs_from_string(buffer: &str) -> Result<HashMap<String, String>> {
    let mut envs_map = HashMap::new();
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

        envs_map.insert(key.unwrap().to_owned(), value.unwrap().to_owned());
    }

    Ok(envs_map)
}

/*
* Download a file from a url with a progress bar

@param url &str
@param file_name &str
*/
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

// Unix specific code
// Returns the shell that is being used
// @return String
#[cfg(any(target_family = "unix"))]
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
