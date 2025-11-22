use std::path::Path;

use crate::{error::Result, profile::SerializedProfile};

pub fn get_serialized_profile<P: AsRef<Path>>(file_path: P) -> Result<SerializedProfile> {
    let file_content = std::fs::read(&file_path)?;

    Ok(serde_json::from_slice(&file_content)?)
}

pub fn save_serialized_profile<P: AsRef<Path>>(
    file_path: P,
    serialized_profile: SerializedProfile,
) -> Result<()> {
    let file = std::fs::OpenOptions::new()
        .write(true)
        .append(false)
        .truncate(true)
        .create(true)
        .open(&file_path)?;

    serde_json::to_writer_pretty(&file, &serialized_profile)?;

    Ok(())
}
