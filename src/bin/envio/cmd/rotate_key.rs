use envio::cipher::{CipherKind, create_cipher, gpg::get_gpg_keys};
use zeroize::Zeroizing;

use crate::{
    config,
    error::{AppError, AppResult},
    error_msg, profile_ops, prompts, success_msg,
};

pub fn run(profile_name: &str) -> AppResult<()> {
    config::warn_if_global_store();
    let mut profile = profile_ops::get_profile_cli(profile_name)?;
    let cipher_kind = profile.metadata.cipher_kind;

    let new_key = match cipher_kind {
        CipherKind::GPG => {
            let available_keys = get_gpg_keys()?;

            if available_keys.is_empty() {
                return Err(AppError::Msg("No GPG keys found".to_string()));
            }

            let labels: Vec<String> = available_keys.iter().map(|(l, _)| l.clone()).collect();
            let selected_label = prompts::select_prompt(prompts::SelectPromptOptions {
                title: "Select the new GPG key you want to use for encryption:".to_string(),
                options: labels,
            })?;

            let fingerprint = available_keys
                .into_iter()
                .find(|(label, _)| *label == selected_label)
                .map(|(_, fp)| fp)
                .ok_or_else(|| AppError::Msg("Failed to resolve selected GPG key".to_string()))?;

            Zeroizing::new(fingerprint)
        }

        CipherKind::PASSPHRASE => {
            let key = prompts::password_prompt(prompts::PasswordPromptOptions {
                title: "Enter your new encryption key:".to_string(),
                help_message: Some(
                    "Remember this key, you will need it to decrypt your profile later".to_string(),
                ),
                min_length: Some(8),
                with_confirmation: true,
                confirmation_error_message: Some("The keys don't match".to_string()),
            })?;
            Zeroizing::new(key)
        }

        CipherKind::SYMMETRIC => {
            let generated = envio::cipher::SYMMETRIC::generate_key();
            println!(
                "{} {}",
                colored::Colorize::bold(colored::Colorize::green("Generated New Symmetric Key:")),
                colored::Colorize::bold(colored::Colorize::white(generated.as_str()))
            );
            println!(
                "{}",
                colored::Colorize::bold(colored::Colorize::red(
                    "Please store this key safely. It will not be shown again, and you need it to decrypt your profile!"
                ))
            );
            generated
        }

        _ => {
            return Err(AppError::Msg(format!(
                "Cipher type '{}' does not support key rotation",
                cipher_kind
            )));
        }
    };

    profile.cipher = create_cipher(cipher_kind, Some(new_key.clone()))?;
    profile.save()?;

    if matches!(cipher_kind, CipherKind::SYMMETRIC | CipherKind::PASSPHRASE) {
        let store = prompts::confirm_prompt(prompts::ConfirmPromptOptions {
            title: "Do you want to securely store the new encryption key in the system keyring?"
                .to_string(),
            default: Some(true),
        })
        .unwrap_or(false);

        if let Ok(entry) = keyring::Entry::new("envio", &profile.metadata.uuid) {
            if store {
                if let Err(e) = entry.set_password(&new_key) {
                    error_msg!("Failed to store key in keyring: {}", e);
                }
            } else {
                let _ = entry.delete_credential();
            }
        }
    }

    success_msg!(
        "Encryption key for profile '{}' has been rotated",
        profile_name
    );
    Ok(())
}
