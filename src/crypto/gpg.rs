/*
 * Note:
 * Crypto uses gpgme to interact with GPG on unix systems however gpgme isn't that well supported on Windows
 * So on Windows we use the gpg command line tool (gpg4win) to interact with GPG
*/
use std::io::Error;

use colored::Colorize;

#[cfg(target_family = "unix")]
use gpgme::{Context, Data, Protocol};

#[cfg(target_family = "windows")]
use regex::Regex;
#[cfg(target_family = "windows")]
use std::collections::VecDeque;
#[cfg(target_family = "windows")]
use std::io::Write;
#[cfg(target_family = "windows")]
use std::process::{Command, Stdio};

use crate::crypto::EncryptionType;

pub struct GPG {
    key_fingerprint: String,
}

impl EncryptionType for GPG {
    fn new(key_fingerprint: String) -> Self {
        GPG { key_fingerprint }
    }

    /*
     * For GPG key is the fingerprint of the GPG key to use
     */
    fn set_key(&mut self, key: String) {
        self.key_fingerprint = key;
    }

    fn get_key(&self) -> String {
        self.key_fingerprint.clone()
    }

    fn as_string(&self) -> &'static str {
        "gpg"
    }

    fn encrypt(&self, data: &str) -> Vec<u8> {
        // Unix specific code
        #[cfg(target_family = "unix")]
        {
            let mut ctx = match Context::from_protocol(Protocol::OpenPgp) {
                Ok(ctx) => ctx,
                Err(e) => {
                    println!("{}: {}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            let key = match ctx.get_key(&self.key_fingerprint) {
                Ok(key) => key,
                Err(e) => {
                    println!("{}: {}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            let mut encrypted_data = Vec::new();
            if let Err(e) = ctx.encrypt(Some(&key), data, &mut encrypted_data) {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            };

            encrypted_data
        }

        // Windows specific code
        #[cfg(target_family = "windows")]
        {
            let mut gpg_process = match Command::new("gpg")
                .arg("--recipient")
                .arg(&self.key_fingerprint)
                .arg("--encrypt")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(gpg_process) => gpg_process,
                Err(e) => {
                    println!("{}: {}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            let stdin = match gpg_process.stdin.as_mut() {
                Some(stdin) => stdin,
                None => {
                    println!("{}: Failed to open stdin", "Error".red());
                    std::process::exit(1);
                }
            };

            if let Err(e) = stdin.write_all(data.as_bytes()) {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }

            // Wait for the GPG process to finish and capture its output
            let output = match gpg_process.wait_with_output() {
                Ok(output) => output,
                Err(e) => {
                    println!("{}: {}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            output.stdout
        }
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<String, Error> {
        // Unix specific code
        #[cfg(target_family = "unix")]
        {
            let mut ctx = match Context::from_protocol(Protocol::OpenPgp) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Err(Error::new(std::io::ErrorKind::Other, e));
                }
            };

            let mut cipher = match Data::from_bytes(encrypted_data) {
                Ok(cipher) => cipher,
                Err(e) => {
                    return Err(Error::new(std::io::ErrorKind::Other, e));
                }
            };

            let mut plain = Vec::new();
            if let Err(e) = ctx.decrypt_and_verify(&mut cipher, &mut plain) {
                return Err(Error::new(std::io::ErrorKind::Other, e));
            };

            Ok(String::from_utf8_lossy(&plain).to_string())
        }

        // Windows specific code
        #[cfg(target_family = "windows")]
        {
            let mut gpg_process = match Command::new("gpg")
                .arg("--yes")
                .arg("--quiet")
                .arg("--decrypt")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
            {
                Ok(gpg) => gpg,
                Err(e) => {
                    return Err(Error::new(std::io::ErrorKind::Other, e));
                }
            };

            let stdin = match gpg_process.stdin.as_mut() {
                Some(stdin) => stdin,
                None => {
                    return Err(Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to open stdin",
                    ));
                }
            };

            if let Err(e) = stdin.write_all(encrypted_data) {
                return Err(Error::new(std::io::ErrorKind::Other, e));
            }

            let output = match gpg_process.wait_with_output() {
                Ok(output) => output,
                Err(e) => {
                    return Err(Error::new(std::io::ErrorKind::Other, e));
                }
            };

            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        }
    }
}

/*
 * Get the GPG keys available on the system
 * Unix specific code

 * @return Vec<(String, String)>: Vec of tuples containing the name and email of the key and the fingerprint
*/
#[cfg(target_family = "unix")]
pub fn get_gpg_keys() -> Vec<(String, String)> {
    let mut context = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut available_keys: Vec<(String, String)> = Vec::new();

    let keys = match context.keys() {
        Ok(keys) => keys,
        Err(e) => {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    };

    for key in keys.flatten() {
        if let Some(user_id) = key.user_ids().next() {
            let name = match user_id.name() {
                Ok(name) => name,
                Err(e) => {
                    println!("{}: {:?}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            let email = match user_id.email() {
                Ok(email) => email,
                Err(e) => {
                    println!("{}: {:?}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            let key_fingerprint = match key.fingerprint() {
                Ok(fingerprint) => fingerprint.to_string(),
                Err(e) => {
                    println!("{}: {:?}", "Error".red(), e);
                    std::process::exit(1);
                }
            };

            available_keys.push((format!("{} <{}>", name, email), key_fingerprint));
        }
    }

    available_keys
}

/*
 * Get the GPG keys available on the system
 * Windows specific code

 * @return Vec<(String, String)>: Vec of tuples containing the name and email of the key and the fingerprint
*/
#[cfg(target_family = "windows")]
pub fn get_gpg_keys() -> Option<Vec<(String, String)>> {
    let output = Command::new("gpg")
        .args(["--list-keys", "--keyid-format", "LONG"])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();

    if stdout.trim().is_empty() {
        return Some(vec![]);
    }

    let mut lines: VecDeque<_> = stdout.lines().collect();

    lines.pop_front()?;
    if lines
        .pop_front()?
        .bytes()
        .filter(|&b| b != b'-')
        .take(1)
        .count()
        > 0
    {
        return None;
    }

    let re_fingerprint = Regex::new(r"^[0-9A-F]{16,}$").unwrap();
    let re_user_id = Regex::new(r"^uid\s*\[[a-z ]+\]\s*(.*)$").unwrap();

    let mut available_keys: Vec<(String, String)> = Vec::new();
    while !lines.is_empty() {
        match lines.pop_front()? {
            l if l.starts_with("pub ") || l.starts_with("sec ") => {
                let fingerprint = format_fingerprint(lines.pop_front()?.trim());
                if !re_fingerprint.is_match(&fingerprint) {
                    return None;
                }

                let mut user_ids = Vec::new();
                while !lines.is_empty() {
                    match lines.pop_front()? {
                        l if l.starts_with("uid ") => {
                            let captures = re_user_id.captures(l)?;
                            user_ids.push(captures[1].to_string());
                        }

                        l if l.trim().is_empty() => break,

                        _ => {}
                    }
                }

                available_keys.push((user_ids[0].clone(), fingerprint))
            }

            l if l.trim().is_empty() => {}

            _ => return None,
        }
    }

    Some(available_keys)
}

/*
 * It formats the fingerprint by removing the spaces and converting it to uppercase
 * Windows specific code

 * @return String: formatted fingerprint
*/
#[cfg(target_family = "windows")]
fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
}
