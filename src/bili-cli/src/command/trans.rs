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
use regex::Regex;
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
    #[arg(short, long, help = "季数", default_value = "1")]
    pub season: u16,

    // 集数
    #[arg(short, long, help = "集数")]
    pub episode: Option<u16>,

    // 转码地址
    #[arg(long, help = "转码地址")]
    pub to: Option<PathBuf>,
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
        let mut args = args.clone();
        let settings = Settings::new()?;
        let ep = Episode::from_path_with_regex(&args.path, settings.episode_regexs)?
            .expect("parse episode failed");
        if ep.title.is_none() ||
            ep.season.is_none() ||
                ep.episode.is_none() {
            return Err(anyhow!("parse episode failed"));
        }

        let ep_args = EpisodeArgs::new(
            args.type_,
            None,
            ep.title.unwrap(),
            ep.season.unwrap(),
            ep.episode.unwrap());

        println!("{ep_args:#?}");
        let name = ep_args.get_name().expect("failed get name");
        println!("{name}");

        let media = MediaSettings::new(name)?;

        let to = ep_args.get_path()?;
        println!("转码目标地址: {to:?}");
        bili_video::transcode_1080(args.path, &to)?;

        let episode_settings = media.get_episode(ep.season.unwrap(), ep.episode.unwrap());
        println!("{episode_settings:#?}");
        if let Some(settings) = episode_settings {
            // 删减片段
            if let Some(exclude) = settings.exclude_segments {
                let temp_path = to.with_extension("remove.mp4");
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
