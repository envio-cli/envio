use colored::Colorize;
use magic_crypt::{new_magic_crypt, MagicCryptTrait};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/*
 * Encrypt a string with a key
 */
pub fn encrypt(key: String, data: String) -> Vec<u8> {
    let mc = new_magic_crypt!(key, 256);

    mc.encrypt_str_to_bytes(data)
}

/*
 * Decrypt a string with a key
 */
pub fn decrypt(key: String, encrypted_data: &[u8]) -> String {
    let mc = new_magic_crypt!(key, 256);

    match mc.decrypt_bytes_to_bytes(encrypted_data) {
        Ok(bytes) => String::from_utf8(bytes).unwrap(),
        Err(e) => {
            println!("{}: {}", "Error".red(), e);
            std::process::exit(1);
        }
    }
}

/*
 * NOT USED YET
 * Hash a string
 */
pub fn hash_string(input: &str) -> String {
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}
