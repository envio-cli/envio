use clap::CommandFactory;
use clap_complete::{generate_to, shells::*};
use std::{env, fs, path::PathBuf, process};

include!("src/bin/envio/cli.rs");

/*
 * Main function of the build script
*/
fn main() {
    let mut cmd = Cli::command();
    let app_name = cmd.get_name().to_string();

    let completions_dir = "completions";

    if let Err(e) = create_dir(completions_dir) {
        panic!("Error: {}", e);
    }

    if let Err(e) = generate_completions(&mut cmd, &app_name, completions_dir) {
        panic!("Error: {}", e);
    }

    let manpage_dir = "man";
    if let Err(e) = create_dir(manpage_dir) {
        panic!("Error: {}", e);
    }

    if let Err(e) = generate_manpages(cmd, manpage_dir) {
        panic!("Error: {}", e);
    }

    let build_timestamp: String = get_buildtimestamp();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
    println!("cargo:rustc-env=BUILD_VERSION={}", get_version());
}

/*
 * Generate manpages for the application

 @param cmd clap::Command
 @param out_dir &str
 @return std::io::Result<()>
*/
fn generate_manpages(cmd: clap::Command, out_dir: &str) -> std::io::Result<()> {
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer).unwrap();

    std::fs::write(PathBuf::from(out_dir).join("envio.1"), buffer)?;

    Ok(())
}

/*
 * Generate completions for the application

 @param cmd clap::Command
 @param app_name &str
 @param outdir &str
 @return std::io::Result<()>
*/
fn generate_completions(
    cmd: &mut clap::Command,
    app_name: &str,
    outdir: &str,
) -> std::io::Result<()> {
    generate_to(Bash, cmd, app_name, outdir)?;
    generate_to(Zsh, cmd, app_name, outdir)?;
    generate_to(Fish, cmd, app_name, outdir)?;
    generate_to(PowerShell, cmd, app_name, outdir)?;

    Ok(())
}

/*
 * Get the version of the application
 * It will try to get the version using git describe
 * If it fails it will use the version from the Cargo.toml file

 @return String
*/
fn get_version() -> String {
    let mut cmd = process::Command::new("git");

    cmd.arg("describe");
    cmd.arg("--abbrev=0");
    cmd.arg("--tags=0");

    if let Ok(status) = cmd.status() {
        if status.success() {
            if let Ok(output) = cmd.output() {
                return format!("{:?}", output);
            }
        }
    }

    println!("Error: Cannot get build version using `git` using CARGO_PKG_VERSION");
    env!("CARGO_PKG_VERSION").to_string()
}

/*
 * Get the build timestamp

 @return String
*/
fn get_buildtimestamp() -> String {
    return chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
}

/*
 * Auxiliary function
 * Create a directory if it does not exist

 @param dir_name &str
*/
fn create_dir(dir_name: &str) -> Result<(), std::io::Error> {
    fs::create_dir_all(dir_name)?;

    Ok(())
}
