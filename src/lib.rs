//! `envio` is a library that simplifies the management of environment variables across multiple profiles
//!
//! It allows users to easily switch between different configurations and apply them to their current environment
//! `envio` also encrypts sensitive environment variable values to ensure secure storage and transmission of environment variables
//!
//! Here is a very simple example:
//! ```
//! // In this example we get the profile passed as an argument to the program
//! // and then print the environment variables in that profile
//!
//! use envio;
//!
//! let args: Vec<String> = std::env::args().collect();
//!
//! if args.len() != 2 {
//!    println!("Usage: <profile_name> <key>");
//!    return;
//! }
//!
//! let profile_name = args[1].to_string();
//! let key = &args[2]; // All profiles have a key that is used to encrypt the environment variables, this ensures that the environment variables are secure
//!
//!
//!  for (env_var, value) in &envio::get_profile(profile_name, key).unwrap().envs {
//!    println!("{}: {}", env_var, value);
//! }
//! ```
//!

pub mod crypto;
pub mod utils;

use std::{
    collections::HashMap,
    io::{Read, Write},
    path::{Path, PathBuf},
};

#[cfg(target_family = "windows")]
use std::process::Command;

use colored::Colorize;
use comfy_table::{Attribute, Cell, Table};

use crate::{
    crypto::{decrypt, encrypt},
    utils::{download_file, get_configdir, get_cwd},
};

/*
 * The Profile struct is used to store the environment variables of a profile
 * It also stores the name of the profile and the path to the profile file
*/
pub struct Profile {
    pub name: String,
    pub envs: HashMap<String, String>,
    pub profile_file_path: PathBuf,
    key: String,
}

impl Profile {
    /*
    * Create a new Profile object

    @param name String
    @param envs HashMap<String, String>
    @param profile_file_path PathBuf
    @param key String
    @return Profile
    */
    pub fn new(
        name: String,
        envs: HashMap<String, String>,
        profile_file_path: PathBuf,
        key: String,
    ) -> Self {
        Profile {
            name,
            envs,
            profile_file_path,
            key,
        }
    }

    /*
    * Add a new environment variable to the profile

    @param env String
    @param env_value String
    */
    pub fn add_env(&mut self, env: String, env_value: String) {
        self.envs.insert(env, env_value);
    }

    /*
    * Edit an existing environment variable of the profile
    * If the environment variable does not exists, it will print an error message

    @param env String
    @param new_value String
    */
    pub fn edit_env(&mut self, env: String, new_value: String) {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.envs.entry(env.clone()) {
            e.insert(new_value);
        } else {
            println!("{}: env `{}` does not exists", "Error".red(), env);
        }
    }

    /*
    * Remove an existing environment variable of the profile
    * If the environment variable does not exists, it will print an error message

    @param env String
    */
    pub fn remove_env(&mut self, env: String) {
        if self.envs.contains_key(&env) {
            self.envs.remove(&env);
        } else {
            println!("{}: env `{}` does not exists", "Error".red(), env);
        }
    }

    /*
     * List all the environment variables of the profile
     */
    pub fn list_envs(&self) {
        let mut table = Table::new();
        table.set_header(vec![
            Cell::new("Environment Variable").add_attribute(Attribute::Bold),
            Cell::new("Value").add_attribute(Attribute::Bold),
        ]);

        for key in self.envs.keys() {
            table.add_row(vec![key, self.envs.get(key).unwrap()]);
        }

        println!("{table}");
    }

    /*
    * Export the environment variables of the profile to a file

    * If the file does not exist, it will be created
    * If the file exists, it will be overwritten
    * If the profile does not have any environment variables, it will print an error message
    * The file will be created in the current working directory

    @param file_name String
    */
    pub fn export_envs(&self, file_name: String) {
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .open(get_cwd().join(file_name))
            .unwrap();

        let mut buffer = String::from("");

        if self.envs.is_empty() {
            println!("{}: No envs to export", "Error".red());
            return;
        }

        for key in self.envs.keys() {
            buffer = buffer + key + "=" + self.envs.get(key).unwrap() + "\n";
        }

        if let Err(e) = writeln!(file, "{}", buffer) {
            println!("{}: {}", "Error".red(), e);
        }

        println!("{}", "Exported envs".bold());
    }

    /*
     * Push the changes to the profile file
     */
    pub fn push_changes(&mut self) {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .append(false)
            .open(&self.profile_file_path)
            .unwrap();

        if let Err(e) = file.set_len(0) {
            println!("{}: {}", "Error".red(), e);
        }

        let mut buffer = String::from("");

        for key in self.envs.keys() {
            buffer = buffer + key + "=" + self.envs.get(key).unwrap() + "\n";
        }

        let encrypted_data = encrypt(self.key.clone(), buffer);
        if let Err(e) = file.write_all(encrypted_data.as_slice()) {
            println!("{}: {}", "Error".red(), e);
        }

        if let Err(e) = file.flush() {
            println!("{}: {}", "Error".red(), e);
        }

        if let Err(e) = file.sync_all() {
            println!("{}: {}", "Error".red(), e);
        }
    }
}

/*
* Create a new profile
* If the profile already exists, it will print an error message

@param name String
@param envs Option<HashMap<String, String>>
@param user_key String
*/
pub fn create_profile(name: String, envs: Option<HashMap<String, String>>, user_key: &str) {
    if check_profile(name.clone()) {
        println!("{}: Profile already exists", "Error".red());
        return;
    }

    let envs = match envs {
        Some(env) => env,
        None => HashMap::new(),
    };

    let config_dir = get_configdir();
    let profile_dir = config_dir.join("profiles");

    if !profile_dir.exists() {
        println!(
            "{}",
            "Profiles directory does not exist creating it now..".bold()
        );
        std::fs::create_dir_all(&profile_dir).unwrap();
    }

    let profile_file = profile_dir.join(name + ".env");

    let mut file = if let Err(e) = std::fs::File::create(&profile_file) {
        println!("{}: {}", "Error".red(), e);
        return;
    } else {
        std::fs::File::create(&profile_file).unwrap()
    };

    if envs.is_empty() {
        if let Err(e) = file.write_all(encrypt(user_key.to_string(), "".to_string()).as_slice()) {
            println!("{}: {}", "Error".red(), e);
        }

        return;
    }

    let mut buffer = String::from("");

    for key in envs.keys() {
        buffer = buffer + key + "=" + envs.get(&key.to_string()).unwrap() + "\n";
    }

    if let Err(e) = file.write_all(encrypt(user_key.to_string(), buffer).as_slice()) {
        println!("{}: {}", "Error".red(), e);
    }

    println!("{}: Profile created", "Success".green());
}

/*
* Check if the profile exists

@param name String
@return bool
*/
pub fn check_profile(name: String) -> bool {
    let configdir = get_configdir();

    let profile_path = configdir.join("profiles").join(name + ".env");

    if profile_path.exists() {
        return true;
    }

    false
}

/*
* Delete a profile
* If the profile does not exist, it will print an error message

@param name String
*/
pub fn delete_profile(name: String) {
    if check_profile(name.clone()) {
        let configdir = get_configdir();
        let profile_path = configdir.join("profiles").join(name.clone() + ".env");

        match std::fs::remove_file(profile_path) {
            Ok(_) => println!("Deleted profile: {}", name),
            Err(e) => println!("{}: {}", "Error".red(), e),
        }
    } else {
        println!("{}: Profile does not exist", "Error".red());
    }
}

/*
 * List all the profiles
 */
pub fn list_profiles(raw: bool) {
    let configdir = get_configdir();
    let profile_dir = configdir.join("profiles");

    if !profile_dir.exists() {
        println!("{}: No profiles found", "Error".red());
        return;
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(profile_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let file_name = path.file_name().unwrap().to_str().unwrap().to_owned();
        let profile_name = file_name.replace(".env", "");
        profiles.push(profile_name);
    }

    if raw {
        for profile in profiles {
            println!("{}", profile);
        }
        return;
    }

    let mut table = Table::new();
    table.set_header(vec![Cell::new("Profiles").add_attribute(Attribute::Bold)]);

    for profile in profiles {
        table.add_row(vec![profile]);
    }

    println!("{table}");
}

/*
* Download the profile from the url and save it to the config directory with the profile name
* passed

@param url String
@param profile_name String
*/
pub fn download_profile(url: String, profile_name: String) {
    println!("Downloading profile from {}", url);

    let location = if get_configdir()
        .join("profiles")
        .join(profile_name.clone() + ".env")
        .to_str()
        .is_none()
    {
        println!("{}: Could not get convert path to string", "Error".red());
        return;
    } else {
        get_configdir()
            .join("profiles")
            .join(profile_name.clone() + ".env")
            .to_str()
            .unwrap()
            .to_owned()
    };

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        })
        .block_on(download_file(url.as_str(), location.as_str()));

    println!("Downloaded profile: {}", profile_name);
}

/*
* Import a profile from a file

@param file_path String
@param profile_name String
*/
pub fn import_profile(file_path: String, profile_name: String) {
    if !Path::new(&file_path).exists() {
        println!("{}: File does not exist", "Error".red());
        return;
    }

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&file_path)
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let location = if get_configdir()
        .join("profiles")
        .join(profile_name.clone() + ".env")
        .to_str()
        .is_none()
    {
        println!("{}: Could not get convert path to string", "Error".red());
        return;
    } else {
        get_configdir()
            .join("profiles")
            .join(profile_name + ".env")
            .to_str()
            .unwrap()
            .to_owned()
    };

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .open(location)
        .unwrap();

    if let Err(e) = file.write(contents.as_bytes()) {
        println!("{}: {}", "Error".red(), e);
    }
}

/*
* Returns a profile object from the profile name passed, it checks if the profile exists
* All profiles are stored in a directory called `profiles` in the config directory
*
* If the profile does not exist, it will return None
*
* If it exists, it will read the contents of the file and decrypt it using the key passed
* It will then return a profile object containing the name of the profile and a hashmap of the environment variables

@param profile_name String
@param key String

@return Option<Profile>
*/
pub fn get_profile(profile_name: String, key: &str) -> Option<Profile> {
    if !check_profile(profile_name.clone()) {
        println!("{}: Profile does not exist", "Error".red());
        return None;
    }

    let profile_file_path = get_configdir()
        .join("profiles")
        .join(format!("{}.env", profile_name));

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&profile_file_path)
        .unwrap();

    let mut encrypted_contents = Vec::new();
    file.read_to_end(&mut encrypted_contents).unwrap();

    let content = decrypt(key.to_string(), &encrypted_contents);

    let mut envs = HashMap::new();

    for line in content.lines() {
        if line.is_empty() {
            continue;
        }

        let mut split = line.split('=');
        let key = split.next().unwrap();

        let value = split.next().unwrap();

        envs.insert(key.to_string(), value.to_string());
    }

    Some(Profile::new(
        profile_name,
        envs,
        profile_file_path,
        key.to_string(),
    ))
}

// Unix specific code
// Creates a shell script that can be sourced to set the environment variables
#[cfg(any(target_family = "unix"))]
pub fn create_shellscript(profile: &str) {
    let configdir = get_configdir();
    let shellscript_path = configdir.join("setenv.sh");

    let mut file = if let Ok(e) = std::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .open(shellscript_path)
    {
        e
    } else {
        println!("{}: Could not open file", "Error".red());
        return;
    };

    let shellscript = format!(
        r#"#!/bin/bash
# This script was generated by envio and should not be modified!


TMP_FILE=$(mktemp)

set +e
ENV_VARS=$(envio list {} -- --no-pretty-print)
EXIT_CODE=$?
SHELL_NAME=$(basename "$SHELL")


if [ ! -f "$TMP_FILE" ]; then
    echo "Failed to create temp file"
    exit 1
fi

if [ $EXIT_CODE -ne 0 ]; then
    echo "Failed to load environment variables"
    rm "$TMP_FILE"
else
    case "$SHELL_NAME" in
        bash | zsh)
            echo "$ENV_VARS" | while IFS= read -r line; do
                echo "export $line" >> "$TMP_FILE"
            done
            ;;
        fish)
            echo "$ENV_VARS" | while IFS= read -r line; do
                echo "set -gX $line" >> "$TMP_FILE"
            done
            ;;
        *)
            echo "Unsupported shell: $SHELL_NAME"
            exit 1
            ;;
    esac

    if [ ! -s "$TMP_FILE" ]; then
        echo "Temp file is empty"
        exit 1
    fi

    source "$TMP_FILE"
    source "$TMP_FILE" &> /dev/null

    if [ $? -ne 0 ]; then
        echo "Failed to load environment variables from temp file"
        exit 1
    fi

    rm "$TMP_FILE"
fi
"#,
        profile
    );

    if let Err(e) = file.write_all(shellscript.as_bytes()) {
        println!("{}: {}", "Error".red(), e);
    }

    if let Err(e) = file.flush() {
        println!("{}: {}", "Error".red(), e);
    }

    if let Err(e) = file.sync_all() {
        println!("{}: {}", "Error".red(), e);
    }
}

// Unix specific code
// Returns the shell that is being used
// @return String
#[cfg(any(target_family = "unix"))]
pub fn get_shell_config() -> String {
    // Gets your default shell
    // This is used to determine which shell config file to edit
    let shell_env_value = if let Ok(e) = std::env::var("SHELL") {
        e
    } else {
        println!("{}: Could not get shell", "Error".red());
        std::process::exit(1);
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

    shell_config.to_string()
}

/*
 * Load the environment variables of the profile into the current session
 */
#[cfg(any(target_family = "unix"))]
pub fn load_profile(profile_name: &str) {
    if !check_profile(profile_name.to_string()) {
        println!("{}: Profile does not exist", "Error".red());
        return;
    }

    let shell_config = get_shell_config();

    create_shellscript(profile_name);

    if !shell_config.is_empty() {
        println!(
            "Reload your shell to apply changes or run `source {}`",
            format_args!("~/{}", shell_config)
        );
    } else {
        println!("Reload your shell to apply changes");
    }
}

/*
 * Windows specific code
*/

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

/*
 * Unload the environment variables of the profile from the current session
 */
#[cfg(any(target_family = "unix"))]
pub fn unload_profile() {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .open(get_configdir().join("setenv.sh"))
        .unwrap();

    if let Err(e) = file.set_len(0) {
        println!("{}: {}", "Error".red(), e);
    }

    println!("Reload your shell to apply changes");
}

/*
 * Windows specific code
*/
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
