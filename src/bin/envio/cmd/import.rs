use std::path::Path;

use envio::{profile::SerializedProfile, utils as envio_utils};
use url::Url;

use crate::{
    config::{self, build_profile_path},
    error::{AppError, AppResult},
    success_msg,
    utils::download_file,
};

pub fn run(source: &str, profile_name: Option<&str>) -> AppResult<()> {
    config::warn_if_global_store();
    let profile_name = profile_name.map(|s| s.to_string()).unwrap_or_else(|| {
        Path::new(source)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("imported")
            .to_string()
    });

    if Url::parse(source).is_ok() {
        download_profile(source, &profile_name)?;
    } else if Path::new(source).exists() {
        import_profile(source, &profile_name)?;
    } else {
        return Err(AppError::Msg(
            "Source must be a valid file path or URL".to_string(),
        ));
    }

    success_msg!("Imported profile");

    // patch the profile name stored in the metadata to match the target name
    let location = config::build_profile_path(&profile_name)?;
    let mut serialized: SerializedProfile = envio_utils::get_serialized_profile(&location)?;
    serialized.metadata.name = profile_name;
    envio_utils::save_serialized_profile(&location, serialized)?;

    Ok(())
}

fn download_profile(url: &str, profile_name: &str) -> AppResult<()> {
    let location = build_profile_path(profile_name)?;

    if location.exists() {
        return Err(AppError::ProfileExists(profile_name.to_owned()));
    }

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(download_file(url, &location))?;
    Ok(())
}

fn import_profile(file_path: &str, profile_name: &str) -> AppResult<()> {
    let path = Path::new(file_path);

    if !path.exists() {
        return Err(AppError::Msg(format!(
            "File `{}` does not exist",
            file_path
        )));
    }

    let contents = std::fs::read_to_string(path)?;
    let location = build_profile_path(profile_name)?;

    if location.exists() {
        return Err(AppError::ProfileExists(profile_name.to_owned()));
    }

    std::fs::write(location, contents)?;
    Ok(())
}
