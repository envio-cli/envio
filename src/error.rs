use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    BincodeEncode(#[from] bincode::error::EncodeError),

    #[error(transparent)]
    BincodeDecode(#[from] bincode::error::DecodeError),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("environment variable `{0}` does not exist")]
    EnvDoesNotExist(String),

    #[error("{0}")]
    Cipher(String),

    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),

    #[error("{0}")]
    Msg(String),
}

pub type Result<T> = std::result::Result<T, Error>;
