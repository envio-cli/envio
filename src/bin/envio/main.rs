mod clap_app;
mod ops;
mod commands;
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
        println!("{}: Failed to parse current version", "Error".red());
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

    let args = ClapApp::parse();

    #[cfg(target_family = "unix")]
    if let Err(e) = initalize_config() {
        println!("{}: {}", "Error".red(), e);
    }

    if let Err(e) = args.command.run() {
        println!("{}: {}", "Error".red(), e);
        std::process::exit(1);
    }
}
