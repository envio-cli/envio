use std::path::Path;

use crate::{error::Result, profile::SerializedProfile};

pub fn get_serialized_profile<P: AsRef<Path>>(file_path: P) -> Result<SerializedProfile> {
    let file_content = std::fs::read(&file_path)?;

    Ok(serde_json::from_slice(&file_content)?)
}
