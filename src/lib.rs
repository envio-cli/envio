//! `envio` is a library that simplifies the management of environment variables across multiple profiles
//!
//! It allows users to easily switch between different configurations and apply them to their current environment
//! `envio` also encrypts sensitive environment variable values to ensure secure storage and transmission of environment variables
//!
//! Here is a very simple example:
//! ```
//! // In this example we get the profile passed as an argument to the program
//! // and then print the environment variables in that profile
// In this example we get the profile passed as an argument to the program and then print the environment variables in that profile
//!let args: Vec<String> = std::env::args().collect();

//!if args.len() != 3 {
//!   println!("Usage: <profile_name> <key>");
//!   return;
//!}
//!
//!let profile_name = args[1].to_string();
//!let key = args[2].to_string(); // All profiles have a key that is used to encrypt the environment variables, this ensures that the environment variables are secure

//!// We use the age encryption type here
//!// If the profile was encrypted with a different encryption type you can use the encryption type that was used to encrypt the profile
//!// For example if the profile was encrypted with the GPG encryption type you would use the following line instead:
//!// let encryption_type = envio::crypto::create_encryption_type(key, "gpg"); -- Over here key would be the fingerprint of the GPG key used to encrypt the profile
//!let encryption_type = envio::crypto::create_encryption_type(key, "age");
//!
//!// print the environment variables in that profile
//! for (env_var, value) in &envio::get_profile(profile_name, encryption_type)
//!     .unwrap()
//!     .envs
//! {
//!     println!("{}: {}", env_var, value);
//! }
//! ```
//!

mod profile;
pub mod utils;

pub mod crypto;
pub mod error;
pub use profile::Profile; // Re-export Profile so that users don't have to use envio::profile::Profile

#[macro_export]
macro_rules! load {
    ($name:expr $(, $get_key:expr)?) => {
        (||->envio::error::Result<()> {
            use envio::Profile;
            use envio::crypto;
            use envio::utils;

            let encrypted_content = utils::get_profile_content($name)?;
            let mut encryption_type;

            match crypto::get_encryption_type(&encrypted_content) {
                Ok(t) => encryption_type = t,
                Err(e) => return Err(e.into()),
            }

            if encryption_type.as_string() == "age" {
                $(
                    let key = $get_key();
                    encryption_type.set_key(key);
                )?
            }

            let profile = match Profile::from_content($name, &encrypted_content, encryption_type) {
                Ok(profile) => profile,
                Err(e) => return Err(e.into()),
            };

            for (env_var, value) in &profile.envs {
                std::env::set_var(env_var, value);
            }

            Ok(())
         })()
    };
}
