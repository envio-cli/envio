use std::{fs, path::PathBuf};

use clap::CommandFactory;
use clap_complete::{generate_to, shells::*};

include!("src/bin/envio/clap_app.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = ClapApp::command();
    let app_name = cmd.get_name().to_string();

    let completions_dir = "completions";
    fs::create_dir_all(completions_dir)?;
    generate_completions(&mut cmd, &app_name, completions_dir)?;

    let manpage_dir = "man";
    fs::create_dir_all(manpage_dir)?;
    generate_manpages(cmd, manpage_dir)?;

    let build_timestamp: String = get_buildtimestamp();
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);

    export_build_env_vars();

    Ok(())
}

fn generate_manpages(cmd: clap::Command, out_dir: &str) -> std::io::Result<()> {
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;

    std::fs::write(PathBuf::from(out_dir).join("envio.1"), buffer)?;

    Ok(())
}

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

fn get_buildtimestamp() -> String {
    return chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();
}

fn export_build_env_vars() {
    for var in &[
        "PROFILE",
        "TARGET",
        "CARGO_CFG_TARGET_FAMILY",
        "CARGO_CFG_TARGET_OS",
        "CARGO_CFG_TARGET_ARCH",
        "CARGO_CFG_TARGET_POINTER_WIDTH",
        "CARGO_CFG_TARGET_ENDIAN",
        "CARGO_CFG_TARGET_FEATURE",
        "HOST",
    ] {
        println!(
            "cargo:rustc-env={}={}",
            var,
            std::env::var(var).unwrap_or_else(|_| "unknown".into())
        );
    }
}
