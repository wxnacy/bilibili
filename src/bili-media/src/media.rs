use std::path::Path;

use anyhow::Result;

use serde::Deserialize;
use settings::Settings;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct SpliterSettings {
    // 多媒体目录
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub count: Option<usize>,
    pub suffix_parts: Option<Vec<String>>,
    pub screenshot_seconds: Option<Vec<u64>>,
    pub remove_parts: Option<Vec<(u64, u64)>>,
}

impl SpliterSettings {
    pub fn screenshot_seconds(&self) -> Vec<u64> {
        self.screenshot_seconds.clone().unwrap_or(vec![10, 20, 30, 300, 400, 500])
    }

    pub fn merge_with(&mut self, other: &SpliterSettings) {
        if other.season.is_some() {
            self.season = other.season;
        }
        if other.episode.is_some() {
            self.episode = other.episode;
        }
        if other.count.is_some() {
            self.count = other.count;
        }
        if other.suffix_parts.is_some() {
            self.suffix_parts = other.suffix_parts.clone();
        }
        if other.screenshot_seconds.is_some() {
            self.screenshot_seconds = other.screenshot_seconds.clone();
        }
        if other.remove_parts.is_some() {
            self.remove_parts = other.remove_parts.clone();
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Uploader {
    // 多媒体目录
    pub season: u16,
    pub episode: u16,
    pub dtime: String,
}


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct MediaSettings {
    pub title: String,
    pub uploaders: Vec<Uploader>,
    pub spliters: Option<Vec<SpliterSettings>>,
    pub spliter: Option<SpliterSettings>
}

impl MediaSettings {
    pub fn new(name: &str) -> Result<Self> {
        let path = Settings::media().join(format!("{}.toml", name));
        Self::from(path)
    }

    pub fn from<P: AsRef<Path>>(path: P) -> Result<Self> {
        let c = Settings::build_config(path)?;
        let s: Self = c.try_deserialize()?;
        Ok(s)
    }

    pub fn get_uploader(&self, season: u16, episode: u16) -> Option<&Uploader> {
        self.uploaders.iter().find(|x| x.season == season && x.episode == episode)
    }

    /// 获取分割信息，拼接主题剧集
    ///
    /// Examples
    ///
    /// ```
    /// use media::MediaSettings;
    ///
    /// let media = MediaSettings::from("examples/media.toml").unwrap();
    ///
    /// let spliter = media.get_spliter(3, 12).unwrap();
    /// assert_eq!(spliter.remove_parts, Some(vec![(0, 90)]));
    /// assert_eq!(spliter.screenshot_seconds, Some(vec![10, 20, 30]));
    /// assert_eq!(spliter.suffix_parts, Some(vec!["ipartment"]));
    /// ```
    pub fn get_spliter(&self, season: u16, episode: u16) -> Option<SpliterSettings> {
        if let Some(spliter) = &self.spliter {
            let mut spliter = spliter.clone();
            if let Some(spliters) = &self.spliters {
                if let Some(ep_spliter) = spliters.iter().find(|x| x.season == Some(season) && x.episode == Some(episode)) {
                    spliter.merge_with(ep_spliter);
                }
            }
            return Some(spliter);
        }
        None
    }

}
