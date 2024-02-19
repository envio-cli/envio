use colored::Colorize;
use inquire::{min_length, Confirm, MultiSelect, Password, PasswordDisplayMode, Select, Text};

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use url::Url;

use envio::crypto::{create_encryption_type, get_encryption_type, get_gpg_keys};

use envio::{
    self, check_profile, create_profile, delete_profile, download_profile, get_profile,
    import_profile, list_profiles, load_profile, parse_envs_from_string, unload_profile,
};

use crate::cli::Command;

/**
 * Get the user key from the user using the inquire crate

 @return String
*/
fn get_userkey() -> String {
    println!("{}", "Loading Profile".green());
    println!("{}", "Enter your encryption key".green());
    let prompt = Password::new("Enter your encryption key:")
        .with_display_toggle_enabled()
        .with_display_mode(PasswordDisplayMode::Masked)
        .with_help_message("OH NO! you forgot your key! just kidding... or did you?")
        .without_confirmation()
        .prompt();

    if let Err(e) = prompt {
        println!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    } else {
        prompt.unwrap()
    }
}

impl Command {
    /**
     * Run the subcommand that was passed to the program
     */
    pub fn run(&self) {
        match self {
            Command::Create {
                profile_name,
                envs,
                envs_file,
                gpg,
            } => {
                if profile_name.is_empty() {
                    println!("{}: Profile name can not be empty!", "Error".red());
                    return;
                }

                if check_profile(profile_name) {
                    println!("{}: Profile already exists", "Error".red());
                    return;
                }

                let gpg_key;
                let encryption_type;

                if gpg.is_some() {
                    if gpg.as_ref().unwrap() == "select" {
                        let available_keys;

                        #[cfg(target_family = "unix")]
                        {
                            available_keys = get_gpg_keys();
                        }

                        #[cfg(target_family = "windows")]
                        {
                            available_keys = get_gpg_keys().unwrap();
                            if available_keys.len() == 0 {
                                println!("{}: No GPG keys found", "Error".red());
                                return;
                            }
                        }

                        let ans = Select::new(
                            "Select the GPG key you want to use for encryption:",
                            available_keys.iter().map(|(s, _)| s.clone()).collect(),
                        )
                        .prompt();

                        if let Err(e) = ans {
                            println!("{}: {}", "Error".red(), e);
                            return;
                        }

                        gpg_key = available_keys
                            .iter()
                            .find_map(|(k, f)| {
                                if k == ans.as_ref().unwrap() {
                                    Some(f.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap();
                    } else {
                        gpg_key = gpg.as_ref().unwrap().to_string();
                    }
                    encryption_type = create_encryption_type(gpg_key, "gpg");
                } else {
                    let prompt = Password::new("Enter your encryption key:")
                        .with_display_toggle_enabled()
                        .with_display_mode(PasswordDisplayMode::Masked)
                        .with_validator(min_length!(8))
                        .with_formatter(&|_| String::from("Input received"))
                        .with_help_message(
                            "Remeber this key, you will need it to decrypt your profile later",
                        )
                        .with_custom_confirmation_error_message("The keys don't match.")
                        .prompt();

                    let user_key = if let Err(e) = prompt {
                        println!("{}: {}", "Error".red(), e);
                        return;
                    } else {
                        prompt.unwrap()
                    };

                    encryption_type = create_encryption_type(user_key, "age");
                }

                let mut envs_hashmap;

                if envs_file.is_some() {
                    let file = envs_file.as_ref().unwrap();

                    if !Path::new(file).exists() {
                        println!("{}: File does not exist", "Error".red());
                        return;
                    }

                    let mut file = std::fs::OpenOptions::new().read(true).open(file).unwrap();

                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer).unwrap();

                    envs_hashmap = Some(parse_envs_from_string(&buffer));

                    if envs_hashmap.is_none() {
                        println!("{}: Unable to parse the file", "Error".red());
                        return;
                    }

                    let mut options = vec![];

                    for (key, value) in envs_hashmap.as_ref().unwrap().clone() {
                        if value.is_empty() {
                            let prompt = Confirm::new(&format!(
                                "Would you like to assign a value to key: {} ?",
                                key
                            ))
                            .with_default(false)
                            .with_help_message(
                                "If you do not want to assign a value to this key, press enter",
                            )
                            .prompt();

                            if let Err(e) = prompt {
                                println!("{}: {}", "Error".red(), e);
                                std::process::exit(1);
                            } else if prompt.unwrap() {
                                let prompt =
                                    Text::new(&format!("Enter the value for {}:", key)).prompt();

                                if let Err(e) = prompt {
                                    println!("{}: {}", "Error".red(), e);
                                    std::process::exit(1);
                                } else {
                                    envs_hashmap
                                        .as_mut()
                                        .unwrap()
                                        .insert(key.to_string(), prompt.unwrap());
                                }
                            }
                        }

                        // we add the keys to the options list so that we can use them in the multi select prompt.
                        // The reason we do not have this in a separate loop is for efficiency reasons
                        options.push(key);
                    }

                    let default_options = (0..options.len()).collect::<Vec<usize>>();

                    let prompt = MultiSelect::new("Select the environment variables you want to keep in your new profile:", options.clone())
                        .with_default(&default_options)
                        .with_help_message("↑↓ to move, space to select one, → to all, ← to none, type to filter, enter to confirm")
                        .prompt();

                    if let Err(e) = prompt {
                        println!("{}: {}", "Error".red(), e);
                        std::process::exit(1);
                    } else {
                        // remove the keys that were not selected
                        let selected_keys = prompt.unwrap();

                        for key in options {
                            if !selected_keys.contains(&key) {
                                envs_hashmap.as_mut().unwrap().remove(&key);
                            }
                        }
                    }
                } else if envs.is_some() {
                    envs_hashmap = Some(HashMap::new());

                    for env in envs.as_ref().unwrap() {
                        if (*env).contains('=') {
                            let mut parts = env.splitn(2, '=');

                            if let Some(key) = parts.next() {
                                if let Some(value) = parts.next() {
                                    envs_hashmap
                                        .as_mut()
                                        .unwrap()
                                        .insert(key.to_string(), value.to_string());
                                } else {
                                    println!(
                                        "{}: Unable to parse value for key '{}'",
                                        "Error".red(),
                                        key
                                    );
                                }
                            } else {
                                println!(
                                    "{}: Unable to parse key-value pair from '{}'",
                                    "Error".red(),
                                    env
                                );
                            }

                            continue;
                        }

                        let value;

                        let prompt = Text::new(&format!("Enter the value for {}:", env)).prompt();

                        if let Err(e) = prompt {
                            println!("{}: {}", "Error".red(), e);
                            std::process::exit(1);
                        } else {
                            value = prompt.unwrap();
                            envs_hashmap
                                .as_mut()
                                .unwrap()
                                .insert(env.to_string(), value);
                        }
                    }
                } else {
                    envs_hashmap = None;
                }

                create_profile(profile_name.to_string(), envs_hashmap, encryption_type);
            }

            Command::Add { profile_name, envs } => {
                if !check_profile(profile_name) {
                    println!("{}: Profile does not exist", "Error".red());
                    return;
                }

                let mut encryption_type = get_encryption_type(profile_name.to_string());
                if encryption_type.as_string() == "age" {
                    encryption_type.set_key(get_userkey());
                }

                let mut profile =
                    if let Some(p) = get_profile(profile_name.to_string(), encryption_type) {
                        p
                    } else {
                        return;
                    };

                for env in envs {
                    if (*env).contains('=') {
                        let mut parts = env.splitn(2, '=');

                        if let Some(key) = parts.next() {
                            if profile.envs.contains_key(key) {
                                println!(
                                    "{}: The environment variable `{}` already exists in profile",
                                    "Error".red(),
                                    key
                                );
                                return;
                            }

                            if let Some(value) = parts.next() {
                                profile.add_env(key.to_string(), value.to_string())
                            } else {
                                println!(
                                    "{}: Unable to parse value for key '{}'",
                                    "Error".red(),
                                    key
                                );
                            }
                        } else {
                            println!(
                                "{}: Unable to parse key-value pair from '{}'",
                                "Error".red(),
                                env
                            );
                        }

                        continue;
                    }

                    if profile.envs.contains_key(env) {
                        println!(
                            "{}: The environment variable `{}` already exists in profile",
                            "Error".red(),
                            env
                        );
                        return;
                    }

                    let value;

                    let prompt = Text::new(&format!("Enter the value for {}:", env)).prompt();

                    if let Err(e) = prompt {
                        println!("{}: {}", "Error".red(), e);
                        std::process::exit(1);
                    } else {
                        value = prompt.unwrap();
                        profile.add_env(env.to_string(), value)
                    }
                }
                println!("{}", "Applying Changes".green());
                profile.push_changes();
            }

            Command::Load { profile_name } => {
                #[cfg(target_family = "unix")]
                {
                    load_profile(profile_name);
                }

                #[cfg(target_family = "windows")]
                {
                    if !check_profile(profile_name) {
                        println!("{}: Profile does not exist", "Error".red());
                        return;
                    }

                    let mut encryption_type = get_encryption_type(profile_name.to_string());
                    if encryption_type.as_string() == "age" {
                        encryption_type.set_key(get_userkey());
                    }

                    let profile =
                        if let Some(p) = get_profile(profile_name.to_string(), encryption_type) {
                            p
                        } else {
                            return;
                        };

                    load_profile(profile);
                }
            }

            #[cfg(target_family = "unix")]
            Command::Unload => {
                unload_profile();
            }

            #[cfg(target_family = "windows")]
            Command::Unload { profile_name } => {
                if !check_profile(profile_name) {
                    println!("{}: Profile does not exist", "Error".red());
                    return;
                }

                let mut encryption_type = get_encryption_type(profile_name.to_string());
                if encryption_type.as_string() == "age" {
                    encryption_type.set_key(get_userkey());
                }

                let profile =
                    if let Some(p) = get_profile(profile_name.to_string(), encryption_type) {
                        p
                    } else {
                        return;
                    };

                unload_profile(profile);
            }
            Command::Launch {
                profile_name,
                command,
            } => {
                let split_command = command.value();
                let program = split_command[0];
                let args = &split_command[1..];

                if !check_profile(profile_name) {
                    println!("{}: Profile does not exist", "Error".red());
                    return;
                }

                let mut encryption_type = get_encryption_type(profile_name.to_string());
                if encryption_type.as_string() == "age" {
                    encryption_type.set_key(get_userkey());
                }

                let profile =
                    if let Some(p) = get_profile(profile_name.to_string(), encryption_type) {
                        p
                    } else {
                        return;
                    };

                let mut cmd = std::process::Command::new(program)
                    .envs(profile.envs)
                    .args(args)
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .spawn()
                    .expect("Failed to execute command");

                let status = match cmd.wait() {
                    Ok(s) => s,
                    Err(e) => {
                        println!("{}: {}", "Error".red(), e);
                        std::process::exit(1);
                    }
                };

                match status.code() {
                    Some(code) => std::process::exit(code),
                    None => {
                        println!("{}: Child process terminated by signal", "Error".red());
                        std::process::exit(1);
                    }
                }
            }

            Command::Remove { profile_name, envs } => {
                if !check_profile(profile_name) {
                    println!("{}: Profile does not exist", "Error".red());
                    return;
                }

                if envs.is_some() && !envs.as_ref().unwrap().is_empty() {
                    let mut encryption_type = get_encryption_type(profile_name.to_string());
                    if encryption_type.as_string() == "age" {
                        encryption_type.set_key(get_userkey());
                    }
                    let mut profile =
                        if let Some(p) = get_profile(profile_name.to_string(), encryption_type) {
                            p
                        } else {
                            return;
                        };

                    for env in envs.as_ref().unwrap() {
                        profile.remove_env(env);
                    }

                    println!("{}", "Applying Changes".green());
                    profile.push_changes();
                } else {
                    delete_profile(profile_name);
                }
            }

            Command::List {
                profiles,
                profile_name,
                no_pretty_print,
            } => {
                if *profiles {
                    if *no_pretty_print {
                        list_profiles(true)
                    } else {
                        list_profiles(false)
                    }
                } else if profile_name.is_some() && !profile_name.as_ref().unwrap().is_empty() {
                    if !check_profile(profile_name.as_ref().unwrap()) {
                        println!("{}: Profile does not exist", "Error".red());
                        return;
                    }

                    let mut encryption_type =
                        get_encryption_type(profile_name.as_ref().unwrap().to_string());

                    if encryption_type.as_string() == "age" {
                        encryption_type.set_key(get_userkey());
                    }

                    let profile = if let Some(p) =
                        get_profile(profile_name.as_ref().unwrap().to_string(), encryption_type)
                    {
                        p
                    } else {
                        return;
                    };

                    if *no_pretty_print {
                        for (key, value) in profile.envs.iter() {
                            println!("{}={}", key, value);
                        }
                    } else {
                        profile.list_envs();
                    }
                }
            }

            Command::Update { profile_name, envs } => {
                if !check_profile(profile_name) {
                    println!("{}: Profile does not exist", "Error".red());
                    return;
                }
                let mut encryption_type = get_encryption_type(profile_name.to_string());
                if encryption_type.as_string() == "age" {
                    encryption_type.set_key(get_userkey());
                }

                let mut profile =
                    if let Some(p) = get_profile(profile_name.to_string(), encryption_type) {
                        p
                    } else {
                        return;
                    };

                for env in envs {
                    if (*env).contains('=') {
                        let mut parts = env.splitn(2, '=');

                        if let Some(key) = parts.next() {
                            if !profile.envs.contains_key(key) {
                                println!(
                                    "{}: The environment variable `{}` does not exist in profile use the `add` command to add the variable",
                                    "Error".red(),
                                    key
                                );
                                return;
                            }

                            if let Some(value) = parts.next() {
                                profile.edit_env(key.to_string(), value.to_string())
                            } else {
                                println!(
                                    "{}: Unable to parse value for key '{}'",
                                    "Error".red(),
                                    key
                                );
                            }
                        } else {
                            println!(
                                "{}: Unable to parse key-value pair from '{}'",
                                "Error".red(),
                                env
                            );
                        }

                        continue;
                    }

                    if !profile.envs.contains_key(env) {
                        println!(
                            "{}: The environment variable `{}` does not exist in profile use the `add` command to add the variable",
                            "Error".red(),
                            env
                        );
                        return;
                    }

                    let new_value;

                    let prompt = Text::new(&format!("Enter the new value for {}:", env)).prompt();

                    if let Err(e) = prompt {
                        println!("{}: {}", "Error".red(), e);
                        std::process::exit(1);
                    } else {
                        new_value = prompt.unwrap();
                        profile.edit_env(env.to_string(), new_value)
                    }
                }

                println!("{}", "Applying Changes".green());
                profile.push_changes();
            }

            Command::Export { profile_name, file } => {
                if !check_profile(profile_name) {
                    println!("{}: Profile does not exist", "Error".red());
                    return;
                }

                let mut file_name = ".env";

                if file.is_some() {
                    file_name = &file.as_ref().unwrap()
                }

                let mut encryption_type = get_encryption_type(profile_name.to_string());
                if encryption_type.as_string() == "age" {
                    encryption_type.set_key(get_userkey());
                }

                let profile =
                    if let Some(p) = get_profile(profile_name.to_string(), encryption_type) {
                        p
                    } else {
                        return;
                    };

                profile.export_envs(file_name);
            }

            Command::Import {
                profile_name,
                file,
                url,
            } => {
                if check_profile(profile_name) {
                    println!("{}: Profile already exists", "Error".red());
                    return;
                }

                if url.is_some() && Url::parse(url.as_ref().unwrap()).is_ok() {
                    download_profile(url.as_ref().unwrap().to_string(), profile_name.to_string());
                    return;
                }

                if file.is_some() {
                    import_profile(file.as_ref().unwrap().to_string(), profile_name.to_string());
                    return;
                }

                println!("{}: You must specify a file or url", "Error".red());
            }

            Command::Version { verbose } => {
                if verbose.is_some() && verbose.unwrap() {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                    println!("{} {}", "Build Timestamp".green(), env!("BUILD_TIMESTAMP"));
                    println!("{} {}", "Author".green(), env!("CARGO_PKG_AUTHORS"));
                    println!("{} {}", "License".green(), env!("CARGO_PKG_LICENSE"));
                    println!("{} {}", "Repository".green(), env!("CARGO_PKG_REPOSITORY"));
                } else {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                }
            }
        }
    }
}
