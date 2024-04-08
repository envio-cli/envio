//! [`envio`](https://envio-cli.github.io/) is a modern and secure CLI tool that
//! simplifies the management of environment variables.
//!
//! It is recommended to first visit the [official
//! website](https://envio-cli.github.io/) and learn more about how `envio`
//! works before using the library or else some of the terms used in the
//! documentation won't make much sense.
//!
//! Along with the CLI tool, `envio` also provides a library that can be used to
//! integrate the functionality provided by envio into your own Rust projects.
//!
//! The library provides a simple API that allows you to easily load environment
//! variables from a profile and use them in your Rust programs. Here is a
//! simple example:
//!
//! ```rust
//!    envio::load!("my_profile");
//!    println!("{}", std::env::var("MY_ENV_VAR").unwrap());
//! ```
//!
//! This will load all the environment variables in the `my_profile` profile and
//! set them in the current environment. You can then use them in your program.
//!
//! The interface is similar to the `dotenv` crate, but `envio` provides a more
//! secure way to load these environment variables ensuring that they aren't
//! stored in plaintext.
//!
//! envio currently supports two encryption methods:
//! - `passkey`
//! - `gpg`
//!
//! The `passkey` method is also known as `age` method since it uses the `age`
//! encryption library.
//!
//! Depending on what encryption method you use, you will have to provide a key
//! when loading the profile. So if `my_profile` was encrypted using the `age`
//! method, you would have to provide a key when loading the profile:
//!
//! ```rust
//!   envio::load!("my_profile", || "mysecretkey".to_string()); // The load macro expects a closure that returns the key
//!   println!("{}", std::env::var("MY_ENV_VAR").unwrap());
//! ```
//!
//! For the `gpg` method, you only need to provide the name of the profile,
//! envio retrives the key fingerprint itself and uses it to decrypt the
//! profile.
//!
//! For more information on how envio's encryption process work, you can take a
//! look at the documentation for the [crypto](crate::crypto) module.
//!
//! 90% of the time, a user will only use the `load` macro to load profiles.
//! However, if you require more control or need to use some other
//! functionality, you can browse the documentation to see what fits your use
//! case.
//!
//! A suggestion would be to take a look at the
//! [Profile](crate::profile::Profile) struct and the
//! [load_profile](crate::load_profile) macro.
//!
//! The CLI tool makes use of the library and also has some additional
//! functionality which is specific to the CLI tool only. However the building
//! blocks were provided by the library.

mod profile;
pub mod utils;

pub mod crypto;
pub mod error;
pub use profile::Env;
pub use profile::EnvVec; // Re-export EnvVec so that users don't have to use envio::profile::EnvVec
pub use profile::Profile; // Re-export Profile so that users don't have to use envio::profile::Profile // Re-export Env so that users don't have to use envio::profile::Env

/// Main macro used to load profiles
///
/// It takes the name of the profile and an optional closure that returns the
/// key. This is only required if the profile was encrypted using the `age`
/// method.
///
/// # Example
///
/// ```rust
/// // Assuming the profile was encrypted using the `gpg` method
/// envio::load!("my_profile");
/// println!("{}", std::env::var("MY_ENV_VAR").unwrap());
/// ```
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
