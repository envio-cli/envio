use std::any::Any;
#[cfg(target_family = "windows")]
use std::collections::VecDeque;
#[cfg(target_family = "windows")]
use std::io::Write;
#[cfg(target_family = "windows")]
use std::process::{Command, Stdio};

#[cfg(target_family = "unix")]
use gpgme::{Context, Data, Protocol};
#[cfg(target_family = "windows")]
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::{
    EnvMap,
    cipher::{Cipher, CipherKind, EncryptedContent},
    error::{Error, Result},
};

#[derive(Serialize, Deserialize, Default, Clone)]
struct Metadata {
    key_fingerprint: String,
}

#[derive(Clone)]
pub struct GPG {
    metadata: Metadata,
}

impl GPG {
    pub fn new(key_fingerprint: String) -> Self {
        GPG {
            metadata: Metadata { key_fingerprint },
        }
    }

    pub fn set_key_fingerprint(&mut self, key_fingerprint: String) {
        self.metadata.key_fingerprint = key_fingerprint;
    }

    pub fn get_key_fingerprint(&self) -> String {
        self.metadata.key_fingerprint.clone()
    }
}

impl Cipher for GPG {
    fn kind(&self) -> CipherKind {
        CipherKind::GPG
    }

    fn encrypt(&mut self, envs: &EnvMap) -> Result<EncryptedContent> {
        let data = envs.as_bytes()?;

        let mut encrypted_data = Vec::new();

        #[cfg(target_family = "unix")]
        {
            let mut ctx = match Context::from_protocol(Protocol::OpenPgp) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Err(Error::Cipher(e.to_string()));
                }
            };

            let key = match ctx.get_key(&self.metadata.key_fingerprint) {
                Ok(key) => key,
                Err(e) => {
                    return Err(Error::Cipher(e.to_string()));
                }
            };

            if let Err(e) = ctx.encrypt(Some(&key), data, &mut encrypted_data) {
                return Err(Error::Cipher(e.to_string()));
            };
        }

        #[cfg(target_family = "windows")]
        {
            let mut gpg_process = Command::new("gpg")
                .arg("--recipient")
                .arg(&self.metadata.key_fingerprint)
                .arg("--encrypt")
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()?;

            let stdin = match gpg_process.stdin.as_mut() {
                Some(stdin) => stdin,
                None => {
                    return Err(Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "failed to open stdin",
                    )));
                }
            };

            stdin.write_all(&data)?;

            // Wait for the GPG process to finish and capture its output
            let output = gpg_process.wait_with_output()?;

            encrypted_data.extend_from_slice(&output.stdout);
        }

        Ok(EncryptedContent::Bytes(encrypted_data))
    }

    fn decrypt(&self, encrypted_data: &EncryptedContent) -> Result<EnvMap> {
        #[cfg(target_family = "unix")]
        {
            let mut ctx = match Context::from_protocol(Protocol::OpenPgp) {
                Ok(ctx) => ctx,
                Err(e) => {
                    return Err(Error::Cipher(e.to_string()));
                }
            };

            let mut cipher = match Data::from_bytes(encrypted_data.as_bytes()?) {
                Ok(cipher) => cipher,
                Err(e) => {
                    return Err(Error::Cipher(e.to_string()));
                }
            };

            let mut plain = Vec::new();
            if let Err(e) = ctx.decrypt_and_verify(&mut cipher, &mut plain) {
                return Err(Error::Cipher(e.to_string()));
            };

            Ok(plain.into())
        }

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
                    return Err(Error::Msg("failed to open stdin".to_string()));
                }
            };

            stdin.write_all(&encrypted_data.as_bytes()?)?;

            let output = gpg_process.wait_with_output()?;

            Ok(output.stdout.into())
        }
    }

    fn export_metadata(&self) -> Option<serde_json::Value> {
        serde_json::to_value(self.metadata.clone()).ok()
    }

    fn import_metadata(&mut self, data: serde_json::Value) -> Result<()> {
        self.metadata = serde_json::from_value(data)?;

        Ok(())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(target_family = "unix")]
pub fn get_gpg_keys() -> Result<Vec<(String, String)>> {
    let mut context = Context::from_protocol(Protocol::OpenPgp).unwrap();
    let mut available_keys: Vec<(String, String)> = Vec::new();

    let keys = match context.keys() {
        Ok(keys) => keys,
        Err(e) => {
            return Err(Error::Cipher(e.to_string()));
        }
    };

    for key in keys.flatten() {
        if let Some(user_id) = key.user_ids().next() {
            let name = match user_id.name() {
                Ok(name) => name,
                Err(e) => {
                    if e.is_none() {
                        return Err(Error::Cipher("Failed to get name from user id".to_string()));
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

#[cfg(target_family = "windows")]
pub fn get_gpg_keys() -> Result<Vec<(String, String)>> {
    let output = Command::new("gpg")
        .args(["--list-keys", "--keyid-format", "LONG"])
        .output()
        .map_err(|_| Error::Msg("Failed to execute GPG command".to_string()))?;

    let stdout = String::from_utf8(output.stdout)
        .map_err(|_| Error::Msg("Failed to parse GPG output as UTF-8".to_string()))?;

    if stdout.trim().is_empty() {
        return Ok(vec![]);
    }

    let mut lines: VecDeque<_> = stdout.lines().collect();

    lines.pop_front(); // remove first line
    if let Some(line) = lines.pop_front() {
        if line.bytes().filter(|&b| b != b'-').take(1).count() > 0 {
            return Err(Error::Msg("Unexpected GPG output format".to_string()));
        }
    }

    let re_fingerprint = Regex::new(r"^[0-9A-F]{16,}$").unwrap();
    let re_user_id = Regex::new(r"^uid\s*\[[a-z ]+\]\s*(.*)$").unwrap();

    let mut available_keys = Vec::new();

    while !lines.is_empty() {
        let line = lines.pop_front().unwrap();
        if line.starts_with("pub ") || line.starts_with("sec ") {
            let fingerprint_line = lines
                .pop_front()
                .ok_or_else(|| Error::Msg("Missing fingerprint".to_string()))?;
            let fingerprint = format_fingerprint(fingerprint_line.trim());

            if !re_fingerprint.is_match(&fingerprint) {
                return Err(Error::Msg("Invalid fingerprint format".to_string()));
            }

            let mut user_ids = Vec::new();
            while let Some(uid_line) = lines.front() {
                if uid_line.starts_with("uid ") {
                    let uid_line = lines.pop_front().unwrap();
                    let captures = re_user_id
                        .captures(&uid_line)
                        .ok_or_else(|| Error::Msg("Failed to parse user ID".to_string()))?;
                    user_ids.push(captures[1].to_string());
                } else if uid_line.trim().is_empty() {
                    lines.pop_front();
                    break;
                } else {
                    break;
                }
            }

            if let Some(first_user) = user_ids.get(0) {
                available_keys.push((first_user.clone(), fingerprint));
            }
        }
    }

    Ok(available_keys)
}

#[cfg(target_family = "windows")]
fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
}
