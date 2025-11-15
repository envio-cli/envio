use std::path::Path;

use crate::{
    error::{Error, Result},
    profile::SerializedProfile,
};

pub(crate) fn get_serialized_profile<P: AsRef<Path>>(file_path: P) -> Result<SerializedProfile> {
    let file_content = std::fs::read(&file_path)?;

    Ok(serde_json::from_slice(&file_content)
        .map_err(|e| Error::Deserialization(format!("Failed to parse profile JSON: {}", e)))?)
}
