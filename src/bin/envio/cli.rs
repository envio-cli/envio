/// Utility/helper functions specific to the CLI version of envio.
/// These functions are designed for CLI usage and may not be something used by users interacting with the API directly
use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[cfg(target_family = "windows")]
use std::process::Command;

use colored::Colorize;
use comfy_table::{Attribute, Cell, Table};
use envio::{
    crypto::EncryptionType,
    error::{Error, Result},
    Profile,
};

use crate::utils::{contains_path_separator, download_file, get_configdir, get_cwd};

#[cfg(target_family = "unix")]
use crate::utils::get_shell_config;

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
    envs: Option<HashMap<String, String>>,
    encryption_type: Box<dyn EncryptionType>,
) -> Result<()> {
    if Profile::does_exist(&name) {
        return Err(Error::ProfileAlreadyExists(name));
    }

    let envs = match envs {
        Some(env) => env,
        None => HashMap::new(),
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

    let profile_file = profile_dir.join(name + ".env");

    let mut file = std::fs::File::create(profile_file)?;

    let mut buffer = String::from("");

    if envs.is_empty() {
        buffer = buffer + &encryption_type.get_key();
    } else {
        for key in envs.keys() {
            buffer = buffer + key + "=" + envs.get(&key.to_string()).unwrap() + "\n";
        }

        buffer = buffer + &encryption_type.get_key();
    }

    file.write_all(encryption_type.encrypt(&buffer).unwrap().as_slice())?;

    file.flush()?;

    file.sync_all()?;

    println!("{}: Profile created", "Success".green());
    Ok(())
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

    let mut keys: Vec<_> = profile.envs.keys().cloned().collect::<Vec<String>>();
    if let Some(envs_selected) = envs_selected {
        if !envs_selected.is_empty() {
            keys = profile
                .envs
                .keys()
                .filter(|item| envs_selected.contains(item))
                .cloned()
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
pub fn list_envs(profile: &Profile) {
    let mut table = Table::new();
    table.set_header(vec![
        Cell::new("Environment Variable").add_attribute(Attribute::Bold),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ]);

    for key in profile.envs.keys() {
        table.add_row(vec![key, profile.envs.get(key).unwrap()]);
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
pub fn create_shellscript(profile: &str) -> Result<()> {
    let configdir = get_configdir()?;
    let shellscript_path = configdir.join("setenv.sh");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .append(false)
        .open(shellscript_path)?;

    let shellscript = format!(
        r#"#!/bin/bash
# This script was generated by envio and should not be modified!

has_error_occurred=false
handle_error() {{
    has_error_occurred=true
}}

raw_output=$(envio list -n {} -v)

if ! echo "$raw_output" | grep -q "="; then
    echo -e "\e[31mError: \e[0mFailed to load environment variables from profile '{}'" >&2
    handle_error
fi

if [ "$has_error_occurred" = false ]; then
    ENV_VARS=$(echo "$raw_output" | awk -F "=" '/^[^=]+=.+/{{print}}')
    SHELL_NAME=$(basename "$SHELL")

    case "$SHELL_NAME" in
        bash | zsh)
            source <(echo '
            #!/bin/bash
            echo "$ENV_VARS" | while IFS= read -r line; do
                export $line
            done 
        ')
            ;;
        fish)
            source <(echo '
            #!/bin/fish
            echo "$ENV_VARS" | while IFS= read -r line; do
                set -gx $line
            done 
        ')
            ;;
        *)
            echo -e "\e[31mError: \e[0mUnsupported shell ($SHELL_NAME)" >&2
            handle_error
            ;;
    esac
fi
"#,
        profile, profile
    );

    file.write_all(shellscript.as_bytes())?;
    file.flush()?;
    file.sync_all()?;
    Ok(())
}

/// Load the environment variables of the profile into the current session
///
/// # Parameters
/// - `profile_name` - the name of the profile to load
///
/// # Returns
/// - `Result<()>`: whether the operation was successful
#[cfg(target_family = "unix")]
pub fn load_profile(profile_name: &str) -> Result<()> {
    if !Profile::does_exist(profile_name) {
        return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
    }

    let shell_config = get_shell_config()?;

    create_shellscript(profile_name)?;

    if !shell_config.is_empty() {
        println!(
            "Reload your shell to apply changes or run `source {}`",
            format_args!("~/{}", shell_config)
        );
    } else {
        println!("Reload your shell to apply changes");
    }

    Ok(())
}

/// Windows implementation of the load_profile function
#[cfg(target_family = "windows")]
pub fn load_profile(profile: Profile) {
    for (env, value) in &profile.envs {
        Command::new("setx")
            .arg(env)
            .arg(value)
            .spawn()
            .expect("setx command failed");
    }

    println!("Reload your shell to apply changes");
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

    println!("Reload your shell to apply changes");
    Ok(())
}

/// Windows implementation of the unload_profile function
#[cfg(target_family = "windows")]
pub fn unload_profile(profile: Profile) {
    for env in profile.envs.keys() {
        Command::new("REG")
            .arg("delete")
            .arg("HKCU\\Environment")
            .arg("/F")
            .arg("/V")
            .arg(format!("\"{}\"", env))
            .arg("")
            .spawn()
            .expect("setx command failed");
    }
    println!("Reload your shell to apply changes");
}
