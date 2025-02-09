use std::{ffi::OsStr, fs, path::PathBuf};

use settings::Settings;

pub fn create_cache_dir<T: AsRef<OsStr>>(name: T) -> anyhow::Result<PathBuf> {
    let name = name.as_ref().to_str().unwrap();
    let root = Settings::cache();
    let cache = root.join(format!("{}-{}", name, lazytool::current_timestamp()));
    if !cache.exists() {
        fs::create_dir_all(&cache)?;
    }
    Ok(cache)
}
