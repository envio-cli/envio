use std::{fs::File, io::Write, path::PathBuf};

use envio::{
    error::{Error, Result},
    Env, EnvVec,
};
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;

#[cfg(target_family = "unix")]
pub fn initalize_config() -> Result<()> {
    use std::path::Path;

    use colored::Colorize;
    use inquire::Text;

    let configdir = get_configdir();
    let homedir = dirs::home_dir().unwrap();

    if !Path::new(&configdir).exists() {
        println!("{}", "Creating config directory".bold());

        std::fs::create_dir(&configdir)?;
        std::fs::create_dir(configdir.join("profiles"))?;
    }

    if !Path::new(&configdir.join("setenv.sh")).exists() {
        println!("{}", "Creating shellscript".bold());
        std::fs::write(configdir.join("setenv.sh"), "")?;

        let shellconfig = get_shell_config()?;

        let mut file_path =
            PathBuf::from(&(homedir.to_str().unwrap().to_owned() + &format!("/{}", shellconfig)));
        if !file_path.exists() {
            let input = Text::new(
                "Shell config file not found, please enter the path to your shell config file:",
            )
            .prompt();

            file_path = if let Ok(val) = input {
                PathBuf::from(val)
            } else {
                return Err(Error::Msg(
                    "Failed to get shell config file path".to_string(),
                ));
            };

            if !file_path.exists() {
                return Err(Error::Msg(
                    "Specified shell config file does not exist".to_string(),
                ));
            }
        }

        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(file_path)
            .unwrap();

        let shellscript_path = &configdir.join("setenv.sh");

        let buffer = if shellconfig.contains("fish") {
            println!(
                    "To use the shellscript properly you need to install the {}(https://github.com/edc/bass) plugin for fish",
                    "bass".bold()
                );
            format!(
                "
# envio DO NOT MODIFY
bass source {}
",
                shellscript_path.to_str().unwrap()
            )
        } else {
            format!(
                "
#envio DO NOT MODIFY
source {}
",
                shellscript_path.to_str().unwrap()
            )
        };

        writeln!(file, "{}", buffer)?
    }

    Ok(())
}

pub fn get_configdir() -> PathBuf {
    let homedir = dirs::home_dir().unwrap();

    homedir.join(".envio")
}

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

pub fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

pub fn does_profile_exist(profile_name: &str) -> bool {
    let profile_path = get_profile_path(profile_name);

    if profile_path.exists() {
        return true;
    }

    false
}

pub fn get_profile_path(profile_name: &str) -> PathBuf {
    get_configdir()
        .join("profiles")
        .join(format!("{}.env", profile_name))
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

        envs_vec.push(Env::from_key_value(
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
