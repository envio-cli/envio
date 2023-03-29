mod cli;
mod commands;

use std::path::Path;

use clap::Parser;
use colored::Colorize;
use semver::Version;
use tokio::runtime::Builder;

use envio::utils;

use cli::Cli;

/*
 * This function is used to get the latest version of envio from github

 @return Option<Version>
*/
async fn get_latest_version() -> Option<Version> {
    let url = "https://api.github.com/repos/humblepenguinn/envio/releases/latest";
    let client = reqwest::Client::new();
    let res = if let Ok(val) = client.get(url).header("User-Agent", "envio").send().await {
        val
    } else {
        return None;
    };

    match res.status() {
        reqwest::StatusCode::OK => {
            let body = if let Ok(val) = res.text().await {
                val
            } else {
                return None;
            };

            if body.contains("tag_name") {
                let mut tag_name = body.split("tag_name").collect::<Vec<&str>>()[1]
                    .split('\"')
                    .collect::<Vec<&str>>()[2];

                tag_name = tag_name.trim_start_matches('v');
                let latest_version = if let Ok(val) = Version::parse(tag_name) {
                    val
                } else {
                    return None;
                };

                return Some(latest_version);
            }

            None
        }

        _ => None,
    }
}

fn main() {
    let rt = if let Ok(val) = Builder::new_current_thread().enable_all().build() {
        val
    } else {
        println!("{}: Failed to create runtime", "Error".red());
        return;
    };

    let latest_version = if let Some(val) = rt.block_on(get_latest_version()) {
        val
    } else {
        println!("{}:  Failed to get latest version", "Error".red());
        return;
    };

    let current_version = if let Ok(val) = Version::parse(env!("BUILD_VERSION")) {
        val
    } else {
        println!("{}: Failed to parse version", "Error".red());
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
        println!(
            "{}",
            "Config directory does not exist\nCreating config directory".bold()
        );
        if let Err(e) = std::fs::create_dir(utils::get_configdir()) {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    }

    let args = Cli::parse();
    args.command.run();
}
