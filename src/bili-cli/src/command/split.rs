use std::{fs, path::PathBuf};
use media::{get_rand_part_path, MediaSettings};

use anyhow::Result;

use bili_video::Spliter;
use clap::{command, Parser};

use super::model::EpisodeArgs;

/// `split` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
pub struct SplitArgs {

    #[command(flatten)]
    pub ep: EpisodeArgs,

    // 数量
    #[arg(short, long, help="分割数量", default_value = "4")]
    pub count: usize,

    // 是否使用快速分离
    #[arg(short('q'), long, help="是否快速分离")]
    pub with_quick: bool,
}

/// `split` 命令入口
pub fn split(args: SplitArgs) -> anyhow::Result<()> {
    let mut args = args.clone();
    let mut ep = args.ep.clone();
    if ep.title.is_empty() && ep.name.is_empty() {
        panic!("title or name must have one");
    }

    let media = MediaSettings::new(&args.ep.name)?;
    if ep.title.is_empty() {
        ep.title = media.title.clone();
    }
    args.ep = ep;

    let spliter = media.spliter.expect("toml not found spliter");
    let suffix_parts = spliter.suffix_parts.clone().expect("toml not found suffix_parts");

    let split_ts = split_and_to_ts(&args)?;
    println!("{split_ts:#?}");
    for ts in split_ts {

        // 合并分割后的视频
        let part = bili_video::concat([
            ts.clone(),
            get_rand_part_path(suffix_parts.clone())?,
        ], ts.with_extension("mp4"))?;
        fs::remove_file(ts)?;

        // 对分割后的视频截图
        for (index, second) in spliter.screenshot_seconds().iter().enumerate() {
            bili_video::screenshot(&part, part.with_extension(format!("{}.png", index)), *second)?;
        }
    }
    Ok(())
}

pub fn split_and_to_ts(args: &SplitArgs) -> Result<Vec<PathBuf>> {
    let ep = args.ep.clone();
    let cache = ep.create_cache_dir()?;
    let target_name = ep.get_name();

    let origin_path = ep.get_path()?;
    let cache_path = cache.join(&target_name).with_extension("mp4");
    fs::copy(&origin_path, &cache_path)?;
    // let remove_path = bili_video::cut(cache_path, cache.join(args.get_name()).with_extension("remove-op.mp4"), 90.0, 2490.0)?;
    // let ts = bili_video::to_ts(&remove_op_path, None)?;
    // let path = bili_video::concat([ts], cache.join(&target_name).with_extension("mp4"))?;

    let split_target = cache.join(&target_name);
    // 分割
    let mut s = Spliter::new(&cache_path);
    let split_paths = s
        .set_parts(args.count)
        .with_quick(args.with_quick)
        .output(split_target)?;


    let mut concat_rs: Vec<PathBuf> = Vec::new();
    for sp in split_paths {
        let ts = bili_video::to_ts(&sp, None)?;
        fs::remove_file(sp)?;
        concat_rs.push(ts);
    }
    fs::remove_file(cache_path)?;
    Ok(concat_rs)
}
