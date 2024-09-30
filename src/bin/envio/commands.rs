/// Implementation of all the subcommands that can be run by the CLI
use chrono::Local;
use colored::Colorize;
use inquire::{
    min_length, Confirm, DateSelect, MultiSelect, Password, PasswordDisplayMode, Select, Text,
};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::io::Read;
use std::path::Path;
use url::Url;

use envio::crypto::create_encryption_type;
use envio::crypto::gpg::get_gpg_keys;
use envio::error::{Error, Result};
use envio::{load_profile, Env, EnvVec, Profile};

use crate::clap_app::Command;
use crate::cli::{self, check_expired_envs};
use crate::utils::parse_envs_from_string;

/// Get the user's encryption key
fn get_userkey() -> String {
    println!("{}", "Loading Profile".green());
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

/// Check to see if the user is using a vi based editor so that we can use the vim mode in the inquire crate
fn get_vim_mode() -> Result<bool> {
    let env = env::var("VISUAL").unwrap_or_else(|_| env::var("EDITOR").unwrap_or_default());

    let program = env.split_whitespace().next().ok_or("")?; // Throw an error if the program is empty, we don't really care about the error message

    let program_stem = Path::new(program)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or("")?; // Same here

    Ok(Regex::new(r"n?vim?").unwrap().is_match(program_stem)) // unwrap is safe here because we know that the regex will always compile
}

impl Command {
    /// Run the subcommand that was passed to the CLI
    pub fn run(&self) -> Result<()> {
        let vim_mode = get_vim_mode().unwrap_or(false);

        match self {
            Command::Create {
                profile_name,
                envs,
                envs_file,
                gpg,
                add_comments,
                add_expiration_date,
            } => {
                if profile_name.is_empty() {
                    return Err(Error::ProfileNameEmpty(profile_name.to_string()));
                }

                if Profile::does_exist(profile_name) {
                    return Err(Error::ProfileExists(profile_name.to_string()));
                }

                let gpg_key;
                let encryption_type;

                if gpg.is_some() {
                    if gpg.as_ref().unwrap() == "select" {
                        let available_keys;

                        #[cfg(target_family = "unix")]
                        {
                            available_keys = get_gpg_keys()?;
                        }

                        #[cfg(target_family = "windows")]
                        {
                            available_keys = match get_gpg_keys() {
                                Some(keys) => keys,
                                None => {
                                    return Err(Error::Crypto("No GPG keys found".to_string()));
                                }
                            };

                            if available_keys.len() == 0 {
                                return Err(Error::Crypto("No GPG keys found".to_string()));
                            }
                        }

                        let ans = Select::new(
                            "Select the GPG key you want to use for encryption:",
                            available_keys.iter().map(|(s, _)| s.clone()).collect(),
                        )
                        .with_vim_mode(vim_mode)
                        .prompt();

                        if let Err(e) = ans {
                            return Err(Error::Msg(e.to_string()));
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

                    encryption_type = create_encryption_type(gpg_key, "gpg")?;
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
                        return Err(Error::Msg(e.to_string()));
                    } else {
                        prompt.unwrap()
                    };

                    encryption_type = create_encryption_type(user_key, "age")?;
                }

                let mut envs_vec;

                if envs_file.is_some() {
                    let file = envs_file.as_ref().unwrap();

                    if !Path::new(file).exists() {
                        return Err(Error::Msg(format!("File '{}' does not exist", file)));
                    }

                    let mut file = std::fs::OpenOptions::new().read(true).open(file).unwrap();

                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer).unwrap();

                    envs_vec = Some(parse_envs_from_string(&buffer)?);

                    if envs_vec.is_none() {
                        return Err(Error::Msg("Unable to parse envs from file".to_string()));
                    }

                    let mut options = vec![];

                    for env in envs_vec.clone().unwrap() {
                        if env.value.is_empty() {
                            let prompt = Confirm::new(&format!(
                                "Would you like to assign a value to key: {} ?",
                                env.name
                            ))
                            .with_default(false)
                            .with_help_message(
                                "If you do not want to assign a value to this key, press enter",
                            )
                            .prompt();

                            if let Err(e) = prompt {
                                return Err(Error::Msg(e.to_string()));
                            } else if prompt.unwrap() {
                                let prompt =
                                    Text::new(&format!("Enter the value for {}:", env.name))
                                        .prompt();

                                if let Err(e) = prompt {
                                    return Err(Error::Msg(e.to_string()));
                                } else {
                                    envs_vec.as_mut().unwrap().push(Env::from_key_value(
                                        env.name.clone(),
                                        prompt.unwrap(),
                                    ));
                                }
                            }
                        }

                        // we add the keys to the options list so that we can use them in the multi select prompt.
                        // The reason we do not have this in a separate loop is for efficiency reasons
                        options.push(env.name.clone());
                    }

                    let default_options = (0..options.len()).collect::<Vec<usize>>();

                    let prompt = MultiSelect::new("Select the environment variables you want to keep in your new profile:", options.clone())
                        .with_default(&default_options)
                        .with_vim_mode(vim_mode)
                        .with_help_message("↑↓ to move, space to select/unselect one, → to all, ← to none, type to filter, enter to confirm")
                        .prompt();

                    if let Err(e) = prompt {
                        return Err(Error::Msg(e.to_string()));
                    } else {
                        // remove the keys that were not selected
                        let selected_keys = prompt.unwrap();

                        for key in options {
                            if !selected_keys.contains(&key) {
                                envs_vec.as_mut().unwrap().remove(&key);
                            }
                        }
                    }
                } else if envs.is_some() {
                    envs_vec = Some(EnvVec::new());

                    for env in envs.as_ref().unwrap() {
                        if (*env).contains('=') {
                            let mut parts = env.splitn(2, '=');

                            if let Some(key) = parts.next() {
                                if let Some(value) = parts.next() {
                                    envs_vec.as_mut().unwrap().push(Env::from_key_value(
                                        key.to_string(),
                                        value.to_string(),
                                    ));
                                } else {
                                    return Err(Error::Msg(format!(
                                        "Unable to parse value for key '{}'",
                                        key
                                    )));
                                }
                            } else {
                                return Err(Error::Msg(format!(
                                    "Unable to parse key-value pair from '{}'",
                                    env
                                )));
                            }

                            continue;
                        }

                        let value;

                        let prompt = Text::new(&format!("Enter the value for {}:", env)).prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            value = prompt.unwrap();
                            envs_vec
                                .as_mut()
                                .unwrap()
                                .push(Env::from_key_value(env.to_string(), value));
                        }
                    }
                } else {
                    envs_vec = Some(EnvVec::new()); // The user created a profile without any envs
                }

                for env in envs_vec.as_mut().unwrap() {
                    if *add_comments {
                        let prompt =
                            Text::new(&format!("Enter a comment for '{}':", env.name)).prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            env.comment = Some(prompt.unwrap());
                        }
                    }

                    if *add_expiration_date {
                        let prompt = DateSelect::new(&format!(
                            "Select an expiration date for '{}':",
                            env.name
                        ))
                        .with_default(Local::now().date_naive())
                        .prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            env.expiration_date = Some(prompt.unwrap());
                        }
                    }
                }

                cli::create_profile(profile_name.to_string(), envs_vec, encryption_type)?;
            }

            Command::Add {
                profile_name,
                envs,
                add_comments,
                add_expiration_date,
            } => {
                if !Profile::does_exist(profile_name) {
                    return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
                }

                let mut profile = load_profile!(profile_name, get_userkey)?;
                check_expired_envs(&profile);

                for env in envs {
                    if (*env).contains('=') {
                        let mut parts = env.splitn(2, '=');

                        if let Some(key) = parts.next() {
                            if profile.envs.contains_key(key) {
                                return Err(Error::EnvExists(key.to_string()));
                            }

                            if let Some(value) = parts.next() {
                                profile.insert_env(key.to_string(), value.to_string())
                            } else {
                                return Err(Error::Msg(format!(
                                    "Unable to parse value for key '{}'",
                                    key
                                )));
                            }
                        } else {
                            return Err(Error::Msg(format!(
                                "Unable to parse key-value pair from '{}'",
                                env
                            )));
                        }

                        continue;
                    }

                    if profile.envs.contains_key(env) {
                        return Err(Error::EnvExists(env.to_string()));
                    }

                    let value;

                    let prompt = Text::new(&format!("Enter the value for {}:", env)).prompt();

                    if let Err(e) = prompt {
                        return Err(Error::Msg(e.to_string()));
                    } else {
                        value = prompt.unwrap();
                        profile.insert_env(env.to_string(), value)
                    }
                }

                for env in &mut profile.envs {
                    if envs.iter().find(|&e| e.contains(&env.name)).is_none() {
                        continue;
                    }

                    if *add_comments {
                        let prompt =
                            Text::new(&format!("Enter a comment for '{}':", env.name)).prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            env.comment = Some(prompt.unwrap());
                        }
                    }

                    if *add_expiration_date {
                        let prompt = DateSelect::new(&format!(
                            "Select an expiration date for '{}':",
                            env.name
                        ))
                        .with_default(Local::now().date_naive())
                        .prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            env.expiration_date = Some(prompt.unwrap());
                        }
                    }
                }

                println!("{}", "Applying Changes".green());
                profile.push_changes()?;
            }

            Command::Load { profile_name } => {
                #[cfg(target_family = "unix")]
                {
                    cli::load_profile(profile_name)?;
                }

                #[cfg(target_family = "windows")]
                {
                    if !Profile::does_exist(profile_name) {
                        return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
                    }

                    let profile = load_profile!(profile_name, get_userkey)?;
                    check_expired_envs(&profile);

                    if let Err(e) = cli::load_profile(profile) {
                        return Err(e);
                    }
                }
            }

            #[cfg(target_family = "unix")]
            Command::Unload => {
                cli::unload_profile()?;
            }

            #[cfg(target_family = "windows")]
            Command::Unload { profile_name } => {
                if !Profile::does_exist(profile_name) {
                    return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
                }

                let profile = load_profile!(profile_name, get_userkey)?;
                check_expired_envs(&profile);

                if let Err(e) = cli::unload_profile(profile) {
                    return Err(e);
                }
            }
            Command::Launch {
                profile_name,
                command,
            } => {
                let split_command = command.value();
                let program = split_command[0];
                let args = &split_command[1..];

                if !Profile::does_exist(profile_name) {
                    return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
                }

                let profile = load_profile!(profile_name, get_userkey)?;
                check_expired_envs(&profile);

                let mut cmd = std::process::Command::new(program)
                    .envs::<HashMap<String, String>, _, _>(profile.envs.into())
                    .args(args)
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .spawn()
                    .expect("Failed to execute command");

                let status = match cmd.wait() {
                    Ok(s) => s,
                    Err(e) => return Err(Error::Msg(format!("Failed to execute command: {}", e))),
                };

                match status.code() {
                    Some(code) => std::process::exit(code),
                    None => {
                        return Err(Error::Msg(
                            "The child process was terminated by a signal".to_string(),
                        ))
                    }
                }
            }

            Command::Remove { profile_name, envs } => {
                if !Profile::does_exist(profile_name) {
                    return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
                }

                if envs.is_some() && !envs.as_ref().unwrap().is_empty() {
                    let mut profile = load_profile!(profile_name, get_userkey)?;
                    check_expired_envs(&profile);

                    for env in envs.as_ref().unwrap() {
                        profile.remove_env(env)?;
                    }

                    println!("{}", "Applying Changes".green());
                    profile.push_changes()?;
                } else {
                    cli::delete_profile(profile_name)?;
                }
            }

            Command::List {
                profiles,
                profile_name,
                no_pretty_print,
                display_comments,
                display_expiration_date,
            } => {
                if *profiles {
                    cli::list_profiles(*no_pretty_print)?;
                } else if profile_name.is_some() && !profile_name.as_ref().unwrap().is_empty() {
                    if !Profile::does_exist(profile_name.as_ref().unwrap()) {
                        return Err(Error::ProfileDoesNotExist(
                            profile_name.as_ref().unwrap().to_string(),
                        ));
                    }

                    let profile = load_profile!(profile_name.as_ref().unwrap(), get_userkey)?;
                    check_expired_envs(&profile);

                    if *no_pretty_print {
                        for env in profile.envs {
                            println!("{}={}", env.name, env.value);
                        }
                    } else {
                        cli::list_envs(&profile, *display_comments, *display_expiration_date);
                    }
                }
            }

            Command::Update {
                profile_name,
                envs,
                update_values,
                update_comments,
                update_expiration_date,
            } => {
                if !Profile::does_exist(profile_name) {
                    return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
                }

                if envs.is_empty() {
                    return Err(Error::Msg(
                        "You must provide at least one environment variable to update".to_string(),
                    ));
                }

                let mut profile = load_profile!(profile_name, get_userkey)?;
                check_expired_envs(&profile);

                if !*update_values && !*update_comments && !*update_expiration_date {
                    return Err(Error::Msg(
                        "You must provide at least one flag to update".to_string(),
                    ));
                }

                if *update_values {
                    for env in envs {
                        if (*env).contains('=') {
                            let mut parts = env.splitn(2, '=');

                            if let Some(key) = parts.next() {
                                if !profile.envs.contains_key(key) {
                                    return Err(Error::EnvDoesNotExist(key.to_string()));
                                }

                                if let Some(value) = parts.next() {
                                    profile.edit_env(key.to_string(), value.to_string())?
                                } else {
                                    return Err(Error::Msg(format!(
                                        "Unable to parse value for key '{}'",
                                        key
                                    )));
                                }
                            } else {
                                return Err(Error::Msg(format!(
                                    "Unable to parse key-value pair from '{}'",
                                    env
                                )));
                            }

                            continue;
                        }

                        if !profile.envs.contains_key(env) {
                            return Err(Error::EnvDoesNotExist(env.to_string()));
                        }

                        let new_value;

                        let prompt =
                            Text::new(&format!("Enter the new value for {}:", env)).prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            new_value = prompt.unwrap();
                            profile.edit_env(env.to_string(), new_value)?;
                        }
                    }
                }

                for env in &mut profile.envs {
                    if envs.iter().find(|&e| e.contains(&env.name)).is_none() {
                        continue;
                    }

                    if *update_comments {
                        let prompt =
                            Text::new(&format!("Enter a new comment for '{}':", env.name)).prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            env.comment = Some(prompt.unwrap());
                        }
                    }

                    if *update_expiration_date {
                        let prompt = DateSelect::new(&format!(
                            "Select a new expiration date for '{}':",
                            env.name
                        ))
                        .with_default(Local::now().date_naive())
                        .prompt();

                        if let Err(e) = prompt {
                            return Err(Error::Msg(e.to_string()));
                        } else {
                            env.expiration_date = Some(prompt.unwrap());
                        }
                    }
                }

                println!("{}", "Applying Changes".green());
                profile.push_changes()?;
            }

            Command::Export {
                profile_name,
                file,
                envs,
            } => {
                if !Profile::does_exist(profile_name) {
                    return Err(Error::ProfileDoesNotExist(profile_name.to_string()));
                }

                let mut file_name = ".env";

                if file.is_some() {
                    file_name = file.as_ref().unwrap()
                }

                let profile = load_profile!(profile_name, get_userkey)?;
                check_expired_envs(&profile);

                if envs.is_some() && envs.as_ref().unwrap().contains(&"select".to_string()) {
                    let prompt = MultiSelect::new("Select the environment variables you want to export:", profile.envs.keys())
                        .with_default(&(0..profile.envs.len()).collect::<Vec<usize>>())
                        .with_vim_mode(vim_mode)
                        .with_help_message("↑↓ to move, space to select/unselect one, → to all, ← to none, type to filter, enter to confirm")
                        .prompt();

                    if let Err(e) = prompt {
                        return Err(Error::Msg(e.to_string()));
                    }

                    cli::export_envs(
                        &profile,
                        file_name,
                        &Some(
                            prompt
                                .unwrap()
                                .iter()
                                .cloned()
                                .map(|s| s.to_owned())
                                .collect(),
                        ),
                    )?;

                    return Ok(());
                }

                cli::export_envs(&profile, file_name, envs)?;
            }

            Command::Import {
                profile_name,
                file,
                url,
            } => {
                if Profile::does_exist(profile_name) {
                    return Err(Error::ProfileExists(profile_name.to_string()));
                }

                if url.is_some() && Url::parse(url.as_ref().unwrap()).is_ok() {
                    cli::download_profile(
                        url.as_ref().unwrap().to_string(),
                        profile_name.to_string(),
                    )?;

                    return Ok(());
                }

                if file.is_some() {
                    cli::import_profile(
                        file.as_ref().unwrap().to_string(),
                        profile_name.to_string(),
                    )?;
                    return Ok(());
                }

                return Err(Error::Msg(
                    "You must provide either a file or a url to import a profile".to_string(),
                ));
            }

            Command::Version { verbose } => {
                if *verbose {
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

        Ok(())
    }
}
