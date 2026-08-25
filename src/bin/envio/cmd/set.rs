use chrono::Local;
use envio::Env;

use crate::{config, error::AppResult, profile_ops, prompts, success_msg};

pub fn run(
    profile_name: &str,
    envs: &[String],
    add_comments: bool,
    add_expires: bool,
) -> AppResult<()> {
    config::warn_if_global_store();
    let mut profile = profile_ops::get_profile_cli(profile_name)?;

    for mut env in parse_envs(profile_name, &profile, envs)? {
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

        profile.envs.insert(env);
    }

    profile.save()?;
    success_msg!("Changes applied");
    Ok(())
}

fn parse_envs(
    _profile_name: &str,
    profile: &envio::Profile,
    envs: &[String],
) -> AppResult<Vec<Env>> {
    let mut result = Vec::new();

    for raw in envs {
        if raw.contains('=') {
            let mut parts = raw.splitn(2, '=');
            let key = parts.next().unwrap_or("").to_string();
            let value = parts.next().unwrap_or("").to_string();
            result.push(Env::from_key_value(key, value));
        } else {
            let value = prompts::text_prompt(prompts::TextPromptOptions {
                title: format!(
                    "Enter the {}value for {}:",
                    if profile.envs.contains_key(raw) {
                        "new "
                    } else {
                        ""
                    },
                    raw
                ),
                default: None,
            })?;
            result.push(Env::from_key_value(raw.to_string(), value));
        }
    }

    Ok(result)
}
