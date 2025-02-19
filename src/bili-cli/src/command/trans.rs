//! 视频转码
//!
//! ```bash
//! # 自动解析 episode
//! cargo run -- trans "龙门镖局1.5/04.mp4" -n 龙门镖局 -s 3
//! cargo run -- trans "龙门镖局.Longmen.Express.2013.E07.4K.2160p.HEVC.AAC-DHTCLUB.mp4" -n 龙门镖局
//! ```
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Result};

use bili_video::Remover;
use clap::{command, Parser};
use lazytool::{path::must_get_filename, Episode};
use media::MediaSettings;
use settings::Settings;

use crate::command::model::EpisodeArgs;

/// `trans` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct TransArgs {
    path: String,

    // 动作
    #[arg(short, long, default_value = "1080p", help = "转码动作")]
    pub action: String,

    // 视频类型
    #[arg(long("type"), default_value = "电视剧", help = "类型")]
    pub type_: String,

    // 剧名
    #[arg(short, long, help = "剧名", default_value_t)]
    pub title: String,

    // 剧名
    #[arg(short, long, help = "简称", default_value_t)]
    pub name: String,

    // 季数
    #[arg(short, long, help = "季数", default_value_t)]
    pub season: u16,

    // 集数
    #[arg(short, long, help = "集数", default_value_t)]
    pub episode: u16,

    // 转码地址
    #[arg(long, help = "转码地址")]
    pub to: Option<PathBuf>,

    // 是否执行
    #[arg(short, long, help = "是否执行")]
    pub yes: bool,
}

/// `trans` 命令入口
pub fn trans(args: TransArgs) -> anyhow::Result<()> {
    if let Some(transer) = get_trans(&args.action) {
        transer.trans(&args)
    } else {
        Err(anyhow!("{} not match", &args.action))
    }
}

trait Trans {
    fn get_action(&self) -> String;
    fn trans(&self, args: &TransArgs) -> Result<()>;
}

#[derive(Debug)]
struct Mp3Trans {}

impl Trans for Mp3Trans {
    fn get_action(&self) -> String {
        "mp3".to_string()
    }

    fn trans(&self, args: &TransArgs) -> Result<()> {
        bili_video::to_mp3(&args.path, args.to.clone()).map_err(|e| anyhow!("to mp3 failed: {}", e))?;
        Ok(())
    }
}

#[derive(Debug)]
struct Mp4Trans {}

impl Trans for Mp4Trans {
    fn get_action(&self) -> String {
        "mp4".to_string()
    }

    fn trans(&self, args: &TransArgs) -> Result<()> {
        let from = Path::new(&args.path);
        if from.is_dir() {
            // 目录遍历文件在转码
            for entry in fs::read_dir(from)? {
                let entry = entry?;
                let path = entry.path();
                let filename = must_get_filename(&path);
                if !filename.ends_with("mkv") {
                    continue;
                }
                let to_path = path.with_extension("mp4");
                if to_path.exists() {
                    continue;
                }
                bili_video::to_mp4(path, Some(to_path))?;
            }
        } else {
            // 文件直接转码
            bili_video::to_mp4(&args.path, args.to.clone())?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Mp41080Trans {}

impl Trans for Mp41080Trans {
    fn get_action(&self) -> String {
        "1080p".to_string()
    }

    fn trans(&self, args: &TransArgs) -> Result<()> {
        let ep = trans_to_episode(args)?;

        println!("{ep:#?}");
        let name = ep.get_name().expect("failed get name");
        println!("{name}");

        let media = MediaSettings::new(&name)?;

        let to = ep.get_path()?;
        println!("转码目标地址: {to:?}");
        let episode_settings = media.get_episode(ep.season, ep.episode);
        println!("{episode_settings:#?}");

        if !args.yes {
            return Ok(());
        }

        bili_video::transcode_1080(&args.path, &to)?;

        if let Some(settings) = episode_settings {
            // 删减片段
            if let Some(exclude) = settings.exclude_segments {
                let temp_path = to.with_extension("need-remove.mp4");
                fs::rename(&to, &temp_path)?;
                let r = Remover::new(&temp_path, exclude);
                r.output(to)?;
                fs::remove_file(&temp_path)?;
            }
        }
        Ok(())
    }
}

fn get_trans(action: &str) -> Option<Box<dyn Trans>> {
    match action {
        "mp3" => Some(Box::new(Mp3Trans {})),
        "mp4" => Some(Box::new(Mp4Trans {})),
        "1080p" => Some(Box::new(Mp41080Trans {})),
        _ => None,
    }
}

fn trans_to_episode(args: &TransArgs) -> Result<EpisodeArgs> {
    let settings = Settings::new()?;
    let ep_opt = Episode::from_path_with_regex(&args.path, settings.episode_regexs)?;
    let mut title = args.title.clone();
    let mut season = args.season;
    let mut episode = args.episode;

    if let Some(ep) = ep_opt {
        title = if let Some(title) = ep.title { title } else {args.title.clone()};
        season = if let Some(season) = ep.season { season } else {args.season};
        episode = if let Some(episode) = ep.episode { episode } else {args.episode};
    }

    if !args.title.is_empty() {
        title = args.title.clone()
    }

    if args.season != 0 {
        season = args.season
    }

    if args.episode != 0 {
        episode = args.episode
    }

    let ep = EpisodeArgs::new(
        args.type_.clone(),
        None,
        title,
        season,
        episode);
    if ep.title.is_empty() ||
        ep.season == 0 ||
            ep.episode == 0 {
        Err(anyhow!("parse episode failed"))
    } else {
        Ok(ep)
    }
}


#[cfg(test)]
mod tests {
    use super::{trans_to_episode, TransArgs};
    use anyhow::Result;

    fn new_trans(path: &str, title: &str, season: u16, episode: u16) -> TransArgs {
        TransArgs {
            path: path.to_string(),
            action: "1080p".to_string(),
            type_: "电视剧".to_string(),
            title: title.to_string(),
            name: String::new(),
            season,
            episode,
            yes: false,
            to: None
        }
    }

    #[test]
    fn test_trans_to_episode() -> Result<()>{
        let args = &new_trans(
            "/Volumes/Getea/影片/电视剧/医馆笑传/医馆笑传S02.37集.1080P/37.mp4",
            "",
            0,
            0,
        );

        let ep = trans_to_episode(args)?;
        assert_eq!(ep.title, String::from("医馆笑传"));
        assert_eq!(ep.season, 2);
        assert_eq!(ep.episode, 37);

        let args = &new_trans(
            "/Volumes/Getea/影片/电视剧/医馆笑传/医馆笑传S02.37集.1080P/37.mp4",
            "美国队长",
            1,
            2,
        );

        let ep = trans_to_episode(args)?;
        assert_eq!(ep.title, String::from("美国队长"));
        assert_eq!(ep.season, 1);
        assert_eq!(ep.episode, 2);

        let args = &new_trans(
            "/Volumes/ZhiTai/bilibili/cache/休息吧，托尼，这么多年辛苦了.mp4",
            "美国队长",
            1,
            2,
        );

        let ep = trans_to_episode(args)?;
        assert_eq!(ep.title, String::from("美国队长"));
        assert_eq!(ep.season, 1);
        assert_eq!(ep.episode, 2);

        Ok(())
    }
}

