use std::{
    fs::{File, create_dir_all},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

use chrono::{Duration, Utc};
use dirs::cache_dir;
use semver::Version;
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;

use crate::{error::AppResult, error_msg};

#[derive(Serialize, Deserialize)]
struct CacheData {
    version: String,
    last_update_time: chrono::DateTime<Utc>,
}

fn get_cache_file() -> AppResult<PathBuf> {
    let app_name = env!("CARGO_PKG_NAME");
    let dir = cache_dir().unwrap().join(app_name);

    if !dir.exists() {
        create_dir_all(&dir)?;
    }

    Ok(dir.join("cache.bin"))
}

fn load_cache() -> AppResult<CacheData> {
    let file = File::open(get_cache_file()?)?;
    let mut reader = BufReader::new(file);

    Ok(bincode::serde::decode_from_std_read(
        &mut reader,
        bincode::config::standard(),
    )?)
}

fn save_cache(data: &CacheData) -> AppResult<()> {
    let file = File::create(get_cache_file()?)?;

    let mut writer = BufWriter::new(file);
    bincode::serde::encode_into_std_write(data, &mut writer, bincode::config::standard())?;

    writer.flush()?;

    Ok(())
}

pub fn get_latest_version() -> AppResult<Version> {
    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(async_get_latest_version())
}

async fn async_get_latest_version() -> AppResult<Version> {
    let cache = load_cache().ok().unwrap_or(CacheData {
        version: "0.0.0".to_string(),
        last_update_time: Utc::now() - Duration::days(7),
    });

    if cache.last_update_time <= Utc::now() - Duration::days(7) {
        let latest_version = fetch_latest_version(&cache.version).await;

        let new_data = CacheData {
            version: latest_version.to_string(),
            last_update_time: Utc::now(),
        };

        save_cache(&new_data)?;
        return Ok(latest_version);
    }

    Ok(Version::parse(&cache.version)?)
}

async fn fetch_latest_version(fallback: &str) -> Version {
    fetch_from_github_api().await.unwrap_or_else(|| {
        error_msg!("Failed to get latest version");
        Version::parse(fallback).unwrap_or_else(|_| Version::parse("0.0.0").unwrap())
    })
}

async fn fetch_from_github_api() -> Option<Version> {
    #[derive(Deserialize)]
    struct Release {
        tag_name: String,
    }

    let url = "https://api.github.com/repos/humblepenguinn/envio/releases/latest";
    let client = reqwest::Client::new();

    let res = client
        .get(url)
        .header("User-Agent", "envio")
        .send()
        .await
        .ok()?;

    if !res.status().is_success() {
        return None;
    }

    let release: Release = res.json().await.ok()?;
    Version::parse(release.tag_name.trim_start_matches('v')).ok()
}
