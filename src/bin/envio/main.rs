mod clap_app;
mod commands;
mod completions;
mod diagnostic;
mod error;
mod ops;
mod log_macros;
mod prompts;
mod tui;
mod utils;
#[cfg(not(debug_assertions))]
mod version;

use clap::Parser;

use clap_app::ClapApp;

use crate::error::AppResult;

#[cfg(target_family = "unix")]
pub fn initialize_config() -> AppResult<()> {
    use colored::Colorize;
    use inquire::Text;
    use std::fs;
    use std::io::Write;
    use std::path::PathBuf;

    use crate::utils::{get_configdir, get_shell_config_path};

    let configdir = get_configdir();
    if !configdir.exists() {
        println!("{}", "Creating config directory".bold());
        fs::create_dir(&configdir)?;
        fs::create_dir(configdir.join("profiles"))?;
    }

    let profile_dir = configdir.join("profiles");
    if !profile_dir.exists() {
        println!("{}", "Creating profile directory".bold());
        fs::create_dir(&profile_dir)?;
    }

    let shellscript_path = configdir.join("setenv.sh");

    if !shellscript_path.exists() {
        println!("{}", "Creating shellscript".bold());
        fs::write(&shellscript_path, "")?;

        let mut shellconfig_path = get_shell_config_path()?;

        if !shellconfig_path.exists() {
            let input = Text::new(
                "Shell config file not found, please enter the path to your shell config file:",
            )
            .prompt()?;

            shellconfig_path = PathBuf::from(&input);
        }

        let mut file = fs::OpenOptions::new()
            .append(true)
            .open(&shellconfig_path)?;

        let script_line = if shellconfig_path.to_str().unwrap().contains("fish") {
            println!(
                "To use the shellscript properly you need to install the {} (https://github.com/edc/bass) plugin for fish",
                "bass".bold()
            );
            format!(
                "\n# envio DO NOT MODIFY\nbass source {}\n",
                shellscript_path.display()
            )
        } else {
            format!(
                "\n# envio DO NOT MODIFY\nsource {}\n",
                shellscript_path.display()
            )
        };

        writeln!(file, "{script_line}")?;
    }

    Ok(())
}

#[cfg(not(debug_assertions))]
fn check_for_updates() -> AppResult<()> {
    use crate::version::get_latest_version;
    use semver::Version;

    let latest_version = get_latest_version()?;
    let current_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    if latest_version > current_version {
        warning_msg!("{} -> {}", current_version, latest_version);
    }

    Ok(())
}

fn run() -> AppResult<()> {
    #[cfg(not(debug_assertions))]
    check_for_updates()?;

    #[cfg(target_family = "unix")]
    initialize_config()?;

    ClapApp::parse().run()
}

fn main() {
    better_panic::Settings::debug()
        .message("Uh oh! Something went wrong!")
        .backtrace_first(false)
        .install();

    match run() {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            error_msg!(e);
            std::process::exit(1);
        }
    }
}
