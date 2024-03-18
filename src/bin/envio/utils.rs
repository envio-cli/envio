/// Utility functions used throughout the binary crate
use std::io::Write;
use std::path::PathBuf;
use std::{collections::HashMap, fs::File};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

/*
* Get the home directory of the user

@return PathBuf
*/
pub fn get_homedir() -> PathBuf {
    match dirs::home_dir() {
        Some(home) => home,
        None => {
            println!("{}: Home directory not found", "Error".red());
            std::process::exit(1);
        }
    }
}

/*
* Get the config directory which is located in the home directory
* The directory is called .envio

@return PathBuf
*/
pub fn get_configdir() -> PathBuf {
    let homedir = get_homedir();

    homedir.join(".envio")
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
pub fn parse_envs_from_string(buffer: &str) -> HashMap<String, String> {
    let mut envs_map = HashMap::new();
    for buf in buffer.lines() {
        if buf.is_empty() || !buf.contains('=') {
            continue;
        }

        let mut split = buf.split('=');

        let key = split.next();
        let mut value = split.next();

        if key.is_none() {
            println!(
                "{}: Can not parse key from buffer: `{}`",
                "Error".red(),
                buf
            );
            std::process::exit(1);
        }

        if value.is_none() {
            value = Some("");
        }

        envs_map.insert(key.unwrap().to_owned(), value.unwrap().to_owned());
    }

    envs_map
}

/*
* Download a file from a url with a progress bar

@param url &str
@param file_name &str
*/
pub async fn download_file(url: &str, file_name: &str) {
    let client = Client::new();
    let mut resp = if let Err(e) = client.get(url).send().await {
        println!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    } else {
        client.get(url).send().await.unwrap()
    };

    let mut file = if let Err(e) = File::create(file_name) {
        println!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    } else {
        File::create(file_name).unwrap()
    };

    let mut content_length = if resp.content_length().is_none() {
        println!("{}: Can not get content length of ", "Error".red());
        std::process::exit(1);
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
        if let Err(e) = file.write_all(&chunk) {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }

        pb.inc(chunk_size as u64);
        content_length -= chunk_size as u64;
    }

    pb.finish();
}
