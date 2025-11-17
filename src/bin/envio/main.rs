mod clap_app;
mod commands;
mod ops;
mod prompts;
mod utils;
mod version;

use clap::Parser;
use clap_app::ClapApp;
use colored::Colorize;
use semver::Version;
#[cfg(target_family = "unix")]
use utils::initalize_config;
use version::get_latest_version;

fn main() {
    color_eyre::install().unwrap();

    let latest_version = get_latest_version();

    let current_version = if let Ok(val) = Version::parse(env!("BUILD_VERSION")) {
        val
    } else {
        eprintln!("{}: Failed to parse current version", "Error".red());
        "0.0.0".parse().unwrap()
    };

    if latest_version > current_version {
        println!(
            "{}: {} -> {}",
            "New version available".yellow(),
            current_version,
            latest_version
        );
    }

    let app = ClapApp::parse();

    #[cfg(target_family = "unix")]
    if let Err(e) = initalize_config() {
        eprintln!("{}: {}", "Error".red(), e);
    }

    if let Err(e) = app.run() {
        eprintln!("{}: {}", "Error".red(), e);
    }
}
