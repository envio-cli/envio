use std::{io::Read, path::Path};

use chrono::Local;
use envio::{
    EnvMap,
    cipher::{CipherKind, create_cipher, gpg::get_gpg_keys},
};
use zeroize::Zeroizing;

use crate::{
    config,
    error::{AppError, AppResult},
    error_msg, profile_ops, prompts, success_msg, utils,
};

pub fn run(
    profile_name: &str,
    description: Option<&str>,
    envs: Option<&[String]>,
    envs_file: Option<&str>,
    cipher_kind: Option<&str>,
    add_comments: bool,
    add_expires: bool,
) -> AppResult<()> {
    config::warn_if_global_store();
    config::get_profile_dir()?;

    let selected_cipher_kind = if let Some(kind) = cipher_kind {
        kind.parse::<CipherKind>()
            .map_err(|e| AppError::Msg(e.to_string()))?
    } else {
        prompts::select_cipher_kind_prompt()?
    };

    let key = resolve_key(selected_cipher_kind)?;
    let cipher = create_cipher(selected_cipher_kind, key.clone())?;

    let mut envs_map = build_envs_map(envs, envs_file)?;

    annotate_envs(&mut envs_map, add_comments, add_expires)?;

    let profile = profile_ops::create_profile(
        profile_name.to_string(),
        description.map(|s| s.to_string()),
        envs_map,
        cipher,
    )?;

    if matches!(
        selected_cipher_kind,
        CipherKind::SYMMETRIC | CipherKind::PASSPHRASE
    ) {
        let store = prompts::confirm_prompt(prompts::ConfirmPromptOptions {
            title: "Do you want to securely store the encryption key in the system keyring?"
                .to_string(),
            default: Some(true),
        })
        .unwrap_or(false);

        if store
            && let Some(ref k) = key
            && let Ok(entry) = keyring::Entry::new("envio", &profile.metadata.uuid)
            && let Err(e) = entry.set_password(k)
        {
            error_msg!("Failed to store key in keyring: {}", e);
        }
    }

    success_msg!("Profile created");
    Ok(())
}

fn resolve_key(cipher_kind: CipherKind) -> AppResult<Option<Zeroizing<String>>> {
    match cipher_kind {
        CipherKind::GPG => {
            let available_keys = get_gpg_keys()?;

            if available_keys.is_empty() {
                return Err(AppError::Msg("No GPG keys found".to_string()));
            }

            if let Ok(env_key) = std::env::var("ENVIO_KEY") {
                if available_keys
                    .iter()
                    .any(|(_, fingerprint)| *fingerprint == env_key)
                {
                    return Ok(Some(env_key.into()));
                } else {
                    return Err(AppError::Msg(
                        "ENVIO_KEY does not match any available GPG fingerprint".to_string(),
                    ));
                }
            }

            let labels: Vec<String> = available_keys.iter().map(|(l, _)| l.clone()).collect();
            let selected_label = prompts::select_prompt(prompts::SelectPromptOptions {
                title: "Select the GPG key you want to use for encryption:".to_string(),
                options: labels,
            })?;

            let fingerprint = available_keys
                .into_iter()
                .find(|(label, _)| *label == selected_label)
                .map(|(_, fp)| fp)
                .unwrap();

            Ok(Some(fingerprint.into()))
        }

        CipherKind::PASSPHRASE => {
            let key = if let Ok(k) = std::env::var("ENVIO_KEY") {
                Zeroizing::new(k)
            } else {
                prompts::password_prompt(prompts::PasswordPromptOptions {
                    title: "Enter your encryption key:".to_string(),
                    help_message: Some(
                        "Remember this key, you will need it to decrypt your profile later"
                            .to_string(),
                    ),
                    min_length: Some(8),
                    with_confirmation: true,
                    confirmation_error_message: Some("The keys don't match".to_string()),
                })?
                .into()
            };
            Ok(Some(key))
        }

        CipherKind::SYMMETRIC => {
            let generated = envio::cipher::SYMMETRIC::generate_key();
            println!(
                "{} {}",
                colored::Colorize::bold(colored::Colorize::green("Generated Symmetric Key:")),
                colored::Colorize::bold(colored::Colorize::white(generated.as_str()))
            );
            println!(
                "{}",
                colored::Colorize::bold(colored::Colorize::red(
                    "Please store this key safely. It will not be shown again, and you need it to decrypt your profile!"
                ))
            );
            Ok(Some(generated))
        }

        _ => Ok(None),
    }
}

fn build_envs_map(envs: Option<&[String]>, envs_file: Option<&str>) -> AppResult<EnvMap> {
    if let Some(file) = envs_file {
        if !Path::new(file).exists() {
            return Err(AppError::Msg(format!("File '{}' does not exist", file)));
        }

        let mut file_handle = std::fs::OpenOptions::new().read(true).open(file)?;
        let mut buffer = String::new();
        file_handle.read_to_string(&mut buffer)?;

        let mut envs_map = utils::parse_envs_from_string(&buffer)?;

        let default_options = (0..envs_map.len()).collect::<Vec<usize>>();
        let selected_keys = prompts::multi_select_prompt(prompts::MultiSelectPromptOptions {
            title: "Select the environment variables you want to keep in your new profile:"
                .to_string(),
            options: envs_map.keys().cloned().collect(),
            default_indices: Some(default_options),
        })?;

        envs_map.retain(|env| selected_keys.contains(&env.key));
        return Ok(envs_map);
    }

    if let Some(envs) = envs {
        let mut envs_map = EnvMap::default();

        for env in envs {
            if env.contains('=') {
                let mut parts = env.splitn(2, '=');
                let key = parts
                    .next()
                    .ok_or_else(|| AppError::Msg(format!("Unable to parse key from '{}'", env)))?;
                let value = parts.next().ok_or_else(|| {
                    AppError::Msg(format!("Unable to parse value for key '{}'", key))
                })?;
                envs_map.insert_from_key_value(key.to_string(), value.to_string());
            } else {
                let value = prompts::text_prompt(prompts::TextPromptOptions {
                    title: format!("Enter the value for {}:", env),
                    default: None,
                })?;
                envs_map.insert_from_key_value(env.to_string(), value);
            }
        }

        return Ok(envs_map);
    }

    Ok(EnvMap::default())
}

fn annotate_envs(envs_map: &mut EnvMap, add_comments: bool, add_expires: bool) -> AppResult<()> {
    for env in envs_map.iter_mut() {
        if add_comments {
            env.comment = Some(prompts::text_prompt(prompts::TextPromptOptions {
                title: format!("Enter a comment for '{}':", env.key),
                default: None,
            })?);
        }

        if add_expires {
            env.expiration_date = Some(prompts::date_prompt(prompts::DatePromptOptions {
                title: format!("Select an expiration date for '{}':", env.key),
                default: Some(Local::now().date_naive()),
            })?);
        }
    }

    Ok(())
}
