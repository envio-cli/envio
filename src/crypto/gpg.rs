use serde::{Deserialize, Serialize};

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
use crate::error::{Error, Result};

// Bytes that identify the file as being encrypted using the `gpg` method
pub const IDENTITY_BYTES: &[u8] = b"-----GPG ENCRYPTED FILE-----";

#[derive(Serialize, Deserialize)]
pub struct GPG {
    key_fingerprint: String,
}

#[typetag::serde]
impl EncryptionType for GPG {
    fn new(key_fingerprint: String) -> Self {
        GPG { key_fingerprint }
    }

    fn set_key(&mut self, key: String) {
        self.key_fingerprint = key;
    }

    fn get_key(&self) -> String {
        self.key_fingerprint.clone()
    }

    fn as_string(&self) -> &'static str {
        "gpg"
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let mut encrypted_data = Vec::new();

        // Unix specific code
        #[cfg(target_family = "unix")]
        {
            let mut ctx = match Context::from_protocol(Protocol::OpenPgp) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Err(Error::Crypto(e.to_string()));
                }
            };

            let key = match ctx.get_key(&self.key_fingerprint) {
                Ok(key) => key,
                Err(e) => {
                    return Err(Error::Crypto(e.to_string()));
                }
            };

            if let Err(e) = ctx.encrypt(Some(&key), data, &mut encrypted_data) {
                return Err(Error::Crypto(e.to_string()));
            };

            encrypted_data.extend_from_slice(IDENTITY_BYTES);
        }

        // Windows specific code
        #[cfg(target_family = "windows")]
        {
            let mut gpg_process = Command::new("gpg")
                .arg("--recipient")
                .arg(&self.key_fingerprint)
                .arg("--encrypt")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            let stdin = match gpg_process.stdin.as_mut() {
                Some(stdin) => stdin,
                None => {
                    return Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to open stdin",
                    )));
                }
            };

            stdin.write_all(data.as_bytes())?;

            // Wait for the GPG process to finish and capture its output
            let output = gpg_process.wait_with_output()?;

            encrypted_data.extend_from_slice(&output.stdout);
            encrypted_data.extend_from_slice(IDENTITY_BYTES);
        }

        Ok(encrypted_data)
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<Vec<u8>> {
        // Unix specific code
        #[cfg(target_family = "unix")]
        {
            let mut ctx = match Context::from_protocol(Protocol::OpenPgp) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Err(Error::Crypto(e.to_string()));
                }
            };

            let mut cipher = match Data::from_bytes(encrypted_data) {
                Ok(cipher) => cipher,
                Err(e) => {
                    return Err(Error::Crypto(e.to_string()));
                }
            };

            let mut plain = Vec::new();
            if let Err(e) = ctx.decrypt_and_verify(&mut cipher, &mut plain) {
                return Err(Error::Crypto(e.to_string()));
            };

            Ok(plain)
        }

        // Windows specific code
        #[cfg(target_family = "windows")]
        {
            let mut gpg_process = Command::new("gpg")
                .arg("--yes")
                .arg("--quiet")
                .arg("--decrypt")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            let stdin = match gpg_process.stdin.as_mut() {
                Some(stdin) => stdin,
                None => {
                    return Err(Error::Msg("Failed to open stdin".to_string()));
                }
            };

            stdin.write_all(encrypted_data)?;

            let output = gpg_process.wait_with_output()?;

            Ok(output.stdout)
        }
    }

    fn is_this_type(encrypted_data: &[u8]) -> bool {
        encrypted_data.len() >= IDENTITY_BYTES.len()
            && &encrypted_data[encrypted_data.len() - IDENTITY_BYTES.len()..] == IDENTITY_BYTES
    }
}

/// Get the GPG keys available on the system
///
/// There are two different implementations for Unix and Windows.
///
/// The Unix implementation uses the gpgme crate to access the GPG keys on the
/// system. The Windows implementation uses the gpg command line tool to access
///
/// # Returns
/// - `Result<Vec<(String, String)>>`: Vec of tuples containing a formatted
///   string of the user id with the email and the key fingerprint
///
/// # Example
///
/// ```rust
/// use envio::crypto::gpg::get_gpg_keys;
///
/// let keys = get_gpg_keys().unwrap();
///
/// for key in keys {
///    println!("{}: {}", key.0, key.1);
/// }
///
/// ```
#[cfg(target_family = "unix")]
pub fn get_gpg_keys() -> Result<Vec<(String, String)>> {
    let mut context = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut available_keys: Vec<(String, String)> = Vec::new();

    let keys = match context.keys() {
        Ok(keys) => keys,
        Err(e) => {
            return Err(Error::Crypto(e.to_string()));
        }
    };

    for key in keys.flatten() {
        if let Some(user_id) = key.user_ids().next() {
            let name = match user_id.name() {
                Ok(name) => name,
                Err(e) => {
                    if e.is_none() {
                        return Err(Error::Crypto("Failed to get name from user id".to_string()));
                    }

                    return Err(Error::Utf8Error(e.unwrap()));
                }
            };

            let email = match user_id.email() {
                Ok(email) => email,
                Err(e) => {
                    return Err(Error::Utf8Error(e.unwrap()));
                }
            };

            let key_fingerprint = match key.fingerprint() {
                Ok(fingerprint) => fingerprint.to_string(),
                Err(e) => {
                    return Err(Error::Utf8Error(e.unwrap()));
                }
            };

            available_keys.push((format!("{} <{}>", name, email), key_fingerprint));
        }
    }

    Ok(available_keys)
}

/// Windows specific implementation of getting the GPG keys on the system
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

/// Utility function to format the fingerprint.
/// Windows specific code
#[cfg(target_family = "windows")]
fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
}
