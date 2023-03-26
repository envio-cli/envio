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
    process::Command,
};

use colored::Colorize;
use comfy_table::{Attribute, Cell, Table};

use crate::{
    crypto::{decrypt, encrypt},
    utils::{download_file, get_configdir, get_cwd, get_homedir},
};

pub struct Profile {
    pub name: String,
    pub envs: HashMap<String, String>,
    pub profile_file_path: PathBuf,
    key: String,
}

impl Profile {
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

    pub fn load_profile(&self) {
        if cfg!(windows) {
            for (env, value) in &self.envs {
                Command::new("setx")
                    .arg(env)
                    .arg(value)
                    .spawn()
                    .expect("setx command failed");
            }

            println!("Reload your shell to apply changes");
            return;
        }

        #[cfg(any(target_family = "unix"))]
        {
            let configdir = get_configdir();
            let shellscript_path = configdir.join("setenv.sh");
            let mut shell_config = "";

            let current_shell = get_shell();

            if current_shell.contains("bash") {
                shell_config = ".bashrc";
            } else if current_shell.contains("zsh") {
                shell_config = ".zshrc";
            }

            if !shellscript_path.exists() {
                println!("Creating shell script");
                create_shellscript();

                if !shell_config.is_empty() {
                    let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .append(true)
                        .open(
                            get_homedir().to_str().unwrap().to_owned()
                                + &format!("/{}", shell_config),
                        )
                        .unwrap();

                    let buffer = format!("# envio \nsource {}", shellscript_path.to_str().unwrap());
                    if let Err(e) = writeln!(file, "{}", buffer) {
                        println!("{}: {}", "Error".red(), e);
                    }
                } else {
                    println!("The current shell is not supported. Please add the following line to your shell config file: source {}", shellscript_path.to_str().unwrap());
                }
            }

            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .append(false)
                .open(&shellscript_path)
                .unwrap();

            let mut buffer = String::from("");

            for (env, value) in &self.envs {
                buffer = buffer + &format!("export {}={}\n", env, value);
            }

            if let Err(e) = writeln!(file, "{}", buffer) {
                println!("{}: {}", "Error".red(), e);
            }

            println!(
                "Reload your shell to apply changes or run `source {}`",
                format_args!("~/{}", shell_config)
            );
        }
    }

    pub fn unload_profile(&self) {
        if cfg!(windows) {
            for env in self.envs.keys() {
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
        }

        #[cfg(any(target_family = "unix"))]
        {
            let file = std::fs::OpenOptions::new()
                .write(true)
                .append(false)
                .open(get_configdir().join("setenv.sh"))
                .unwrap();

            if let Err(e) = file.set_len(0) {
                println!("{}: {}", "Error".red(), e);
            }
        }

        println!("Reload your shell to apply changes");
    }

    pub fn add_env(&mut self, env: String, env_value: String) {
        self.envs.insert(env, env_value);
    }

    pub fn edit_env(&mut self, env: String, new_value: String) {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.envs.entry(env.clone()) {
            e.insert(new_value);
        } else {
            println!("{}: env `{}` does not exists", "Error".red(), env);
        }
    }

    pub fn remove_env(&mut self, env: String) {
        if self.envs.contains_key(&env) {
            self.envs.remove(&env);
        } else {
            println!("{}: env `{}` does not exists", "Error".red(), env);
        }
    }

    pub fn list_envs(&self) {
        println!("Listing envs from profile: {}", self.name);
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

    let profile_file = profile_dir.join(name.clone() + ".env");

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

    println!("Created profile: {}", name);
}

pub fn check_profile(name: String) -> bool {
    let configdir = get_configdir();

    let profile_path = configdir.join("profiles").join(name + ".env");

    if profile_path.exists() {
        return true;
    }

    false
}

pub fn delete_profile(name: String) {
    if check_profile(name.clone()) {
        let configdir = get_configdir();
        let profile_path = configdir.join("profiles").join(name.clone() + ".env");

        match std::fs::remove_file(profile_path) {
            Ok(_) => println!("Deleted profile: {}", name),
            Err(e) => println!("{}: {}", "Error".red(), e),
        }
    } else {
        println!("{}: Profile does not exists", "Error".red());
    }
}

pub fn list_profiles() {
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

    println!("Listing profiles");
    for profile in profiles {
        println!("{}", profile);
    }
}

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

pub fn import_profile(file_path: String, profile_name: String) {
    if !Path::new(&file_path).exists() {
        println!("{}: File does not exists", "Error".red());
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

pub fn get_profile(profile_name: String, key: &str) -> Option<Profile> {
    if !check_profile(profile_name.clone()) {
        println!("{}: Profile does not exist", "Error".red());
        return None;
    }

    let profile_file_path = home::home_dir()
        .unwrap()
        .join(".envio")
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

#[cfg(any(target_family = "unix"))]
pub fn create_shellscript() {
    let configdir = get_configdir();
    let shellscript_path = configdir.join("setenv.sh");

    let mut file = if let Err(e) = std::fs::File::create(&shellscript_path) {
        println!("{}: {}", "Error".red(), e);
        return;
    } else {
        std::fs::File::create(&shellscript_path).unwrap()
    };

    let shellscript = "";

    if let Err(e) = file.write_all(shellscript.as_bytes()) {
        println!("{}: {}", "Error".red(), e);
    }
}

#[cfg(any(target_family = "unix"))]
pub fn get_shell() -> String {
    let shell = std::env::var("SHELL").unwrap();
    let shell = shell.split('/').collect::<Vec<&str>>();

    shell[shell.len() - 1].to_string()
}
