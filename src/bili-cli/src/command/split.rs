use std::{fs, path::PathBuf};
use media::get_rand_part_path;

use anyhow::Result;

use bili_video::Spliter;
use clap::{command, Parser};

use crate::{cache::get_episode_name, create_cache_dir, get_episode_path};

/// `split` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct SplitArgs {
    #[arg(short, long("type"), default_value = "电视剧", help="类型")]
    pub type_: String,

    // 剧名
    #[arg(short, long, help="剧名")]
    pub name: String,

    // 季数
    #[arg(short, long, help="季数", default_value = "1")]
    pub season: u16,

    // 集数
    #[arg(short, long, help="集数")]
    pub episode: u16,

    // 数量
    #[arg(short, long, help="分割数量", default_value = "4")]
    pub count: usize,

    // 是否使用快速分离
    #[arg(short('q'), long, help="是否快速分离")]
    pub with_quick: bool,
}

impl SplitArgs {
    pub fn get_path(&self) -> Result<PathBuf> {
        let path = get_episode_path(&self.type_, &self.name, self.season, self.episode);
        Ok(path)
    }

    pub fn get_cache_dir(&self) -> Result<PathBuf> {
        let dir = create_cache_dir(self.get_name())?;
        Ok(dir)
    }

    pub fn get_name(&self) -> String {
        get_episode_name(&self.name, self.season, self.episode)
    }

}

/// `split` 命令入口
pub fn split(args: SplitArgs) -> anyhow::Result<()> {

    let split_ts = split_and_to_ts(&args)?;
    println!("{split_ts:#?}");
    for ts in split_ts {
        let part = bili_video::concat([
            ts.clone(),
            get_rand_part_path(vec!["ipartment"])?,
            // get_rand_part_path("lord_loser")?,
            // get_rand_part_path("feichai")?,
            // get_rand_part_path("longmen")?,
        ], ts.with_extension("mp4"))?;
        fs::remove_file(ts)?;

        bili_video::screenshot(&part, part.with_extension("png"), 10.0)?;
        bili_video::screenshot(&part, part.with_extension("1.png"), 20.0)?;
        bili_video::screenshot(&part, part.with_extension("2.png"), 30.0)?;
        bili_video::screenshot(&part, part.with_extension("3.png"), 180.0)?;
        bili_video::screenshot(&part, part.with_extension("4.png"), 190.0)?;
        bili_video::screenshot(&part, part.with_extension("5.png"), 200.0)?;
    }
    Ok(())
}

pub fn split_and_to_ts(args: &SplitArgs) -> Result<Vec<PathBuf>> {
    let cache = args.get_cache_dir()?;
    let target_name = args.get_name();

    let origin_path = args.get_path()?;
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
