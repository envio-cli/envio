mod clap_app;
mod cli;
mod commands;
mod utils;
mod version;

#[cfg(target_family = "unix")]
use std::io::Write;
#[cfg(target_family = "unix")]
use std::path::{Path, PathBuf};

use clap::Parser;
use colored::Colorize;
#[cfg(target_family = "unix")]
use inquire::Text;
use semver::Version;

use clap_app::ClapApp;
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

    #[cfg(target_family = "unix")]
    {
        let configdir = match utils::get_configdir() {
            Ok(val) => val,
            Err(e) => {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }
        };

        let homedir = match utils::get_homedir() {
            Ok(val) => val,
            Err(e) => {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }
        };

        if !Path::new(&configdir).exists() {
            println!("{}", "Creating config directory".bold());
            if let Err(e) = std::fs::create_dir(&configdir) {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }

            if let Err(e) = std::fs::create_dir(configdir.join("profiles")) {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }
        }

        if !Path::new(&configdir.join("setenv.sh")).exists() {
            println!("{}", "Creating shellscript".bold());
            if let Err(e) = std::fs::write(configdir.join("setenv.sh"), "") {
                println!("{}: {}", "Error".red(), e);
                if let Err(e) = std::fs::remove_dir_all(&configdir) {
                    println!("{}: {}", "Error".red(), e);
                    std::process::exit(1);
                }

                std::process::exit(1);
            }

            let shellconfig = match utils::get_shell_config() {
                Ok(val) => val,
                Err(e) => {
                    println!("{}: {}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            let mut file_path = PathBuf::from(
                &(homedir.to_str().unwrap().to_owned() + &format!("/{}", shellconfig)),
            );
            if !file_path.exists() {
                let input = Text::new(
                    "Shell config file not found, please enter the path to your shell config file:",
                )
                .prompt();

                file_path = if let Ok(val) = input {
                    PathBuf::from(val)
                } else {
                    println!("{}: {}", "Error".red(), input.err().unwrap());
                    if let Err(e) = std::fs::remove_dir_all(&configdir) {
                        println!("{}: {}", "Error".red(), e);
                        std::process::exit(1);
                    }
                    std::process::exit(1);
                };

                if !file_path.exists() {
                    println!(
                        "{}: Specified shell config file does not exist either!?",
                        "Error".red()
                    );

                    if let Err(e) = std::fs::remove_dir_all(&configdir) {
                        println!("{}: {}", "Error".red(), e);
                        std::process::exit(1);
                    }

                    std::process::exit(1);
                }
            }

            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .open(file_path)
                .unwrap();

            let shellscript_path = &configdir.join("setenv.sh");

            let buffer = if shellconfig.contains("fish") {
                println!(
                    "To use the shellscript properly you need to install the {}(https://github.com/edc/bass) plugin for fish",
                    "bass".bold()
                );
                format!(
                    "
# envio DO NOT MODIFY
bass source {}
",
                    shellscript_path.to_str().unwrap()
                )
            } else {
                format!(
                    "
#envio DO NOT MODIFY
source {}
",
                    shellscript_path.to_str().unwrap()
                )
            };

            if let Err(e) = writeln!(file, "{}", buffer) {
                println!("{}: {}", "Error".red(), e);
            }
        }
    }

    let args = ClapApp::parse();

    if let Err(e) = args.command.run() {
        println!("{}: {}", "Error".red(), e);
    }
}
