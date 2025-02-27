use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};

use clap::{command, Parser};
use lazytool::path::must_to_string;
use media::MediaSettings;
use settings::Settings;

use crate::create_cache_dir;


/// `EpisodeArgs` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
pub struct EpisodeArgs {
    #[arg(long("type"), default_value = "电视剧", help="类型")]
    pub type_: String,

    // 短命
    #[arg(short, long, help="英文名", default_value_t)]
    pub name: String,

    // 剧名
    #[arg(short, long, help="剧名", default_value_t)]
    pub title: String,

    // 剧集名
    #[arg(long, help="剧集名", default_value_t)]
    pub episode_title: String,

    // 季数
    #[arg(short, long, help="季数", default_value = "1")]
    pub season: u16,

    // 集数
    #[arg(short, long, help="集数")]
    pub episode: u16,
}

impl EpisodeArgs {

    pub fn new(type_: String, name: Option<String>, title: String, season: u16, episode: u16) -> Self {
        let mut n = "".to_string();
        if let Some(name_) = name {
            n = name_;
        }
        Self { type_, name: n, title, season, episode, episode_title: String::new() }
    }

    pub fn fill_from_media(&mut self, media: &MediaSettings) -> &mut Self {
        if self.title.is_empty() {
            self.title = media.title.clone()
        }
        if self.episode_title.is_empty() {
            if let Some(ep) = media.get_episode(self.season, self.episode) {
                if let Some(title) = ep.title {
                    self.episode_title = title
                }
            }
        }
        if self.season > 1000 {
            self.type_ = "电影".to_string();
        }
        self
    }

    pub fn get_name(&self) -> Option<String> {
        if self.name.is_empty() {
            let settings = Settings::new().expect("Failed get settings");
            settings.get_media_by_title(self.title.as_str()).map(|media| media.name.clone())
        } else {
            Some(self.name.clone())
        }
    }

    pub fn is_drama(&self) -> bool {
        self.type_ == "电视剧"
    }

    pub fn get_full_title(&self) -> String {
        if self.is_drama() {
            format!("{}S{:02}E{:02}", &self.title, self.season, self.episode)
        } else {
            format!("{}.{:04}.{:05}", &self.episode_title, self.season, self.episode)
        }
    }

    pub fn get_path(&self) -> Result<PathBuf> {
        let media = MediaSettings::new(&self.get_name().expect("failed get name"))?;
        let path = media.media_dir()
            .join(&self.type_)
            .join(&media.title)
            .join(format!("{}{}", &media.title, &self.season))
            .join(format!("S{:02}E{:02}.mp4", &self.season, &self.episode));
        Ok(path)
    }

    /// 创建临时目录
    pub fn create_cache_dir(&self) -> Result<PathBuf> {
        let dir = create_cache_dir(self.get_full_title())?;
        Ok(dir)
    }

    /// 获取分割视频的缓存目录
    pub fn get_cache_dir(&self) -> Result<PathBuf> {
        let name = self.get_full_title();
        let mut names: Vec<String> = Vec::new();
        let cache_dir = Settings::cache();
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            // 添加符合名称的目录
            if path.file_name().unwrap().to_str().unwrap().starts_with(&name) {
                names.push(must_to_string(path));
            }
        }
        if names.is_empty() {
            return Err(anyhow!("{} can not found cache_dir", &name));
        }
        // 按照名称倒序
        names.sort_by(|a, b| b.cmp(a));
        Ok(cache_dir.join(&names[0]))
    }
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use media::MediaSettings;

    use super::EpisodeArgs;

    #[test]
    fn test_fill_from_media() {
        let mut ep = EpisodeArgs::try_parse_from([
            "test",
            "-n", "media",
            "-e", "2",
        ]).unwrap();
        let media = MediaSettings::from_path("../bili-media/examples/media.toml").unwrap();
        ep.fill_from_media(&media);
        assert_eq!(ep.title, "多媒体");
        assert_eq!(ep.episode_title, "标题");
        assert_eq!(ep.get_full_title(), "多媒体S01E02");

        let mut ep = EpisodeArgs::try_parse_from([
            "test",
            "-n", "media",
            "-s", "2009",
            "-e", "1201",
        ]).unwrap();
        let media = MediaSettings::from_path("../bili-media/examples/media.toml").unwrap();
        ep.fill_from_media(&media);
        assert_eq!(ep.get_full_title(), "疯狂的赛车.2009.01201");
    }
}
