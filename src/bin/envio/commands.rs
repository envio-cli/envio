use std::{collections::HashMap, io::Read, path::Path};

use chrono::Local;
use colored::Colorize;
use envio::{
    crypto::{create_cipher, gpg::get_gpg_keys, CipherKind},
    get_profile, Env, EnvVec,
};
use strum::IntoEnumIterator;
use url::Url;

use crate::{
    clap_app::{ClapApp, Command, ProfileCommand},
    error::{AppError, AppResult},
    ops,
    output::error,
    prompts, utils,
};

fn get_userkey() -> String {
    match prompts::password_prompt(prompts::PasswordPromptOptions {
        title: "Enter your encryption key:".to_string(),
        help_message: Some("OH NO! you forgot your key! just kidding... or did you?".to_string()),
        min_length: None,
        with_confirmation: false,
        confirmation_error_message: None,
    }) {
        Ok(key) => key,
        Err(e) => {
            error(e);
            std::process::exit(1);
        }
    }
}

impl ClapApp {
    pub fn run(&self) -> AppResult<()> {
        match &self.command {
            Command::Profile(ProfileCommand::New {
                profile_name,
                description,
                envs,
                envs_file,
                cipher_kind,
                comments: add_comments,
                expires: add_expires,
            }) => {
                let selected_cipher_kind = if let Some(kind) = cipher_kind {
                    kind.parse::<CipherKind>()
                        .map_err(|e| AppError::Msg(e.to_string()))?
                } else {
                    let cipher_options: Vec<String> =
                        CipherKind::iter().map(|k| k.to_string()).collect();

                    prompts::select_prompt(prompts::SelectPromptOptions {
                        title: "Select the encryption method:".to_string(),
                        options: cipher_options,
                    })?
                    .parse::<CipherKind>()
                    .unwrap() // always safe
                };

                let key = match selected_cipher_kind {
                    CipherKind::GPG => {
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
                                    return Err(AppError::Msg("No GPG keys found".to_string()));
                                }
                            };
                        }

                        if available_keys.is_empty() {
                            return Err(AppError::Msg("No GPG keys found".to_string()));
                        }

                        Some(prompts::select_prompt(prompts::SelectPromptOptions {
                            title: "Select the GPG key you want to use for encryption:".to_string(),
                            options: available_keys.iter().map(|(s, _)| s.clone()).collect(),
                        })?)
                    }

                    CipherKind::AGE => {
                        Some(prompts::password_prompt(prompts::PasswordPromptOptions {
                            title: "Enter your encryption key:".to_string(),
                            help_message: Some(
                                "Remember this key, you will need it to decrypt your profile later"
                                    .to_string(),
                            ),
                            min_length: Some(8),
                            with_confirmation: true,
                            confirmation_error_message: Some("The keys don't match".to_string()),
                        })?)
                    }

                    _ => None,
                };

                let cipher = create_cipher(selected_cipher_kind, key.as_deref())?;

                let mut envs_vec;

                if envs_file.is_some() {
                    let file = envs_file.as_ref().unwrap();

                    if !Path::new(file).exists() {
                        return Err(AppError::Msg(format!("File '{}' does not exist", file)));
                    }

                    let mut file = std::fs::OpenOptions::new().read(true).open(file).unwrap();

                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer).unwrap();

                    envs_vec = Some(utils::parse_envs_from_string(&buffer)?);

                    if envs_vec.is_none() {
                        return Err(AppError::Msg("Unable to parse envs from file".to_string()));
                    }

                    let mut options = vec![];

                    for env in envs_vec.clone().unwrap() {
                        if env.value.is_empty() {
                            if prompts::confirm_prompt(prompts::ConfirmPromptOptions {
                                title: format!(
                                    "Would you like to assign a value to key: {} ?",
                                    env.name
                                ),
                                help_message: Some(
                                    "If you do not want to assign a value to this key, press enter"
                                        .to_string(),
                                ),
                                default: false,
                            })? {
                                let value = prompts::text_prompt(prompts::TextPromptOptions {
                                    title: format!("Enter the value for {}:", env.name),
                                    default: None,
                                })?;
                                envs_vec
                                    .as_mut()
                                    .unwrap()
                                    .push(Env::from_key_value(env.name.clone(), value));
                            }
                        }

                        // we add the keys to the options list so that we can use them in the multi
                        // select prompt. The reason we do not have this in
                        // a separate loop is for efficiency reasons
                        options.push(env.name.clone());
                    }

                    let default_options = (0..options.len()).collect::<Vec<usize>>();
                    let selected_keys = prompts::multi_select_prompt(prompts::MultiSelectPromptOptions {
                        title:
                            "Select the environment variables you want to keep in your new profile:"
                                .to_string(),
                        options: options.clone(),
                        default_indices: Some(default_options),
                    })?;

                    for key in options {
                        if !selected_keys.contains(&key) {
                            envs_vec.as_mut().unwrap().remove(&key);
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
                                    return Err(AppError::Msg(format!(
                                        "Unable to parse value for key '{}'",
                                        key
                                    )));
                                }
                            } else {
                                return Err(AppError::Msg(format!(
                                    "Unable to parse key-value pair from '{}'",
                                    env
                                )));
                            }

                            continue;
                        }

                        let value = prompts::text_prompt(prompts::TextPromptOptions {
                            title: format!("Enter the value for {}:", env),
                            default: None,
                        })?;
                        envs_vec
                            .as_mut()
                            .unwrap()
                            .push(Env::from_key_value(env.to_string(), value));
                    }
                } else {
                    envs_vec = Some(EnvVec::new()); // the user created a profile without any envs
                }

                for env in envs_vec.as_mut().unwrap() {
                    if *add_comments {
                        env.comment = Some(prompts::text_prompt(prompts::TextPromptOptions {
                            title: format!("Enter a comment for '{}':", env.name),
                            default: None,
                        })?);
                    }

                    if *add_expires {
                        env.expiration_date =
                            Some(prompts::date_prompt(prompts::DatePromptOptions {
                                title: format!("Select an expiration date for '{}':", env.name),
                                default: Some(Local::now().date_naive()),
                            })?);
                    }
                }

                ops::create_profile(
                    profile_name.to_string(),
                    description.clone(),
                    envs_vec,
                    cipher,
                )?;
            }

            Command::Set {
                profile_name,
                envs,
                comments: add_comments,
                expires: add_expires,
            } => {
                let mut profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                ops::check_expired_envs(&profile);

                for env in envs {
                    if env.contains('=') {
                        let mut parts = env.splitn(2, '=');

                        if let Some(key) = parts.next() {
                            if let Some(value) = parts.next() {
                                if profile.envs.contains_key(key) {
                                    profile.edit_env(key.to_string(), value.to_string())?;
                                } else {
                                    profile.insert_env(key.to_string(), value.to_string());
                                }
                            } else {
                                return Err(AppError::Msg(format!(
                                    "Unable to parse value for key '{}'",
                                    key
                                )));
                            }
                        } else {
                            return Err(AppError::Msg(format!(
                                "Unable to parse key-value pair from '{}'",
                                env
                            )));
                        }

                        continue;
                    }

                    let prompt = prompts::text_prompt(prompts::TextPromptOptions {
                        title: format!(
                            "Enter the {}value for {}:",
                            if profile.envs.contains_key(env) {
                                "new "
                            } else {
                                ""
                            },
                            env
                        ),
                        default: None,
                    })?;

                    if profile.envs.contains_key(env) {
                        profile.edit_env(env.to_string(), prompt)?;
                    } else {
                        profile.insert_env(env.to_string(), prompt);
                    }
                }

                for env in &mut profile.envs {
                    if envs.iter().find(|&e| e.contains(&env.name)).is_none() {
                        continue;
                    }

                    if *add_comments {
                        env.comment = Some(prompts::text_prompt(prompts::TextPromptOptions {
                            title: format!("Enter a comment for '{}':", env.name),
                            default: None,
                        })?);
                    }

                    if *add_expires {
                        env.expiration_date =
                            Some(prompts::date_prompt(prompts::DatePromptOptions {
                                title: format!("Select an expiration date for '{}':", env.name),
                                default: Some(Local::now().date_naive()),
                            })?);
                    }
                }

                println!("{}", "Applying Changes".green());
                profile.save()?;
            }

            Command::Unset { profile_name, keys } => {
                let mut profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                ops::check_expired_envs(&profile);

                for key in keys {
                    profile.remove_env(&key)?;
                }

                println!("{}", "Applying Changes".green());
                profile.save()?;
            }

            Command::Load { profile_name } => {
                #[cfg(target_family = "unix")]
                {
                    ops::load_profile(profile_name)?;
                }

                #[cfg(target_family = "windows")]
                {
                    let profile =
                        get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                    ops::load_profile(profile)?;
                }
            }

            #[cfg(target_family = "unix")]
            Command::Unload => {
                ops::unload_profile()?;
            }

            #[cfg(target_family = "windows")]
            Command::Unload { profile_name } => {
                let profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                ops::unload_profile(profile)?;
            }

            Command::Run {
                profile_name,
                command,
            } => {
                if command.is_empty() {
                    return Err(AppError::Msg("Command cannot be empty".to_string()));
                }

                let program = &command[0];
                let args = &command[1..];

                let profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;
                ops::check_expired_envs(&profile);

                let mut cmd = std::process::Command::new(program)
                    .envs::<HashMap<String, String>, _, _>(profile.envs.into())
                    .args(args)
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .spawn()
                    .expect("Failed to execute command");

                let status = match cmd.wait() {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(AppError::Msg(format!("Failed to execute command: {}", e)))
                    }
                };

                match status.code() {
                    Some(code) => std::process::exit(code),
                    None => {
                        return Err(AppError::Msg(
                            "The child process was terminated by a signal".to_string(),
                        ))
                    }
                }
            }

            Command::Profile(ProfileCommand::Delete { profile_name }) => {
                ops::delete_profile(profile_name)?;
            }

            Command::Profile(ProfileCommand::List { no_pretty_print }) => {
                ops::list_profiles(*no_pretty_print)?;
            }

            Command::Profile(ProfileCommand::Show {
                profile_name,
                no_pretty_print,
                show_comments,
                show_expiration,
            }) => {
                let profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;
                ops::check_expired_envs(&profile);

                if *no_pretty_print {
                    for env in profile.envs {
                        println!("{}={}", env.name, env.value);
                    }
                } else {
                    ops::list_envs(&profile, *show_comments, *show_expiration);
                }
            }

            Command::Export {
                profile_name,
                output_file_path,
                keys,
            } => {
                let profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                ops::check_expired_envs(&profile);

                let envs_selected = if keys.is_some() {
                    let keys_vec = keys.as_ref().unwrap();
                    if keys_vec.contains(&"select".to_string()) {
                        let keys = profile.envs.keys();
                        let default_indices: Vec<usize> = (0..keys.len()).collect();
                        Some(prompts::multi_select_prompt(
                            prompts::MultiSelectPromptOptions {
                                title: "Select the environment variables you want to export:"
                                    .to_string(),
                                options: keys,
                                default_indices: Some(default_indices),
                            },
                        )?)
                    } else {
                        Some(keys_vec.clone())
                    }
                } else {
                    None
                };

                ops::export_envs(
                    &profile,
                    output_file_path.as_deref().unwrap_or(".env"),
                    &envs_selected,
                )?;
            }

            Command::Import {
                source,
                profile_name,
            } => {
                let profile_name = if let Some(name) = profile_name {
                    name.clone()
                } else {
                    Path::new(source)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("imported")
                        .to_string()
                };

                if Url::parse(source).is_ok() {
                    ops::download_profile(source.to_string(), profile_name)?;
                    return Ok(());
                }

                if Path::new(source).exists() {
                    ops::import_profile(source.to_string(), profile_name)?;
                    return Ok(());
                }

                return Err(AppError::Msg(
                    "Source must be a valid file path or URL".to_string(),
                ));
            }

            Command::Version { verbose } => {
                println!("{} {}", "Version".green(), env!("BUILD_VERSION"));

                if *verbose {
                    println!("{} {}", "Author".green(), env!("CARGO_PKG_AUTHORS"));
                    println!("{} {}", "License".green(), env!("CARGO_PKG_LICENSE"));
                    println!("{} {}", "Repository".green(), env!("CARGO_PKG_REPOSITORY"));
                    println!("{} {}", "Build Timestamp".green(), env!("BUILD_TIMESTAMP"));
                }
            }
        }

        Ok(())
    }
}
