use anyhow::Result;

use serde::Deserialize;
use settings::Settings;


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
}

impl MediaSettings {
    pub fn new(name: &str) -> Result<Self> {
        let path = Settings::media().join(format!("{}.toml", name));
        let c = Settings::build_config(path)?;
        let s: Self = c.try_deserialize()?;
        Ok(s)
    }

    pub fn get_uploader(&self, season: u16, episode: u16) -> Option<&Uploader> {
        self.uploaders.iter().filter(|x| x.season == season && x.episode == episode).next()

    }
}
