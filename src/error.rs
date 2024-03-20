use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Profile `{0}` already exists")]
    ProfileAlreadyExists(String),
    #[error("Profile `{0}` does not exist")]
    ProfileDoesNotExist(String),
    #[error("Profile `{0}` name is empty")]
    ProfileNameEmpty(String),
    #[error("Profile `{0}` already exists")]
    ProfileExists(String),
    #[error("Profile `{0}` is empty")]
    EmptyProfile(String),
    #[error("Environment variable `{0}` does not exist")]
    EnvDoesNotExist(String),
    #[error("Environment variable `{0}` already exists")]
    EnvExists(String),
    #[error("Crypto error: {0}")]
    Crypto(String),
    #[error("Invalid encryption type: {0}")]
    InvalidEncryptionType(String),
    #[error("Invalid UTF-8: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error("{0}")]
    Msg(String),
}

impl From<&'static str> for Error {
    fn from(s: &'static str) -> Self {
        Error::Msg(s.to_owned())
    }
}

impl From<String> for Error {
    fn from(s: String) -> Self {
        Error::Msg(s)
    }
}

/// A type alias for `Result<T, envio::error::Error>`.
pub type Result<T> = std::result::Result<T, Error>;
