use std::path::{Path, PathBuf};

use anyhow::Result;

use serde::Deserialize;
use settings::Settings;

#[derive(Debug, Clone, Deserialize)]
#[allow(unused)]
pub struct MarkSettings {
    // 多媒体目录
    pub id: String,
    pub title: String,
    pub path: Option<PathBuf>,
    pub parts: Option<Vec<String>>,
    pub suffix_parts: Option<Vec<String>>,
    pub with_suffix: Option<bool>,
    pub exclude_segments: Option<Vec<(u64, u64)>>,
    pub include_segments: Option<Vec<(u64, u64)>>,
    pub trans_1080p: Option<bool>,
}

impl MarkSettings {
    pub fn with_suffix(&self) -> bool {
        match &self.with_suffix {
            Some(p) => *p,
            None => true,
        }
    }

    pub fn trans_1080p(&self) -> bool {
        match &self.trans_1080p {
            Some(p) => *p,
            None => false,
        }
    }
}

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
    pub name: String,
    pub title: String,
    pub media_dir: Option<String>,
    pub suffix_parts: Option<Vec<String>>,
    pub uploaders: Vec<Uploader>,
    pub spliters: Option<Vec<SpliterSettings>>,
    pub spliter: Option<SpliterSettings>,
    pub marks: Option<Vec<MarkSettings>>,

    // 配置
    #[serde(skip)]
    pub settings: Option<Settings>,
}

impl MediaSettings {
    pub fn new(name: &str) -> Result<Self> {
        let path = Settings::media().join(format!("{}.toml", name));
        Self::from_path(path)
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let c = Settings::build_config(path)?;
        let mut s: Self = c.try_deserialize()?;
        s.settings = Some(Settings::new()?);
        Ok(s)
    }

    pub fn settings(&self) -> &Settings {
        self.settings.as_ref().expect("Failed get settings")
    }

    /// 获取媒体存储位置
    pub fn media_dir(&self) -> PathBuf {
        let media = self.media_dir.clone().ok_or(&self.settings().app.media_dir).unwrap();
        PathBuf::from(media)
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
    /// let media = MediaSettings::from_path("examples/media.toml").unwrap();
    ///
    /// let spliter = media.get_spliter(3, 12).unwrap();
    /// assert_eq!(spliter.remove_parts, Some(vec![(0, 90)]));
    /// assert_eq!(spliter.screenshot_seconds, Some(vec![10, 20, 30]));
    /// assert_eq!(spliter.suffix_parts, Some(vec!["ipartment".to_string()]));
    /// assert_eq!(spliter.count, Some(3));
    ///
    /// let spliter = media.get_spliter(3, 11).unwrap();
    /// assert_eq!(spliter.count, Some(2));
    ///
    /// let spliter = media.get_spliter(4, 11).unwrap();
    /// assert_eq!(spliter.count, Some(5));
    /// ```
    pub fn get_spliter(&self, season: u16, episode: u16) -> Option<SpliterSettings> {
        // 使用基础分割器
        if let Some(spliter) = &self.spliter {
            let mut spliter = spliter.clone();
            if let Some(spliters) = &self.spliters {
                // 查找每季分割器
                if let Some(ep_spliter) = spliters.iter().find(|x| x.season == Some(season) && x.episode.is_none()) {
                    spliter.merge_with(ep_spliter);
                }
                // 查找每集分割器
                if let Some(ep_spliter) = spliters.iter().find(|x| x.season == Some(season) && x.episode == Some(episode)) {
                    spliter.merge_with(ep_spliter);
                }
            }
            return Some(spliter);
        }
        None
    }

    /// 获取制作视频配置
    ///
    /// Examples
    ///
    /// ```
    /// use media::MediaSettings;
    /// use std::path::PathBuf;
    ///
    /// let media = MediaSettings::from_path("examples/media.toml").unwrap();
    ///
    /// let item = media.get_mark("2-14-1").unwrap();
    /// assert_eq!(item.parts, Some(vec!["爱2.14.3".to_string(), "爱2.14.4".to_string()]));
    /// assert_eq!(item.suffix_parts, Some(vec!["ipartment".to_string()]));
    ///
    /// let item = media.get_mark("2-14-2").unwrap();
    /// assert_eq!(item.suffix_parts, Some(vec!["ipartment".to_string(), "longmen".to_string()]));
    /// assert!(item.with_suffix());
    /// assert!(!item.trans_1080p());
    ///
    /// let item = media.get_mark("path").unwrap();
    /// assert_eq!(item.path, Some(PathBuf::from("examples/data/trailer.mp4")));
    /// assert!(!item.with_suffix());
    /// assert!(item.trans_1080p());
    /// assert_eq!(item.include_segments, Some(vec![(0, 90)]));
    /// assert_eq!(item.exclude_segments, Some(vec![(0, 90)]));
    /// ```
    pub fn get_mark(&self, id: &str) -> Option<MarkSettings> {
        if let Some(marks) = &self.marks {
            if let Some(mark) = marks.iter().find(|x| x.id == id) {
                let mut mark = mark.clone();
                if mark.suffix_parts.is_none() {
                    mark.suffix_parts = self.suffix_parts.clone();
                }
                return Some(mark);
            }
        }
        None
    }
}
