mod cli;
mod commands;
mod version;

#[cfg(target_family = "unix")]
use std::io::Write;
use std::path::Path;

use clap::Parser;
use colored::Colorize;
use semver::Version;

use envio::utils;

use cli::Cli;
use version::get_latest_version;

fn main() {
    let latest_version = get_latest_version();

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
