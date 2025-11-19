use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AppError {
    #[error(transparent)]
    Library(#[from] envio::error::Error),

    #[error("profile `{0}` does not exist")]
    ProfileDoesNotExist(String),

    #[error("profile name is empty")]
    ProfileNameEmpty,

    #[error("profile `{0}` already exists")]
    ProfileExists(String),

    #[error("profile `{0}` is empty")]
    EmptyProfile(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Prompt(#[from] inquire::InquireError),

    #[error(transparent)]
    VersionParse(#[from] semver::Error),

    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    #[error("{0}")]
    Msg(String),
}

pub type AppResult<T> = std::result::Result<T, AppError>;
