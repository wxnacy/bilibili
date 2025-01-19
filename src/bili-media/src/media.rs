use anyhow::Result;

use serde::Deserialize;
use settings::Settings;

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct Spliter {
    // 多媒体目录
    pub season: Option<u16>,
    pub episode: Option<u16>,
    pub count: Option<usize>,
    pub suffix_parts: Option<Vec<String>>,
    pub screenshot_seconds: Option<Vec<f64>>,
}

impl Spliter {
    pub fn screenshot_seconds(&self) -> Vec<f64> {
        self.screenshot_seconds.clone().unwrap_or(vec![10.0, 20.0, 30.0, 300.0, 400.0, 500.0])
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
    pub spliters: Option<Vec<Spliter>>,
    pub spliter: Option<Spliter>
}

impl MediaSettings {
    pub fn new(name: &str) -> Result<Self> {
        let path = Settings::media().join(format!("{}.toml", name));
        let c = Settings::build_config(path)?;
        let s: Self = c.try_deserialize()?;
        Ok(s)
    }

    pub fn get_uploader(&self, season: u16, episode: u16) -> Option<&Uploader> {
        self.uploaders.iter().find(|x| x.season == season && x.episode == episode)
    }

    pub fn get_spliter(&self, season: u16, episode: u16) -> Option<&Spliter> {
        if let Some(spliters) = &self.spliters {
            return spliters.iter().find(|x| x.season == Some(season) && x.episode == Some(episode));
        }
        None
    }

}
