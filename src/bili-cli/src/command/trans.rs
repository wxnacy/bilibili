//! 视频转码
//!
//! ```bash
//! # 自动解析 episode
//! cargo run -- trans "龙门镖局1.5/04.mp4" -n 龙门镖局 -s 3
//! cargo run -- trans "龙门镖局.Longmen.Express.2013.E07.4K.2160p.HEVC.AAC-DHTCLUB.mp4" -n 龙门镖局
//! ```
use std::{fs, path::{Path, PathBuf}};

use anyhow::Result;

use clap::{command, Parser};
use lazytool::path::must_get_filename;
use regex::Regex;


/// `trans` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct TransArgs {
    from: String,

    // 动作
    #[arg(short, long, default_value = "1080p", help="转码动作")]
    pub action: String,

    // 视频类型
    #[arg(short, long("type"), default_value = "电视剧", help="类型")]
    pub type_: String,

    // 剧名
    #[arg(short, long, help="剧名", default_value = "")]
    pub name: String,

    // 季数
    #[arg(short, long, help="季数", default_value = "1")]
    pub season: u8,

    // 集数
    #[arg(short, long, help="集数")]
    pub episode: Option<u16>,
}

impl TransArgs {
    pub fn get_dir(&self) -> Result<PathBuf> {
        let ep_dir_name = format!("{}{}", &self.name, self.season);
        let dirname = format!(
            "/Volumes/ZhiTai/Movies/{}/{}/{}",
            &self.type_,
            &self.name,
            ep_dir_name,
        );
        let dirname = dirname.as_str();
        let dir = Path::new(&dirname);
        if !dir.exists() {
            fs::create_dir_all(dir)?;
        }
        Ok(dir.to_path_buf())
    }

    pub fn get_to(&self) -> Result<PathBuf> {
        let path = self.get_dir()?;
        let path = path.join(format!("S{:02}E{:02}.mp4", &self.season, self.get_episode()));
        Ok(path)
    }

    pub fn get_episode(&self) -> u16 {
        // 如果传值直接返回
        if let Some(episode) = &self.episode {
            return *episode;
        }

        // 从名称中获取
        if let Some(episode) = self.get_episode_by_from() {
            return episode;
        }

        panic!("can not get episode")
    }

    pub fn get_episode_by_from(&self) -> Option<u16> {
        let from_path = Path::new(&self.from);
        if from_path.is_file() {
            if let Some(filename) = from_path.file_name() {
                if let Some(name) = filename.to_str() {
                    return Self::match_episode(name);
                }
            }
        }
        None
    }

    /// 通过文件名匹配到剧集
    ///
    /// Examples
    ///
    /// ```
    /// use bili_cli::command::TransArgs;
    ///
    /// let ep = TransArgs::match_episode("03.mp4");
    /// assert_eq!(ep, Some(3));
    ///
    /// let ep = TransArgs::match_episode("龙门镖局.Longmen.Express.2013.E07.4K.2160p.HEVC.AAC-DHTCLUB.mp4");
    /// assert_eq!(ep, Some(7));
    /// ```
    pub fn match_episode(filename: &str) -> Option<u16> {
        // 使用文件名直接解析
        if let Some(ep) = Self::match_episode_by_parse(filename) {
            return Some(ep);
        }
        // 创建正则表达式以匹配 E 后跟数字的模式
        let re = Regex::new(r"E(\d+)").unwrap();
        // 使用正则表达式进行匹配
        if let Some(captures) = re.captures(filename) {
            // 捕获组 1 即为数字部分
            if let Some(season_number) = captures.get(1) {
                // 将字符串解析为 u8
                return season_number.as_str().parse::<u16>().ok();
            }
        }
        None // 返回 None 代表未找到季数信息
    }

    /// 直接解析名称
    pub fn match_episode_by_parse(filename: &str) -> Option<u16> {
        // 按照 "." 分割文件名
        let parts: Vec<&str> = filename.split('.').collect();

        // 获取文件名的第一部分（假设是数字）
        if let Some(first_part) = parts.first() {
            // 尝试将其解析为 u16
            return first_part.parse::<u16>().ok();
        }
        None // 返回 None 代表未找到数字信息
    }

}

/// `trans` 命令入口
pub fn trans(args: TransArgs) -> anyhow::Result<()> {
    match args.action.as_str() {

        // mp4 视频转为指定 1080p 格式
        "1080p" => {
            let to = args.get_to()?;
            println!("转码目标地址: {to:?}");
            bili_video::transcode_1080(args.from, to)?;
        },

        // 将视频转为 mp4 格式
        "mp4" => {
            let from = Path::new(&args.from);
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
                bili_video::to_mp4(&args.from, None)?;
            }
        },
        _ => eprintln!("action: {} not found", &args.action)
    }

    Ok(())
}
