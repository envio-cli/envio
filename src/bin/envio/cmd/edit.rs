use std::{io::Write, path::PathBuf};

use envio::{Env, EnvMap, Profile};

use crate::{
    config,
    error::{AppError, AppResult},
    error_msg, profile_ops, prompts, success_msg,
};

pub fn run(profile_name: &str) -> AppResult<()> {
    config::warn_if_global_store();
    let mut profile = profile_ops::get_profile_cli(profile_name)?;
    open_editor_loop(&mut profile)?;
    profile.save()?;
    success_msg!("Changes applied");
    Ok(())
}

fn format_profile_for_editing(profile_name: &str, envs: &EnvMap) -> String {
    let mut output = format!(
        r#"# Profile: {profile_name}
#
# Edit the variables below.
#
# Variable format:
#   KEY=VALUE
#
# Comments:
#   One or more lines starting with '#'
#   immediately before a variable.
#
# Expiration date:
#   # expires: YYYY-MM-DD
#   Place it immediately before the variable.
#
# Example:
#   # Database credentials
#   # Used by the production API
#   # expires: 2027-01-01
#   DATABASE_URL=postgres://...
#
# Delete a variable by removing its KEY=VALUE line.
# To remove a comment or expiration date, delete its corresponding '#' line(s).
# ---
"#
    );

    for env in envs.iter() {
        if let Some(comment) = &env.comment {
            for line in comment.lines() {
                output.push_str("# ");
                output.push_str(line);
                output.push('\n');
            }
        }
        if let Some(expires) = &env.expiration_date {
            output.push_str("# expires: ");
            output.push_str(&expires.format("%Y-%m-%d").to_string());
            output.push('\n');
        }
        output.push_str(&env.key);
        output.push('=');
        output.push_str(&env.value);
        output.push_str("\n\n");
    }

    output
}

fn parse_edited_profile(content: &str) -> AppResult<EnvMap> {
    let mut envs = EnvMap::default();
    let mut current_comments = Vec::new();
    let mut current_expiration = None;

    for (line_idx, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with('#') {
            if trimmed.starts_with("# --") {
                current_comments.clear();
                current_expiration = None;
                continue;
            }

            let comment_content = trimmed.trim_start_matches('#').trim();

            if comment_content.to_lowercase().starts_with("expires:") {
                let date_str = comment_content["expires:".len()..].trim();
                let date =
                    chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").map_err(|_| {
                        AppError::Msg(format!(
                            "Line {}: Invalid expiration date format '{}'. Expected YYYY-MM-DD",
                            line_idx + 1,
                            date_str
                        ))
                    })?;
                current_expiration = Some(date);
            } else {
                current_comments.push(comment_content.to_string());
            }

            continue;
        }

        if let Some(equal_pos) = trimmed.find('=') {
            let key = trimmed[..equal_pos].trim();
            let value = trimmed[equal_pos + 1..].trim();

            if key.is_empty() {
                return Err(AppError::Msg(format!(
                    "Line {}: Key cannot be empty in KEY=VALUE pair",
                    line_idx + 1
                )));
            }

            let comment = if current_comments.is_empty() {
                None
            } else {
                Some(current_comments.join("\n"))
            };

            envs.insert(Env::new(
                key.to_string(),
                value.to_string(),
                comment,
                current_expiration,
            ));

            current_comments.clear();
            current_expiration = None;
        } else {
            return Err(AppError::Msg(format!(
                "Line {}: Invalid format. Expected KEY=VALUE or a comment line starting with '#'",
                line_idx + 1
            )));
        }
    }

    if envs.is_empty() {
        return Err(AppError::Msg(
            "No environment variables found in the edited profile.".to_string(),
        ));
    }

    Ok(envs)
}

fn open_editor_loop(profile: &mut Profile) -> AppResult<()> {
    let editor = std::env::var("EDITOR").map_err(|_| {
        AppError::Msg(
            "EDITOR environment variable is not set. Please set it to your preferred text editor"
                .to_string(),
        )
    })?;

    let editor_parts: Vec<String> = editor.split_whitespace().map(|s| s.to_string()).collect();
    if editor_parts.is_empty() {
        return Err(AppError::Msg("No text editor found".to_string()));
    }
    let program = &editor_parts[0];
    let args = &editor_parts[1..];

    let initial_content = format_profile_for_editing(&profile.metadata.name, &profile.envs);

    let temp_dir = std::env::temp_dir();
    let temp_file_name = format!("envio-edit-{}.txt", uuid::Uuid::new_v4());
    let temp_file_path = temp_dir.join(&temp_file_name);

    let _guard = TempFileGuard {
        path: temp_file_path.clone(),
    };

    {
        let mut opts = std::fs::OpenOptions::new();
        opts.write(true).create(true).truncate(true);

        #[cfg(target_family = "unix")]
        {
            use std::os::unix::fs::OpenOptionsExt;
            opts.mode(0o600);
        }

        let mut file = opts.open(&temp_file_path)?;
        file.write_all(initial_content.as_bytes())?;
    }

    loop {
        let mut child = std::process::Command::new(program)
            .args(args)
            .arg(&temp_file_path)
            .stdin(std::process::Stdio::inherit())
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .map_err(|e| AppError::Msg(format!("Failed to spawn editor: {}", e)))?;

        let status = child
            .wait()
            .map_err(|e| AppError::Msg(format!("Failed to wait on editor: {}", e)))?;

        if !status.success() {
            return Err(AppError::Msg(format!(
                "Editor exited with error code: {}",
                status.code().unwrap_or(1)
            )));
        }

        let edited_content = std::fs::read_to_string(&temp_file_path)?;
        match parse_edited_profile(&edited_content) {
            Ok(new_envs) => {
                profile.envs = new_envs;
                return Ok(());
            }
            Err(e) => {
                error_msg!("{}", e);
                let choice = prompts::select_prompt(prompts::SelectPromptOptions {
                    title: "How would you like to proceed?".to_string(),
                    options: vec![
                        "Re-edit the file".to_string(),
                        "Abort (discard changes)".to_string(),
                    ],
                })?;

                if choice == "Abort (discard changes)" {
                    return Err(AppError::Msg("Edit aborted".to_string()));
                }
            }
        }
    }
}

struct TempFileGuard {
    path: PathBuf,
}

impl Drop for TempFileGuard {
    fn drop(&mut self) {
        if self.path.exists() {
            if let Ok(metadata) = std::fs::metadata(&self.path) {
                let len = metadata.len();
                if len > 0 {
                    let zeroes = vec![0u8; len as usize];
                    let _ = std::fs::write(&self.path, zeroes);
                }
            }
            let _ = std::fs::remove_file(&self.path);
        }
    }
}
