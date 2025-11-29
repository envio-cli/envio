use std::{fs, path::PathBuf};

use clap::CommandFactory;
use clap_complete::{Shell, generate_to};

include!("../src/bin/envio/clap_app.rs");

pub fn gen_man_and_comp() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = ClapApp::command();
    let app_name = cmd.get_name().to_string();

    let completions_dir = "completions";
    fs::create_dir_all(&completions_dir)?;

    for shell in &[Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell] {
        generate_to(*shell, &mut cmd, &app_name, &completions_dir)?;
    }

    let completions_base = PathBuf::from("../../../completions");
    println!(
        "cargo:rustc-env=ENVIO_GENERATED_COMPLETION_BASH={}",
        completions_base.join("envio.bash").display()
    );
    println!(
        "cargo:rustc-env=ENVIO_GENERATED_COMPLETION_FISH={}",
        completions_base.join("envio.fish").display()
    );
    println!(
        "cargo:rustc-env=ENVIO_GENERATED_COMPLETION_ZSH={}",
        completions_base.join("_envio").display()
    );
    println!(
        "cargo:rustc-env=ENVIO_GENERATED_COMPLETION_PS1={}",
        completions_base.join("_envio.ps1").display()
    );

    let manpage_dir = "man";
    fs::create_dir_all(&manpage_dir)?;
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    std::fs::write(format!("{}/envio.1", manpage_dir), buffer)?;

    Ok(())
}

pub fn export_build_env_vars() {
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

    let build_timestamp: String = chrono::Local::now()
        .naive_local()
        .format("%Y-%m-%d %H:%M:%S")
        .to_string();

    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", build_timestamp);
}
