use std::path::{Path, PathBuf};

use anyhow::Result;

use serde::Deserialize;
use settings::Settings;

pub trait EpisodeSettings {
    fn get_season(&self) -> Option<u16>;
    fn get_episode(&self) -> Option<u16>;
    fn merge_with(&mut self, other: &Self);
}

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

#[derive(Debug, Clone, Deserialize, Default)]
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

}

impl EpisodeSettings for SpliterSettings {

    fn get_episode(&self) -> Option<u16> {
        self.episode
    }

    fn get_season(&self) -> Option<u16> {
        self.season
    }

    fn merge_with(&mut self, other: &SpliterSettings) {
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

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(unused)]
pub struct TransSettings {
    // 多媒体目录
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub exclude_segments: Option<Vec<(u64, u64)>>,
}

impl EpisodeSettings for TransSettings {
    fn get_season(&self) -> Option<u16> {
        self.season
    }

    fn get_episode(&self) -> Option<u16> {
        self.episode
    }

    fn merge_with(&mut self, other: &TransSettings) {
        if other.season.is_some() {
            self.season = other.season;
        }
        if other.episode.is_some() {
            self.episode = other.episode;
        }
        if other.exclude_segments.is_some() {
            self.exclude_segments = other.exclude_segments.clone();
        }
    }
}

#[derive(Debug, Clone, Deserialize, Default)]
#[allow(unused)]
pub struct UploaderSettings {
    // 多媒体目录
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub dtime: Option<String>,
    pub tag: Option<String>,
}

impl EpisodeSettings for UploaderSettings {

    fn get_season(&self) -> Option<u16> {
        self.season
    }

    fn get_episode(&self) -> Option<u16> {
        self.episode
    }

    fn merge_with(&mut self, other: &UploaderSettings) {
        if other.season.is_some() {
            self.season = other.season;
        }
        if other.episode.is_some() {
            self.episode = other.episode;
        }
        if other.dtime.is_some() {
            self.dtime = other.dtime.clone();
        }
        if other.tag.is_some() {
            self.tag = other.tag.clone();
        }
    }
}


#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct MediaSettings {
    pub name: String,
    pub title: String,
    pub media_dir: Option<String>,
    pub suffix_parts: Option<Vec<String>>,

    // 转码配置
    // pub tran: Option<TransSettings>,
    pub trans: Option<Vec<TransSettings>>,

    // 上传配置
    pub uploaders: Option<Vec<UploaderSettings>>,

    // 分割配置
    pub spliters: Option<Vec<SpliterSettings>>,

    // 制作配置
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
        if let Some(_dir) = &self.media_dir {
            return PathBuf::from(_dir);
        }
        PathBuf::from(&self.settings().app.media_dir)
    }

    /// 获取上传信息
    ///
    /// Examples
    ///
    /// ```
    /// use media::MediaSettings;
    ///
    /// let media = MediaSettings::from_path("examples/media.toml").unwrap();
    ///
    /// let uploader = media.get_uploader(3, 6).unwrap();
    /// assert_eq!(uploader.tag, Some("电视剧,影视剪辑,龙门镖局1.5,龙门镖局".to_string()));
    /// assert_eq!(uploader.dtime, Some("2025-01-19 11:00:00".to_string()));
    ///
    /// let uploader = media.get_uploader(3, 7).unwrap();
    /// assert_eq!(uploader.tag, Some("电视剧,影视剪辑,龙门镖局1.5,龙门镖局".to_string()));
    /// assert_eq!(uploader.dtime, None);
    ///
    /// let uploader = media.get_uploader(2, 7).unwrap();
    /// assert_eq!(uploader.tag, Some("电视剧,影视剪辑,龙门镖局".to_string()));
    /// assert_eq!(uploader.dtime, None);
    /// ```
    pub fn get_uploader(&self, season: u16, episode: u16) -> Option<UploaderSettings> {
        self.get_episode_settings(season, episode, &None, &self.uploaders)
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
    /// assert_eq!(spliter.remove_parts, Some(vec![(0, 80)]));
    ///
    /// let spliter = media.get_spliter(4, 11).unwrap();
    /// assert_eq!(spliter.count, Some(5));
    /// ```
    pub fn get_spliter(&self, season: u16, episode: u16) -> Option<SpliterSettings> {
        self.get_episode_settings(season, episode, &None, &self.spliters)
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

    /// 获取转码视频配置
    ///
    /// ```
    pub fn get_trans(&self, season: u16, episode: u16) -> Option<TransSettings> {
        self.get_episode_settings(season, episode, &None, &self.trans)
    }

    pub fn get_episode_settings<T: EpisodeSettings + Clone + Default>(
        &self,
        season: u16,
        episode: u16,
        stg: &Option<T>,
        settings: &Option<Vec<T>>
    ) -> Option<T> {
        // config 有值时
        let mut item = T::default();
        let mut has = false;
        if let Some(config) = stg {
            item.merge_with(config);
            has = true;
        }

        if let Some(configs) = settings {
            // 查找默认
            if let Some(ep) = configs.iter()
                .find(|x| x.get_season().is_none() && x.get_episode().is_none()) {
                item.merge_with(ep);
                has = true;
            }
            // 查找每季
            if let Some(ep) = configs.iter()
                .find(|x| x.get_season() == Some(season) && x.get_episode().is_none()) {
                item.merge_with(ep);
                has = true;
            }
            // 查找每集
            if let Some(ep) = configs.iter()
                .find(|x| x.get_season() == Some(season) && x.get_episode() == Some(episode)) {
                item.merge_with(ep);
            has = true;
            }
        }
        if has { Some(item) } else { None }
    }
}
