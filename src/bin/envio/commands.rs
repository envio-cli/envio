use std::{io::Read, path::Path};

use chrono::Local;
use colored::Colorize;
use envio::{
    Env, EnvMap,
    cipher::{CipherKind, create_cipher, gpg::get_gpg_keys},
    get_profile,
    profile::SerializedProfile,
};
use indexmap::IndexMap;
use strum::IntoEnumIterator;
use url::Url;

use crate::{
    clap_app::{ClapApp, Command},
    completions,
    diagnostic::DiagnosticReport,
    error::{AppError, AppResult},
    error_msg, ops, prompts, success_msg,
    tui::TuiApp,
    utils,
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
            error_msg!(e);
            std::process::exit(1);
        }
    }
}

impl ClapApp {
    pub fn run(&self) -> AppResult<()> {
        if self.diagnostic {
            DiagnosticReport::generate()?.print()?;
            return Ok(());
        }

        match &self.command {
            Command::Create {
                profile_name,
                description,
                envs,
                envs_file,
                cipher_kind,
                comments: add_comments,
                expires: add_expires,
            } => {
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
                        let available_keys = get_gpg_keys()?;

                        if available_keys.is_empty() {
                            return Err(AppError::Msg("No GPG keys found".to_string()));
                        }

                        let labels: Vec<String> = available_keys
                            .iter()
                            .map(|(label, _)| label.clone())
                            .collect();

                        let selected_label =
                            prompts::select_prompt(prompts::SelectPromptOptions {
                                title: "Select the GPG key you want to use for encryption:"
                                    .to_string(),
                                options: labels,
                            })?;

                        let fingerprint = available_keys
                            .into_iter()
                            .find(|(label, _)| *label == selected_label)
                            .map(|(_, fingerprint)| fingerprint)
                            .unwrap();

                        Some(fingerprint)
                    }

                    CipherKind::PASSPHRASE | CipherKind::AGE => {
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

                let mut envs_map;

                if envs_file.is_some() {
                    let file = envs_file.as_ref().unwrap();

                    if !Path::new(file).exists() {
                        return Err(AppError::Msg(format!("File '{}' does not exist", file)));
                    }

                    let mut file = std::fs::OpenOptions::new().read(true).open(file)?;

                    let mut buffer = String::new();
                    file.read_to_string(&mut buffer)?;

                    envs_map = utils::parse_envs_from_string(&buffer)?;

                    let default_options = (0..envs_map.len()).collect::<Vec<usize>>();
                    let selected_keys = prompts::multi_select_prompt(prompts::MultiSelectPromptOptions {
                        title:
                            "Select the environment variables you want to keep in your new profile:"
                                .to_string(),
                        options: envs_map.keys().cloned().collect(),
                        default_indices: Some(default_options),
                    })?;

                    envs_map.retain(|env| selected_keys.contains(&env.key));
                } else if envs.is_some() {
                    envs_map = EnvMap::new();

                    for env in envs.as_ref().unwrap() {
                        if (*env).contains('=') {
                            let mut parts = env.splitn(2, '=');

                            if let Some(key) = parts.next() {
                                if let Some(value) = parts.next() {
                                    envs_map
                                        .insert_from_key_value(key.to_string(), value.to_string());
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
                        envs_map.insert_from_key_value(env.to_string(), value);
                    }
                } else {
                    envs_map = EnvMap::new(); // the user created a profile without any envs
                }

                for env in envs_map.iter_mut() {
                    if *add_comments {
                        env.comment = Some(prompts::text_prompt(prompts::TextPromptOptions {
                            title: format!("Enter a comment for '{}':", env.key),
                            default: None,
                        })?);
                    }

                    if *add_expires {
                        env.expiration_date =
                            Some(prompts::date_prompt(prompts::DatePromptOptions {
                                title: format!("Select an expiration date for '{}':", env.key),
                                default: Some(Local::now().date_naive()),
                            })?);
                    }
                }

                ops::create_profile(
                    profile_name.to_string(),
                    description.clone(),
                    envs_map,
                    cipher,
                )?;

                success_msg!("Profile created");
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

                let mut set_envs = Vec::new();

                for env in envs {
                    if env.contains('=') {
                        let mut parts = env.splitn(2, '=');

                        if let Some(key) = parts.next() {
                            if let Some(value) = parts.next() {
                                set_envs
                                    .push(Env::from_key_value(key.to_string(), value.to_string()));
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

                    set_envs.push(Env::from_key_value(env.to_string(), prompt));
                }

                for mut env in set_envs {
                    if *add_comments {
                        env.comment = Some(prompts::text_prompt(prompts::TextPromptOptions {
                            title: format!("Enter a comment for '{}':", env.key),
                            default: None,
                        })?);
                    }

                    if *add_expires {
                        env.expiration_date =
                            Some(prompts::date_prompt(prompts::DatePromptOptions {
                                title: format!("Select an expiration date for '{}':", env.key),
                                default: Some(Local::now().date_naive()),
                            })?);
                    }

                    profile.envs.insert(env);
                }

                println!("{}", "Applying Changes".green());
                profile.save()?;
            }

            Command::Unset { profile_name, keys } => {
                let mut profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                ops::check_expired_envs(&profile);

                for key in keys {
                    profile.envs.remove(key)?;
                }

                println!("{}", "Applying Changes".green());
                profile.save()?;
            }

            Command::Load { profile_name } => {
                #[cfg(target_family = "unix")]
                {
                    ops::load_profile(profile_name)?;

                    let shell_config = utils::get_shell_config_path()?;

                    if shell_config.exists() {
                        success_msg!(
                            "Reload your shell to apply changes or run `source {}`",
                            utils::shorten_home(&shell_config)
                        );
                    } else {
                        success_msg!("Reload your shell to apply changes");
                    }
                }

                #[cfg(target_family = "windows")]
                {
                    let profile =
                        get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                    ops::load_profile(profile)?;

                    success_msg!("Restart your shell to apply changes");
                }
            }

            #[cfg(target_family = "unix")]
            Command::Unload => {
                ops::unload_profile()?;

                success_msg!("Restart your shell to apply changes");
            }

            #[cfg(target_family = "windows")]
            Command::Unload { profile_name } => {
                let profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;

                ops::unload_profile(profile)?;

                success_msg!("Restart your shell to apply changes");
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
                    .envs::<IndexMap<String, String>, _, _>(profile.envs.into())
                    .args(args)
                    .stdout(std::process::Stdio::inherit())
                    .stderr(std::process::Stdio::inherit())
                    .spawn()
                    .expect("Failed to execute command");

                let status = match cmd.wait() {
                    Ok(s) => s,
                    Err(e) => {
                        return Err(AppError::Msg(format!("Failed to execute command: {}", e)));
                    }
                };

                match status.code() {
                    Some(code) => std::process::exit(code),
                    None => {
                        return Err(AppError::Msg(
                            "The child process was terminated by a signal".to_string(),
                        ));
                    }
                }
            }

            Command::Delete { profile_name } => {
                ops::delete_profile(profile_name)?;
                success_msg!("Deleted profile");
            }

            Command::List { no_pretty_print } => {
                ops::list_profiles(*no_pretty_print)?;
            }

            Command::Show {
                profile_name,
                no_pretty_print,
                show_comments,
                show_expiration,
            } => {
                let profile =
                    get_profile(utils::get_profile_path(profile_name)?, Some(get_userkey))?;
                ops::check_expired_envs(&profile);

                if *no_pretty_print {
                    for env in profile.envs {
                        println!("{}={}", env.key, env.value);
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
                        let keys: Vec<String> = profile.envs.keys().cloned().collect();
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

                let output_file_path = output_file_path.as_deref().unwrap_or(".env");
                ops::export_envs(&profile, output_file_path, &envs_selected)?;

                success_msg!("Exported envs to {}", output_file_path);
            }

            Command::Import {
                source,
                profile_name,
            } => {
                let profile_name = profile_name.clone().unwrap_or_else(|| {
                    Path::new(source)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("imported")
                        .to_string()
                });

                if Url::parse(source).is_ok() {
                    ops::download_profile(source.to_string(), &profile_name)?;
                } else if Path::new(source).exists() {
                    ops::import_profile(source.to_string(), &profile_name)?;
                } else {
                    return Err(AppError::Msg(
                        "Source must be a valid file path or URL".to_string(),
                    ));
                }

                success_msg!("Imported profile");

                let location = utils::build_profile_path(&profile_name);

                let mut serialized_profile: SerializedProfile =
                    envio::utils::get_serialized_profile(&location)?;

                serialized_profile.metadata.name = profile_name;
                serialized_profile.metadata.file_path = location.clone();

                envio::utils::save_serialized_profile(&location, serialized_profile)?;
            }

            Command::Tui => {
                let mut terminal = ratatui::init();
                TuiApp::default()?.run(&mut terminal)?;
                ratatui::restore();
            }

            Command::Completion { shell } => match shell.as_str() {
                "bash" => println!("{}", completions::BASH_COMPLETION),
                "zsh" => println!("{}", completions::ZSH_COMPLETION),
                "fish" => println!("{}", completions::FISH_COMPLETION),
                "powershell" => println!("{}", completions::PS1_COMPLETION),
                _ => return Err(AppError::UnsupportedShell(shell.to_string())),
            },

            Command::Version { verbose } => {
                println!("{} {}", "Version".green(), env!("CARGO_PKG_VERSION"));

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
