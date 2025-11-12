/// Utility/helper functions specific to the CLI version of envio.
/// These functions are designed for CLI usage and may not be something used by users interacting with the API directly
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[cfg(target_family = "windows")]
use std::process::Command;

use chrono::{Local, NaiveDate};
use colored::Colorize;
use comfy_table::{Attribute, Cell, Table};
use envio::{
    crypto::EncryptionType,
    error::{Error, Result},
    EnvVec, Profile,
};

use crate::utils::{contains_path_separator, download_file, get_configdir, get_cwd};

/// Create a new profile which is stored in the profiles directory
///
/// # Parameters
/// - `name` - the name of the profile
/// - `envs` - the environment variables of the profile
/// - `encryption_type` - the encryption type of the profile
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
pub fn create_profile(
    name: String,
    envs: Option<EnvVec>,
    encryption_type: Box<dyn EncryptionType>,
) -> Result<()> {
    if Profile::does_exist(&name) {
        return Err(Error::ProfileAlreadyExists(name));
    }

    let envs = match envs {
        Some(env) => env,
        None => EnvVec::new(),
    };

    let config_dir = get_configdir()?;
    let profile_dir = config_dir.join("profiles");

    if !profile_dir.exists() {
        println!(
            "{}",
            "Profiles directory does not exist creating it now..".bold()
        );
        std::fs::create_dir_all(&profile_dir).unwrap();
    }

    let profile_file_path = profile_dir.join(name.clone() + ".env");

    Profile::new(name, envs, profile_file_path, encryption_type).push_changes()?;

    println!("{}: Profile created", "Success".green());
    Ok(())
}

pub fn check_expired_envs(profile: &Profile) {
    for env in &profile.envs {
        if let Some(date) = env.expiration_date {
            if date <= Local::now().date_naive() {
                println!(
                    "{}: Environment variable '{}' has expired",
                    "Warning".yellow(),
                    env.name
                );
            }
        }
    }
}

/// Export all the environment variables of the profile to a file in plain text
///
/// # Parameters
/// - `profile` - the profile to export ([Profile] object)
/// - `file_name` - the name of the file to export to
/// - `envs_selected` - the environment variables to export
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
pub fn export_envs(
    profile: &Profile,
    file_name: &str,
    envs_selected: &Option<Vec<String>>,
) -> Result<()> {
    let path = if contains_path_separator(file_name) {
        PathBuf::from(file_name)
    } else {
        get_cwd().join(file_name)
    };

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)
        .unwrap();

    let mut buffer = String::from("");

    if profile.envs.is_empty() {
        return Err(Error::EmptyProfile(profile.name.to_string()));
    }

    let mut keys: Vec<_> = profile.envs.keys();

    if let Some(envs_selected) = envs_selected {
        if !envs_selected.is_empty() {
            keys = keys
                .into_iter()
                .filter(|item| envs_selected.contains(item))
                .collect::<Vec<String>>();
        }

        if keys.is_empty() {
            return Err(Error::Msg("No envs to export".to_string()));
        }
    }

    for key in keys {
        buffer = buffer + key.as_str() + "=" + profile.envs.get(key.as_str()).unwrap() + "\n";
    }

    write!(file, "{}", buffer)?;

    println!("{}", "Exported envs".bold());
    Ok(())
}

/// List the environment variables stored in a profile
///
/// # Parameters
/// - `profile` - the profile to list the environment variables of ([Profile] object)
pub fn list_envs(profile: &Profile, display_comments: bool, display_expired: bool) {
    let mut table = Table::new();

    let mut header = vec![
        Cell::new("Environment Variable").add_attribute(Attribute::Bold),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ];

    if display_comments {
        header.push(Cell::new("Comment").add_attribute(Attribute::Bold));
    }

    if display_expired {
        header.push(Cell::new("Expiration Date").add_attribute(Attribute::Bold));
    }

    table.set_header(header);

    let mut row;
    for env in &profile.envs {
        row = vec![env.name.clone(), env.value.clone()];

        if display_comments {
            if let Some(comment) = &env.comment {
                row.push(comment.clone());
            } else {
                row.push("No comment".to_string());
            }
        }

        if display_expired {
            if let Some(date) = &env.expiration_date {
                row.push(date.to_string());
            } else {
                row.push(NaiveDate::from_ymd_opt(1970, 1, 1).unwrap().to_string());
            }
        }

        table.add_row(row);
    }

    println!("{table}");
}

/// Delete a profile from the profiles directory
///
/// # Parameters
/// - `name` - the name of the profile to delete
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
pub fn delete_profile(name: &str) -> Result<()> {
    if Profile::does_exist(name) {
        let configdir = get_configdir()?;
        let profile_path = configdir.join("profiles").join(format!("{}.env", name));

        match std::fs::remove_file(profile_path) {
            Ok(_) => println!("{}: Deleted profile", "Success".green()),
            Err(e) => return Err(Error::Io(e)),
        }
    } else {
        return Err(Error::ProfileDoesNotExist(name.to_string()));
    }

    Ok(())
}

/// List all the stored profiles in the profiles directory
///
/// # Parameters
/// - `raw` - whether to list the profiles in raw format. If true, the profiles
///   will be listed without any decorations
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
pub fn list_profiles(raw: bool) -> Result<()> {
    let configdir = get_configdir()?;
    let profile_dir = configdir.join("profiles");

    if !profile_dir.exists() {
        return Err(Error::Msg("profiles directory does not exist".to_string()));
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(profile_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        match path.extension() {
            None => continue,
            Some(ext) => {
                if ext != "env" {
                    continue;
                }
            }
        }
        let profile_name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        if profile_name.starts_with('.') {
            continue;
        }
        profiles.push(profile_name);
    }

    if raw {
        if profiles.is_empty() {
            println!("{}", "No profiles found".bold());
            return Ok(());
        }
        for profile in profiles {
            println!("{}", profile);
        }
        return Ok(());
    }

    let mut table = Table::new();
    table.set_header(vec![Cell::new("Profiles").add_attribute(Attribute::Bold)]);

    for profile in profiles {
        table.add_row(vec![profile]);
    }

    println!("{table}");
    Ok(())
}

/// Download a profile from a URL and store it in the profiles directory
///
/// # Parameters
/// - `url` - the URL to download the profile from
/// - `profile_name` - the name of the profile to store the downloaded profile as
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
pub fn download_profile(url: String, profile_name: String) -> Result<()> {
    println!("Downloading profile from {}", url);
    let configdir = get_configdir()?;

    let location = match configdir
        .join("profiles")
        .join(profile_name.clone() + ".env")
        .to_str()
    {
        Some(location) => location.to_owned(),
        None => {
            return Err(Error::Msg("Could not convert path to string".to_string()));
        }
    };

    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            return Err(Error::Msg(format!("Failed to create tokio runtime: {}", e)));
        }
    };

    runtime.block_on(download_file(url.as_str(), location.as_str()))?;

    println!("Downloaded profile: {}", profile_name);
    Ok(())
}

/// Import a profile stored somewhere on the system but not in the profiles directory
///
/// # Parameters
/// - `file_path` - the path to the profile file
/// - `profile_name` - the name of the profile to store the imported profile as
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
pub fn import_profile(file_path: String, profile_name: String) -> Result<()> {
    if !Path::new(&file_path).exists() {
        return Err(Error::Msg(format!("File `{}` does not exist", file_path)));
    }

    let configdir = get_configdir()?;

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&file_path)
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let location = match configdir
        .join("profiles")
        .join(profile_name.clone() + ".env")
        .to_str()
    {
        Some(location) => location.to_owned(),
        None => {
            return Err(Error::Msg("Could not convert path to string".to_string()));
        }
    };

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(location)
        .unwrap();

    file.write_all(contents.as_bytes())?;

    Ok(())
}

// Unix specific code
// Creates a shell script that can be sourced to set the environment variables
#[cfg(target_family = "unix")]
pub fn create_shellscript(profile: &Profile) -> Result<()> {
    let configdir = get_configdir()?;
    let shellscript_path = configdir.join("setenv.sh");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .append(false)
        .open(shellscript_path)?;

    // Build export statements for bash/zsh
    let mut bash_exports = String::new();
    for env in &profile.envs {
        // Escape single quotes in values
        let escaped_value = env.value.replace('\'', "'\\''");
        bash_exports.push_str(&format!("export {}='{}'\n", env.name, escaped_value));
    }

    // Build set statements for fish
    let mut fish_exports = String::new();
    for env in &profile.envs {
        // Escape single quotes in values
        let escaped_value = env.value.replace('\'', "'\\''");
        fish_exports.push_str(&format!("set -gx {} '{}'\n", env.name, escaped_value));
    }

    let shellscript = format!(
        r#"#!/bin/bash
# This script was generated by envio and should not be modified!

SHELL_NAME=$(basename "$SHELL")

case "$SHELL_NAME" in
    bash | zsh)
{}
        ;;
    fish)
{}
        ;;
    *)
        echo -e "\e[31mError: \e[0mUnsupported shell ($SHELL_NAME)" >&2
        ;;
esac
"#,
        bash_exports.trim_end(),
        fish_exports.trim_end()
    );

    file.write_all(shellscript.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

/// Load the environment variables of the profile into the current session
///
/// # Parameters
/// - `profile` - the profile to load
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
#[cfg(target_family = "unix")]
pub fn load_profile(profile: Profile) -> Result<()> {
    create_shellscript(&profile)?;

    println!("{}", "Environment variables loaded successfully!".green());

    Ok(())
}

/// Windows implementation of the load_profile function
#[cfg(target_family = "windows")]
pub fn load_profile(profile: Profile) -> envio::error::Result<()> {
    for env in profile.envs {
        let output = Command::new("setx").arg(&env.name).arg(&env.value).output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(envio::error::Error::Msg(format!(
                        "Failed to execute setx for environment variable: {} with value: {}",
                        env.name, env.value
                    )));
                }
            }
            Err(e) => {
                return Err(envio::error::Error::Msg(format!("{}", e)));
            }
        }
    }

    println!("{}", "Environment variables loaded successfully!".green());
    println!("{}", "Note: Reload your shell to apply changes on Windows".yellow());
    Ok(())
}

/// Unload the environment variables of the profile from the current session
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
#[cfg(target_family = "unix")]
pub fn unload_profile() -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .open(get_configdir()?.join("setenv.sh"))
        .unwrap();

    file.set_len(0)?;

    println!("{}", "Environment variables unloaded successfully!".green());
    Ok(())
}

/// Windows implementation of the unload_profile function
#[cfg(target_family = "windows")]
pub fn unload_profile(profile: Profile) -> Result<()> {
    for env in profile.envs.keys() {
        let status = Command::new("REG")
            .arg("delete")
            .arg("HKCU\\Environment")
            .arg("/F")
            .arg("/V")
            .arg(&env)
            .status();

        match status {
            Ok(status) => {
                if !status.success() {
                    return Err(Error::Msg(format!(
                        "Failed to delete environment variable: {}",
                        env
                    )));
                }
            }
            Err(e) => {
                return Err(Error::Msg(format!("{}", e)));
            }
        }
    }
    println!("{}", "Environment variables unloaded successfully!".green());
    println!("{}", "Note: Reload your shell to apply changes on Windows".yellow());

    Ok(())
}
