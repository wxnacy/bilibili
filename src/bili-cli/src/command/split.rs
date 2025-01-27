use std::{fs, path::PathBuf};
use bili_video::{Remover, Spliter};
use media::{get_rand_part_path, MediaSettings, SpliterSettings};

use anyhow::Result;

use clap::{command, Parser};

use super::model::EpisodeArgs;

/// `split` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(about, long_about = None)]
pub struct SplitArgs {

    #[command(flatten)]
    pub ep: EpisodeArgs,

    // 数量
    #[arg(short, long, help="别名", default_value_t)]
    pub alias: String,

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
    args.ep = ep.clone();

    let spliter = media.get_spliter(ep.season, ep.episode).unwrap();
    let suffix_parts = spliter.suffix_parts.clone().expect("toml not found suffix_parts");

    let split_ts = split_and_to_ts(&args, &spliter)?;
    println!("{split_ts:#?}");
    for ts in split_ts {
        // 拼接后缀
        let mut need_concat_ts = vec![ts.clone()];
        for part in &suffix_parts {
            need_concat_ts.push(get_rand_part_path(vec![part.to_string()])?);
        }

        // 合并分割后的视频
        let part = bili_video::concat(need_concat_ts, ts.with_extension("mp4"))?;
        fs::remove_file(ts)?;

        // 对分割后的视频截图
        for (index, second) in spliter.screenshot_seconds().iter().enumerate() {
            bili_video::screenshot(&part, part.with_extension(format!("{}.png", index)), (*second) as f64)?;
        }
    }
    Ok(())
}

pub fn split_and_to_ts(
    args: &SplitArgs,
    spliter: &SpliterSettings,
) -> Result<Vec<PathBuf>> {
    let ep = args.ep.clone();
    let cache = ep.create_cache_dir()?;
    let mut target_name = ep.get_name();
    // 使用别名
    if !args.alias.is_empty() {
        target_name = args.alias.clone();
    }

    let origin_path = ep.get_path()?;
    let mut cache_path = cache.join(&target_name).with_extension("mp4");
    fs::copy(&origin_path, &cache_path)?;

    // 判断是否去掉片头片尾
    if let Some(remove_parts) = &spliter.remove_parts {
        if !remove_parts.is_empty() {
            let remove_part_path = cache.join(&target_name).with_extension("remove.mp4");
            let mut r = Remover::new(&cache_path, remove_parts.to_vec());
            r.with_quick(args.with_quick).output(&remove_part_path)?;
            fs::remove_file(&cache_path)?;
            cache_path = remove_part_path;
        }
    }

    let split_target = cache.join(&target_name);
    // 分割
    let mut s = Spliter::new(&cache_path);
    let mut count = args.count;
    // 如果配置中没有分割数量，使用参数中的配置
    if spliter.count.is_some() {
        count = spliter.count.unwrap();
    }
    let split_paths = s
        .set_parts(count)
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
