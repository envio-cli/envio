use colored::Colorize;
use inquire::{min_length, Password, PasswordDisplayMode, Select};

use std::io::Read;
use std::path::Path;
use url::Url;

use envio::crypto::{create_encryption_type, get_encryption_type};

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

                if check_profile(profile_name.to_string()) {
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
                            //available_keys = get_gpg_keys();
                            available_keys = vec![
                                (
                                    format!(
                                        "{} <{}>",
                                        String::from("Humble Penguin"),
                                        String::from("humblepenguinoffical@gmail.com")
                                    ),
                                    "0321A87CF6A2CE28B499D3967468C855E2F39F75".to_string(),
                                ),
                                (
                                    format!(
                                        "{} <{}>",
                                        String::from("John Doe"),
                                        String::from("johndoe@gmail.com")
                                    ),
                                    "0321A87CF6A2CE28B499D3967468C855E2F39F75".to_string(),
                                ),
                            ]
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

                let envs_hashmap;

                if envs_file.is_some() {
                    let file = envs_file.as_ref().unwrap();

                    if !Path::new(file).exists() {
                        println!("{}: File does not exist", "Error".red());
                        return;
                    }

                    let mut file = std::fs::OpenOptions::new().read(true).open(file).unwrap();

                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer).unwrap();

                    envs_hashmap = Some(parse_envs_from_string(buffer));
                } else if envs.is_some() {
                    envs_hashmap = Some(parse_envs_from_string(envs.as_ref().unwrap().join(" ")));
                } else {
                    envs_hashmap = None;
                }

                create_profile(profile_name.to_string(), envs_hashmap, encryption_type);
            }

            Command::Add { profile_name, envs } => {
                if !check_profile(profile_name.to_string()) {
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
                    let mut split = env.split('=');

                    let key = split.next();
                    let value = split.next();

                    if key.is_none() || value.is_none() {
                        println!("{}: Can not parse Environment variable", "Error".red());
                        println!(
                            "{}",
                            "Environment variables should be in the format of KEY=VALUE".bold()
                        );
                        return;
                    }

                    if profile.envs.contains_key(key.unwrap()) {
                        println!("{}: The Environment variable `{}` already exists in profile use the update command to update the value", "Error".red(), key.unwrap());
                        return;
                    }

                    profile.add_env(key.unwrap().to_owned(), value.unwrap().to_owned());
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
                    if !check_profile(profile_name.to_string()) {
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
                if !check_profile(profile_name.to_string()) {
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
                program,
            } => {
                if !check_profile(profile_name.to_string()) {
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
                
                let output = std::process::Command::new(&program[0])
                    .envs(profile.envs)
                    .args(&program[1..])
                    .output()
                    .expect("Failed to execute process");

                if output.stderr.is_empty() {
                    println!("{}", String::from_utf8(output.stdout).unwrap());
                } else {
                    println!("{}", String::from_utf8(output.stderr).unwrap());
                }
            }

            Command::Remove { profile_name, envs } => {
                if !check_profile(profile_name.to_string()) {
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
                        profile.remove_env(env.to_string());
                    }

                    println!("{}", "Applying Changes".green());
                    profile.push_changes();
                } else {
                    delete_profile(profile_name.to_string());
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
                    if !check_profile(profile_name.as_ref().unwrap().to_string()) {
                        println!("{}: Profile does not exist", "Error".red());
                        return;
                    }

                    let mut encryption_type = get_encryption_type(profile_name.as_ref().unwrap().to_string());

                    if encryption_type.as_string() == "age" {
                        encryption_type.set_key(get_userkey());
                    }

                    let profile =
                        if let Some(p) = get_profile(profile_name.as_ref().unwrap().to_string(), encryption_type) {
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
                if !check_profile(profile_name.to_string()) {
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
                    let mut split = env.split('=');

                    let key = split.next();
                    let value = split.next();

                    if key.is_none() || value.is_none() {
                        println!("{}: Can not parse Environment variable", "Error".red());
                        println!(
                            "{}",
                            "Environment variables should be in the format of key=value".bold()
                        );
                        return;
                    }

                    if profile.envs.contains_key(key.unwrap()) {
                        profile.edit_env(key.unwrap().to_owned(), value.unwrap().to_owned())
                    } else {
                        println!(
                            "{}: The Environment variable `{}` does not exist in profile use the `add` command to add the variable",
                            "Error".red(),
                            key.unwrap()
                        );
                        return;
                    }
                }

                println!("{}", "Applying Changes".green());
                profile.push_changes();
            }

            Command::Export { profile_name, file } => {
                if !check_profile(profile_name.to_string()) {
                    println!("{}: Profile does not exist", "Error".red());
                    return;
                }

                let mut file_name = String::from(".env");

                if file.is_some() {
                    file_name = file.as_ref().unwrap().to_string();
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
                if check_profile(profile_name.to_string()) {
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
