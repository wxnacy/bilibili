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

pub fn get_episode_path(
    type_: &str,
    name: &str,
    season: u16,
    episode: u16,
) -> PathBuf {
    let path = format!(
        "/Volumes/ZhiTai/Movies/{}/{}/{}{}/S{:02}E{:02}.mp4",
        type_,
        name,
        name,
        season,
        season,
        episode,
    );
    PathBuf::from(path)
}

pub fn get_episode_name(
    name: &str,
    season: u16,
    episode: u16,
) -> String {
    String::from(format!("{}S{:02}E{:02}", name, season, episode))
}
