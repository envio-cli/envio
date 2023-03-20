mod cli;
mod commands;

use std::path::Path;

use clap::Parser;
use colored::Colorize;

use envio::utils;

use cli::Cli;

fn main() {
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

    if utils::check_for_updates() {
        println!("{}: New update available", "Update".green());
    }

    let args = Cli::parse();
    args.command.run();
}
