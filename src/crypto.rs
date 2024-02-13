/*
 * Note:
 * Crypto uses gpgme to interact with GPG on unix systems however gpgme isn't that well supported on Windows
 * So on Windows we use the gpg command line tool (gpg4win) to interact with GPG
*/
use std::{boxed::Box, io::Error};

#[cfg(target_family = "windows")]
use std::collections::VecDeque;

use std::io::{Read, Write};

#[cfg(target_family = "windows")]
use std::process::{Command, Stdio};

use age::secrecy::Secret;
use colored::Colorize;

#[cfg(target_family = "windows")]
use regex::Regex;

// Unix specific code
#[cfg(target_family = "unix")]
use gpgme::{Context, Data, Protocol};

use crate::utils::get_configdir;

/*
 * EncryptionType trait
 * This trait is used to implement different encryption methods
 * Note: AGE is not a real encryption method, but a wrapper around the age crate to use the same interface as the other encryption methods
*/
pub trait EncryptionType {
    /*
    * Set the key to use for encryption/decryption

    * @param key: key to use - for GPG it's the fingerprint of the key
    */
    fn set_key(&mut self, key: String);
    /*
    * Get the key used for encryption/decryption

    * @return String
    */
    fn get_key(&self) -> String;
    /*
    * Encrypt data

    * @param data: data to encrypt
    * @return encrypted data
    */
    fn encrypt(&self, data: &str) -> Vec<u8>;
    /*
     * Decrypt data
     */
    fn decrypt(&self, encrypted_data: &[u8]) -> Result<String, Error>;
    /*
     * Return the name of the encryption type
     */
    fn as_string(&self) -> &'static str;
}

/*
 * GPG encryption
*/
pub struct GPG {
    key_fingerprint: String,
}

impl EncryptionType for GPG {
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
 * It formats the fingerprint by removing the spaces and converting it to uppercase
 * Windows specific code

 * @return String: formatted fingerprint
*/
#[cfg(target_family = "windows")]
fn format_fingerprint<S: AsRef<str>>(fingerprint: S) -> String {
    fingerprint.as_ref().trim().to_uppercase()
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
 * AGE encryption - its not a real encryption type, but a wrapper around the age crate to keep the
 * same interface as the other encryption types
*/
pub struct AGE {
    key: String,
}

impl EncryptionType for AGE {
    fn set_key(&mut self, key: String) {
        self.key = key;
    }

    fn get_key(&self) -> String {
        self.key.clone()
    }

    fn as_string(&self) -> &'static str {
        "age"
    }

    fn encrypt(&self, data: &str) -> Vec<u8> {
        let encryptor = age::Encryptor::with_user_passphrase(Secret::new(self.key.to_owned()));

        let mut encrypted = vec![];
        let mut writer = match encryptor.wrap_output(&mut encrypted) {
            Ok(writer) => writer,
            Err(e) => {
                println!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }
        };

        if let Err(e) = writer.write_all(data.as_bytes()) {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        };

        if let Err(e) = writer.finish() {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        };

        encrypted
    }

    fn decrypt(&self, encrypted_data: &[u8]) -> Result<String, Error> {
        let decryptor = match age::Decryptor::new(encrypted_data).unwrap() {
            age::Decryptor::Passphrase(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted = vec![];
        let mut reader = match decryptor.decrypt(&Secret::new(self.key.to_owned()), None) {
            Ok(reader) => reader,
            Err(e) => {
                return Err(Error::new(std::io::ErrorKind::Other, e));
            }
        };

        if let Err(e) = reader.read_to_end(&mut decrypted) {
            return Err(Error::new(std::io::ErrorKind::Other, e));
        }

        String::from_utf8(decrypted).map_err(|e| Error::new(std::io::ErrorKind::Other, e))
    }
}

/*
 * Create an encryption type based on the string passed

 * @param key: String - the key to use for encryption/decryption (for GPG this is the fingerprint)
 * @param encryption_type_str: &str - the string to match against
 * @return Box<dyn EncryptionType>: the encryption type
*/
pub fn create_encryption_type(key: String, encryption_type_str: &str) -> Box<dyn EncryptionType> {
    match encryption_type_str {
        "age" => Box::new(AGE { key }),
        "gpg" => Box::new(GPG {
            key_fingerprint: key,
        }),
        _ => {
            println!("{}: Invalid encryption type", "Error".red());
            std::process::exit(1);
        }
    }
}

/*
 * Get the encryption type for a profile
 * This is used to get the encryption type for a profile when decrypting a file, so we know which
 * encryption type to use

 * @param profile_name: String - the name of the profile
 * @return Box<dyn EncryptionType>: the encryption type
*/
pub fn get_encryption_type(profile_name: String) -> Box<dyn EncryptionType> {
    let config_dir = get_configdir();
    let profile_dir = config_dir.join("profiles");

    let profile_file_path = profile_dir.join(profile_name + ".env");

    let mut file = match std::fs::File::open(&profile_file_path) {
        Ok(file) => file,
        Err(e) => {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    };

    let mut file_contents = Vec::new();
    file.read_to_end(&mut file_contents).unwrap();

    let gpg_instance = GPG {
        key_fingerprint: "".to_string(),
    };

    // If the file can be decrypted with GPG, then we use GPG, otherwise we use AGE
    if gpg_instance.decrypt(&file_contents).is_ok() {
        Box::new(gpg_instance)
    } else {
        Box::new(AGE {
            key: "".to_string(),
        })
    }
}
