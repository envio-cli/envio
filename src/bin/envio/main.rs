mod cli;
mod commands;

#[cfg(target_family = "unix")]
use std::io::Write;

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::Command;

use clap::Parser;
use colored::Colorize;
use semver::Version;

use envio::utils;

use cli::Cli;

/*
 * This function is used to get the latest version of envio from github

 @return Option<Version>
*/
fn get_latest_version() -> Option<Version> {
    if Command::new("git").arg("--version").output().is_err() {
        println!("{}: Git is not installed", "Error".red());
        return None;
    }

    let owner = "humblepenguinn";
    let repo = "envio";
    let output = Command::new("git")
        .arg("ls-remote")
        .arg(format!("https://github.com/{}/{}.git", owner, repo))
        .output()
        .unwrap();
    let reader = BufReader::new(output.stdout.as_slice());
    let mut latest_tag = None;

    for line in reader.lines().filter_map(|x| x.ok()) {
        let parts: Vec<_> = line.split('\t').collect();
        if parts.len() != 2 {
            continue;
        }
        let (ref_name, _) = (parts[1], parts[0]);
        if ref_name.starts_with("refs/tags/") {
            let tag = ref_name.trim_start_matches("refs/tags/").to_owned();
            latest_tag =
                latest_tag.map_or(Some(tag.clone()), |latest| Some(std::cmp::max(latest, tag)));
        }
    }

    if let Some(mut tag) = latest_tag {
        tag = tag.trim_start_matches('v').to_string();
        if let Ok(version) = Version::parse(&tag) {
            return Some(version);
        }
    }

    None
}

fn main() {
    let latest_version = if let Some(val) = get_latest_version() {
        val
    } else {
        println!("{}:  Failed to get latest version", "Error".red());
        println!(
            "{}: You can still use envio but won't be notified about new versions!",
            "Warning".yellow()
        );
        Version::parse("0.0.0").unwrap()
    };

    let current_version = if let Ok(val) = Version::parse(env!("BUILD_VERSION")) {
        val
    } else {
        println!("{}: Failed to parse current version", "Error".red());
        return;
    };

    if latest_version > current_version {
        println!(
            "{}: {} -> {}",
            "New version available".yellow(),
            current_version,
            latest_version
        );
    }

    if !Path::new(&utils::get_configdir()).exists() {
        println!("{}", "Creating config directory".bold());
        if let Err(e) = std::fs::create_dir(utils::get_configdir()) {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }

        if let Err(e) = std::fs::create_dir(utils::get_configdir().join("profiles")) {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    }

    #[cfg(target_family = "unix")]
    {
        if !Path::new(&utils::get_configdir().join("setenv.sh")).exists() {
            println!("{}", "Creating shellscript".bold());
            if let Err(e) = std::fs::write(utils::get_configdir().join("setenv.sh"), "") {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }

            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .append(true)
                .open(
                    utils::get_homedir().to_str().unwrap().to_owned()
                        + &format!("/{}", envio::get_shell_config()),
                )
                .unwrap();

            let buffer = if envio::get_shell_config().contains("fish") {
                println!(
                    "To use the shellscript properly you need to install the {}(https://github.com/edc/bass) plugin for fish",
                    "bass".bold()
                );
                format!(
                    "# envio DO NOT MODIFY\n bass source {}",
                    &utils::get_configdir().join("setenv.sh").to_str().unwrap()
                )
            } else {
                format!(
                    "#envio DO NOT MODIFY\n source {}",
                    &utils::get_configdir().join("setenv.sh").to_str().unwrap()
                )
            };

            if let Err(e) = writeln!(file, "{}", buffer) {
                println!("{}: {}", "Error".red(), e);
            }
        }
    }

    let args = Cli::parse();
    args.command.run();
}
