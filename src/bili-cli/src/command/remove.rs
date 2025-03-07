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
use tokio::time::error::Elapsed;

use crate::command::model::EpisodeArgs;

/// `trans` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct RemoveArgs {
    path: String,

    // 转码地址
    #[arg(long, help = "转码地址")]
    pub to: Option<PathBuf>,

    /// 指定时间对
    #[arg(
        long,
        help = "指定时间对，格式为 x,y",
        value_parser = parse_pair,
        action = clap::ArgAction::Append
    )]
    pub pairs: Vec<(u64, u64)>,

    // 是否执行
    // #[arg(short, long, help = "是否执行")]
    // pub yes: bool,

    // 是否使用快速分离
    #[arg(short('q'), long, help="是否快速分离")]
    pub with_quick: bool,
}

/// `trans` 命令入口
pub fn remove(args: RemoveArgs) -> anyhow::Result<()> {
    println!("{:?}", &args);

    let from = &args.path;
    let default_to = PathBuf::from(&from).with_extension("-remove.mp4");
    let to = if let Some(to) = &args.to {
        to.as_path()
    } else {
        Path::new(&default_to)
    };
    println!("{:?}", to);
    let mut r = Remover::new(from, args.pairs);
    if args.with_quick {
        r.with_quick(args.with_quick);
    }
    r.output(to)?;
    Ok(())
}

fn parse_pair(s: &str) -> Result<(u64, u64), String> {
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 2 {
        return Err("参数格式错误，应该为 x,y".into());
    }

    let x = parts[0].parse::<u64>().map_err(|_| "x 不是有效的数字")?;
    let y = parts[1].parse::<u64>().map_err(|_| "y 不是有效的数字")?;

    Ok((x, y))
}
