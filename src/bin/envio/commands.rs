use colored::Colorize;
use inquire::{min_length, Password, PasswordDisplayMode};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use url::Url;

use envio::crypto::encrypt;
use envio::utils::get_configdir;
use envio::{
    self, check_profile, create_profile, delete_profile, download_profile, get_profile,
    import_profile, list_profiles, load_profile, unload_profile,
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
            Command::Create(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name;

                if command_args.args.len() == 1 {
                    profile_name = command_args.args[0].clone();
                } else if command_args.args.len() == 2 {
                    profile_name = command_args.args[1].clone();
                } else {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                if check_profile(profile_name.to_string()) {
                    println!("{}: Profile already exists", "Error".red());
                    return;
                }

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

                if command_args.args.len() == 1 {
                    create_profile(profile_name, None, &user_key);
                } else if command_args.args.len() == 2 {
                    if !Path::new(&command_args.args[0]).exists() {
                        println!("{}: File does not exist", "Error".red());
                        return;
                    }

                    let mut file = std::fs::OpenOptions::new()
                        .read(true)
                        .open(command_args.args[0].clone())
                        .unwrap();

                    let mut contents = String::new();
                    file.read_to_string(&mut contents).unwrap();

                    let buffer = encrypt(user_key, contents.clone());

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

                    if let Err(e) = file.write(buffer.as_slice()) {
                        println!("{}: {}", "Error".red(), e);
                    }
                    println!("{}: Profile created", "Success".green());
                } else {
                    let mut envs = HashMap::new();
                    for (count, arg) in command_args.args[1..].iter().enumerate() {
                        if count > command_args.args.len() - 2 {
                            break;
                        }

                        let mut split = arg.split('=');

                        let key = split.next();
                        let value = split.next();

                        if key.is_none() || value.is_none() {
                            println!("{}: Can not parse arguments", "Error".red());
                            println!(
                                "{}",
                                "Arguments should be in the format of key=value".bold()
                            );
                            return;
                        }

                        envs.insert(key.unwrap().to_owned(), value.unwrap().to_owned());
                    }

                    create_profile(command_args.args[0].clone(), Some(envs), &user_key);
                }
            }
            Command::Add(command_args) => {
                if command_args.args.len() <= 1 {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();

                let mut profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
                    p
                } else {
                    return;
                };

                for (count, arg) in command_args.args[1..].iter().enumerate() {
                    if count > command_args.args.len() - 2 {
                        break;
                    }

                    let mut split = arg.split('=');

                    let key = split.next();
                    let value = split.next();

                    if key.is_none() || value.is_none() {
                        println!("{}: Can not parse arguments", "Error".red());
                        println!(
                            "{}",
                            "Arguments should be in the format of key=value".bold()
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

            Command::Load(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();

                #[cfg(target_family = "unix")]
                {
                    load_profile(&profile_name);
                }

                #[cfg(target_family = "windows")]
                {
                    let profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
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
            Command::Unload(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();

                let profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
                    p
                } else {
                    return;
                };

                unload_profile(profile);
            }

            Command::Launch(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();
                let profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
                    p
                } else {
                    return;
                };

                let program_to_launch = command_args.args[1].clone();

                let output = std::process::Command::new(program_to_launch)
                    .envs(profile.envs)
                    .args(&command_args.args[2..])
                    .output()
                    .expect("Failed to execute process");

                if output.stderr.is_empty() {
                    println!("{}", String::from_utf8(output.stdout).unwrap());
                } else {
                    println!("{}", String::from_utf8(output.stderr).unwrap());
                }
            }

            Command::Remove(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                if command_args.args.len() == 1 {
                    delete_profile(command_args.args[0].clone());
                } else {
                    let profile_name = command_args.args[0].clone();
                    let mut profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
                        p
                    } else {
                        return;
                    };

                    for arg in command_args.args[1..].iter() {
                        profile.remove_env(arg.to_owned());
                    }

                    println!("{}", "Applying Changes".green());
                    profile.push_changes();
                }
            }
            Command::List(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();
                if command_args.args.len() == 2 {
                    if command_args.args[1] == "--no-pretty-print" {
                        if profile_name == "profiles" {
                            list_profiles(true)
                        } else {
                            let profile = if let Some(p) = get_profile(profile_name, &get_userkey())
                            {
                                p
                            } else {
                                return;
                            };

                            for (key, value) in profile.envs.iter() {
                                println!("{}={}", key, value);
                            }
                        }
                    }
                    return;
                }
                if profile_name == "profiles" {
                    list_profiles(false)
                } else {
                    let profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
                        p
                    } else {
                        return;
                    };

                    profile.list_envs();
                }
            }

            Command::Update(command_args) => {
                if command_args.args.len() <= 1 {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();

                let mut profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
                    p
                } else {
                    return;
                };

                for arg in command_args.args[1..].iter() {
                    let mut split = arg.split('=');

                    let key = split.next();
                    let value = split.next();

                    if key.is_none() || value.is_none() {
                        println!("{}: Can not parse arguments", "Error".red());
                        println!(
                            "{}",
                            "Arguments should be in the format of key=value".bold()
                        );
                        return;
                    }

                    if profile.envs.contains_key(key.unwrap()) {
                        profile.edit_env(key.unwrap().to_owned(), value.unwrap().to_owned())
                    } else {
                        println!(
                            "{}: The Environment Variable `{}` does not exist in profile use the `add` command to add the variable",
                            "Error".red(),
                            key.unwrap()
                        );
                        return;
                    }
                }

                println!("{}", "Applying Changes".green());
                profile.push_changes();
            }

            Command::Export(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                let profile_name = command_args.args[0].clone();
                let mut file_name = String::from(".env");

                if command_args.args.len() > 1 {
                    file_name = command_args.args[1].clone();
                }

                let profile = if let Some(p) = get_profile(profile_name, &get_userkey()) {
                    p
                } else {
                    return;
                };

                profile.export_envs(file_name);
            }

            Command::Import(command_args) => {
                if command_args.args.is_empty() {
                    println!("{}: Invalid number of arguments", "Error".red());
                    return;
                }

                if command_args.args.len() < 2 {
                    println!("{}: Please provide a profile name", "Error".red());
                    return;
                }

                let profile_name = command_args.args[1].clone();

                if check_profile(profile_name.clone()) {
                    println!("{}: Profile already exists", "Error".red());
                    return;
                }

                if Url::parse(command_args.args[0].as_str()).is_ok() {
                    download_profile(command_args.args[0].clone(), profile_name);
                    return;
                }

                let file_path = command_args.args[0].clone();
                let profile_name = command_args.args[1].clone();

                import_profile(file_path, profile_name);
            }

            Command::Version(command_args) => {
                if command_args.args.is_empty() {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                } else if command_args.args[0] == "verbose" {
                    println!("{} {}", "Version".green(), env!("BUILD_VERSION"));
                    println!("{} {}", "Build Timestamp".green(), env!("BUILD_TIMESTAMP"));
                    println!("{} {}", "Author".green(), env!("CARGO_PKG_AUTHORS"));
                    println!("{} {}", "License".green(), env!("CARGO_PKG_LICENSE"));
                    println!("{} {}", "Repository".green(), env!("CARGO_PKG_REPOSITORY"));
                } else {
                    println!("{}: Invalid argument", "Error".red());
                }
            }
        }
    }
}
