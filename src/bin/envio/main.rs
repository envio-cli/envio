mod clap_app;
mod commands;
mod error;
mod ops;
mod output;
mod prompts;
mod tui;
mod utils;
mod version;

use clap::Parser;
use clap_app::ClapApp;
use output::{error, warning};
use semver::Version;
#[cfg(target_family = "unix")]
use utils::initalize_config;
use version::get_latest_version;

fn main() {
    color_eyre::install().unwrap();

    let latest_version = get_latest_version();

    let current_version = if let Ok(val) = Version::parse(env!("CARGO_PKG_VERSION")) {
        val
    } else {
        error("Failed to parse current version");
        "0.0.0".parse().unwrap()
    };

    if latest_version > current_version {
        warning(format!("{} -> {}", current_version, latest_version));
    }

    let app = ClapApp::parse();

    #[cfg(target_family = "unix")]
    if let Err(e) = initalize_config() {
        error(e);
    }

    if let Err(e) = app.run() {
        error(e);
    }
}
