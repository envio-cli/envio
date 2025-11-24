use std::{fs::File, io::Write, path::PathBuf};

use envio::{
    profile::{ProfileMetadata, SerializedProfile},
    Env, EnvMap,
};
use indicatif::{ProgressBar, ProgressState, ProgressStyle};
use reqwest::Client;

use crate::error::{AppError, AppResult};

#[cfg(target_family = "unix")]
pub fn initalize_config() -> AppResult<()> {
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
            .prompt()?;

            file_path = PathBuf::from(input);

            if !file_path.exists() {
                return Err(AppError::Msg(
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

pub fn get_profile_dir() -> PathBuf {
    get_configdir().join("profiles")
}

pub fn contains_path_separator(s: &str) -> bool {
    s.contains('/') || s.contains('\\')
}

pub fn get_cwd() -> PathBuf {
    std::env::current_dir().unwrap()
}

// use this to get the path of a profile that does not exist yet
// like when creating a new profile
pub fn build_profile_path(profile_name: &str) -> PathBuf {
    get_profile_dir().join(format!("{}.env", profile_name))
}

// use this to get the path of a profile that exists
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

    for buf in buffer.lines() {
        if buf.is_empty() || !buf.contains('=') {
            continue;
        }

        let mut split = buf.splitn(2, '=');

        let key = split.next();
        let value = split.next();

        if key.is_none() {
            return Err(AppError::Msg("Can not parse key from buffer".to_string()));
        }

        if value.is_none() {
            return Err(AppError::Msg(format!(
                "Can not parse value from buffer for key: `{}`",
                key.unwrap()
            )));
        }

        envs_vec.insert(Env::from_key_value(
            key.unwrap().to_string(),
            value.unwrap().to_string(),
        ));
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

    let content_length = resp.content_length();
    let style = match content_length {
        Some(total) => {
            pb.set_length(total);
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .with_key(
                    "eta",
                    |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                        write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                    },
                )
                .progress_chars("#>-")
        }
        None => return Err(AppError::Msg("content length is not available".to_string())),
    };

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
pub fn get_shell_config() -> AppResult<&'static str> {
    let shell_env_value = if let Ok(e) = std::env::var("SHELL") {
        e
    } else {
        return Err(AppError::Msg("Failed to get shell".to_string()));
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
