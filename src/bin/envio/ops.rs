#[cfg(target_family = "windows")]
use std::process::Command;
use std::{
    io::{Read, Write},
    path::{Path, PathBuf},
};

use chrono::Local;
use colored::Colorize;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use envio::{cipher::Cipher, profile::ProfileMetadata, EnvMap, Profile};

#[cfg(target_family = "unix")]
use crate::utils::get_shell_config;
use crate::{
    error::{AppError, AppResult},
    output::{success, warning},
    utils::{
        contains_path_separator, download_file, get_configdir, get_cwd, get_profile_metadata,
        get_profile_path,
    },
};

pub fn create_profile(
    name: String,
    description: Option<String>,
    envs: EnvMap,
    cipher: Box<dyn Cipher>,
) -> AppResult<()> {
    let config_dir = get_configdir();
    let profile_dir = config_dir.join("profiles");

    if !profile_dir.exists() {
        println!(
            "{}",
            "Profiles directory does not exist creating it now..".bold()
        );
        std::fs::create_dir_all(&profile_dir).unwrap();
    }

    let profile_file_path = profile_dir.join(name.clone() + ".env");

    if profile_file_path.exists() {
        return Err(AppError::ProfileExists(name));
    }

    let metadata = ProfileMetadata {
        name,
        version: env!("CARGO_PKG_VERSION").to_string(),
        description,
        file_path: profile_file_path,
        cipher_kind: cipher.kind(),
        cipher_metadata: cipher.get_metadata(),
        created_at: Local::now(),
        updated_at: Local::now(),
    };

    Profile::new(metadata, cipher, envs).save()?;

    Ok(())
}

pub fn check_expired_envs(profile: &Profile) {
    for env in &profile.envs {
        if let Some(date) = env.expiration_date {
            if date <= Local::now().date_naive() {
                warning(format!("environment variable '{}' has expired", env.key));
            }
        }
    }
}

pub fn export_envs(
    profile: &Profile,
    output_file_path: &str,
    envs_selected: &Option<Vec<String>>,
) -> AppResult<()> {
    let path = if contains_path_separator(output_file_path) {
        PathBuf::from(output_file_path)
    } else {
        get_cwd().join(output_file_path)
    };

    if profile.envs.is_empty() {
        return Err(AppError::EmptyProfile(profile.metadata.name.clone()));
    }

    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;

    let envs_to_export: Vec<_> = match envs_selected {
        Some(selected) if !selected.is_empty() => selected
            .iter()
            .filter_map(|key| profile.envs.get(key))
            .collect(),
        _ => profile.envs.iter().collect(),
    };

    if envs_to_export.is_empty() {
        return Err(AppError::Msg("No envs to export".to_string()));
    }

    for env in envs_to_export {
        writeln!(file, "{}={}", env.key, env.value)?;
    }

    println!("{}", format!("Exported envs to {}", path.display()).bold());

    Ok(())
}

pub fn list_envs(profile: &Profile, show_comments: bool, show_expiration: bool) {
    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    let mut header = vec![
        Cell::new("Environment Variable").add_attribute(Attribute::Bold),
        Cell::new("Value").add_attribute(Attribute::Bold),
    ];

    if show_comments {
        header.push(Cell::new("Comment").add_attribute(Attribute::Bold));
    }

    if show_expiration {
        header.push(Cell::new("Expiration Date").add_attribute(Attribute::Bold));
    }

    table.set_header(header);

    let mut row;

    for env in &profile.envs {
        row = vec![env.key.clone(), env.value.clone()];

        if show_comments {
            row.push(env.comment.clone().unwrap_or_else(|| "".to_string()));
        }

        if show_expiration {
            row.push(
                env.expiration_date
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| "".to_string()),
            );
        }

        table.add_row(row);
    }

    println!("{table}");
}

pub fn delete_profile(profile_name: &str) -> AppResult<()> {
    std::fs::remove_file(get_profile_path(profile_name)?)?;
    success("Deleted profile");

    Ok(())
}

pub fn list_profiles(no_pretty_print: bool) -> AppResult<()> {
    let configdir = get_configdir();
    let profile_dir = configdir.join("profiles");

    if !profile_dir.exists() {
        return Err(AppError::Msg(
            "Profiles directory does not exist".to_string(),
        ));
    }

    let mut profiles = Vec::new();
    for entry in std::fs::read_dir(profile_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        match path.extension() {
            None => continue,
            Some(ext) => {
                if ext != "env" {
                    continue;
                }
            }
        }
        let profile_name = path.file_stem().unwrap().to_str().unwrap().to_owned();
        if profile_name.starts_with('.') {
            continue;
        }
        profiles.push(profile_name);
    }

    if no_pretty_print {
        if profiles.is_empty() {
            println!("{}", "No profiles found".bold());
            return Ok(());
        }

        for profile in profiles {
            println!(
                "{} - {}",
                profile,
                get_profile_metadata(&profile)?
                    .description
                    .unwrap_or("".to_string())
            );
        }
        return Ok(());
    }

    let mut table = Table::new();
    table.set_content_arrangement(ContentArrangement::Dynamic);

    table.set_header(vec![
        Cell::new("Name").add_attribute(Attribute::Bold),
        Cell::new("Description").add_attribute(Attribute::Bold),
        Cell::new("Created At").add_attribute(Attribute::Bold),
        Cell::new("Updated At").add_attribute(Attribute::Bold),
    ]);

    for profile in profiles {
        let metadata = get_profile_metadata(&profile)?;
        table.add_row(vec![
            &profile,
            &metadata.description.unwrap_or("".to_string()),
            &metadata.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            &metadata.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
        ]);
    }

    println!("{table}");
    Ok(())
}

pub fn download_profile(url: String, profile_name: &String) -> AppResult<()> {
    let configdir = get_configdir();

    let location = configdir
        .join("profiles")
        .join(format!("{}.env", profile_name));

    if location.exists() {
        return Err(AppError::ProfileExists(profile_name.clone()));
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(download_file(url.as_str(), location.to_str().unwrap()))?;

    success("Downloaded profile");

    Ok(())
}

pub fn import_profile(file_path: String, profile_name: &String) -> AppResult<()> {
    if !Path::new(&file_path).exists() {
        return Err(AppError::Msg(format!(
            "File `{}` does not exist",
            file_path
        )));
    }

    let configdir = get_configdir();

    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&file_path)
        .unwrap();

    let mut contents = String::new();
    file.read_to_string(&mut contents).unwrap();

    let location = configdir
        .join("profiles")
        .join(format!("{}.env", profile_name));

    if location.exists() {
        return Err(AppError::ProfileExists(profile_name.clone()));
    }

    std::fs::write(location, contents)?;

    success("Imported profile");

    Ok(())
}

#[cfg(target_family = "unix")]
pub fn create_shellscript(profile: &str) -> AppResult<()> {
    let configdir = get_configdir();
    let shellscript_path = configdir.join("setenv.sh");

    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .append(false)
        .open(shellscript_path)?;

    let shellscript = format!(
        r#"#!/bin/bash
# This script was generated by envio and should not be modified!

raw_output=$(envio profile show {p} --no-pretty-print)

if ! echo "$raw_output" | grep -q "="; then
    echo -e "\e[31mError: \e[0mFailed to load environment variables from profile '{p}'" >&2
    return 1 2>/dev/null || exit 1
fi

ENV_VARS=$(echo "$raw_output" | awk -F "=" '/^[^=]+=.+/ {{print}}')

while IFS= read -r line; do
    var="${{line%%=*}}"
    val="${{line#*=}}"
    export "$var"="$val"
done <<< "$ENV_VARS"
"#,
        p = profile,
    );

    file.write_all(shellscript.as_bytes())?;
    file.flush()?;
    file.sync_all()?;

    Ok(())
}

#[cfg(target_family = "unix")]
pub fn load_profile(profile_name: &str) -> AppResult<()> {
    get_profile_path(profile_name)?; // will error if the profile does not exist

    let shell_config = get_shell_config()?;

    create_shellscript(profile_name)?;

    if !shell_config.is_empty() {
        println!(
            "Reload your shell to apply changes or run `source {}`",
            format_args!("~/{}", shell_config)
        );
    } else {
        println!("Reload your shell to apply changes");
    }

    Ok(())
}

#[cfg(target_family = "windows")]
pub fn load_profile(profile: Profile) -> AppResult<()> {
    for env in profile.envs {
        let output = Command::new("setx").arg(&env.key).arg(&env.value).output();

        match output {
            Ok(output) => {
                if !output.status.success() {
                    return Err(AppError::Msg(format!(
                        "Failed to execute setx for environment variable: {} with value: {}",
                        env.key, env.value
                    )));
                }
            }
            Err(e) => {
                return Err(AppError::Msg(format!("{}", e)));
            }
        }
    }

    println!("Reload your shell to apply changes");
    Ok(())
}

#[cfg(target_family = "unix")]
pub fn unload_profile() -> AppResult<()> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .open(get_configdir().join("setenv.sh"))
        .unwrap();

    file.set_len(0)?;

    println!("Reload your shell to apply changes");
    Ok(())
}

#[cfg(target_family = "windows")]
pub fn unload_profile(profile: Profile) -> AppResult<()> {
    for key in profile.envs.keys() {
        let status = Command::new("REG")
            .arg("delete")
            .arg("HKCU\\Environment")
            .arg("/F")
            .arg("/V")
            .arg(&key)
            .status();

        match status {
            Ok(status) => {
                if !status.success() {
                    return Err(AppError::Msg(format!(
                        "Failed to delete environment variable: {}",
                        key
                    )));
                }
            }
            Err(e) => {
                return Err(AppError::Msg(format!("{}", e)));
            }
        }
    }

    println!("Reload your shell to apply changes");

    Ok(())
}
