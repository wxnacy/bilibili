use std::{fs, path::PathBuf};
use bili_video::{Remover, Spliter};
use lazytool::path::must_get_filename;
use media::{get_rand_part_path, MediaSettings, SpliterSettings};

use anyhow::{Result, anyhow};

use clap::{command, Parser};
use settings::Settings;

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
    #[arg(short, long, help="分割数量", default_value_t)]
    pub count: usize,

    // 是否使用快速分离
    #[arg(short('q'), long, help="是否快速分离")]
    pub with_quick: bool,

    // 是否使用缓存
    #[arg(short('C'), long, help="是否使用缓存")]
    pub with_cache: bool,
}

/// `split` 命令入口
pub fn split(args: SplitArgs) -> anyhow::Result<()> {
    println!("{args:#?}");
    let mut args = args.clone();
    let mut ep = args.ep.clone();
    if ep.title.is_empty() && ep.name.is_empty() {
        panic!("title or name must have one");
    }

    let media = MediaSettings::new(&args.ep.get_name().expect("failed get name"))?;
    ep.fill_from_media(&media);
    args.ep = ep.clone();

    let spliter = media.get_spliter(ep.season, ep.episode).unwrap();
    println!("{spliter:#?}");

    // 封装分割数量
    if args.count == 0 && spliter.count.is_some(){
        args.count = spliter.count.unwrap();
    }
    if args.count == 0 {
        return Err(anyhow!("count is 0"));
    }

    let suffix_parts = spliter.suffix_parts.clone().expect("toml not found suffix_parts");

    // 转码时已经移除片头片段，分割时不再使用视频的片头
    // 处理移除片段
    // let episode_opt = media.get_episode(ep.season, ep.episode);
    // if let Some(episode) = episode_opt {
        // if episode.exclude_segments.is_some() && spliter.remove_parts.is_none() {
            // spliter.remove_parts = episode.exclude_segments
        // }
    // }

    let split_ts = if args.with_cache { get_cache_ts_list(&args)? } else { split_and_to_ts(&args, &spliter)? };
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
    let mut target_name = ep.get_full_title();
    if !ep.episode_title.is_empty() {
        target_name = format!("{}-{}", &ep.episode_title, ep.get_full_title())
    }
    // 使用别名
    if !args.alias.is_empty() {
        target_name = args.alias.clone();
    }

    let origin_path = ep.get_path()?;
    let mut cache_path = cache.join(&target_name).with_extension("mp4");
    fs::copy(&origin_path, &cache_path)?;

    // 判断是否去掉片头片尾
    if let Some(remove_parts) = &spliter.exclude_segments {
        if !remove_parts.is_empty() {
            println!("remove_parts {:?}", &remove_parts);
            let remove_part_path = cache.join(&target_name).with_extension("remove.mp4");
            let mut r = Remover::new(&cache_path, remove_parts.to_vec());
            r.with_quick(args.with_quick).output(&remove_part_path)?;
            fs::remove_file(&cache_path)?;
            cache_path = remove_part_path;
        }
    }

    let split_target = cache.join(target_name);
    // 分割
    let mut s = Spliter::new(&cache_path);
    let split_paths = s
        .set_parts(args.count)
        .with_quick(args.with_quick)
        .output(split_target)?;

    let ts_cache_dir = create_cache_ts_dir(args)?;

    let mut concat_rs: Vec<PathBuf> = Vec::new();
    for sp in split_paths {
        let ts = bili_video::to_ts(&sp, None)?;
        fs::remove_file(sp)?;
        let ts_temp = ts_cache_dir.join(must_get_filename(&ts));
        if !ts_temp.exists() {
            fs::copy(&ts, ts_temp)?;
        }
        concat_rs.push(ts);
    }
    fs::remove_file(cache_path)?;
    Ok(concat_rs)
}

fn create_cache_ts_dir(args: &SplitArgs) -> Result<PathBuf> {
    let dir = get_cache_ts_dir(args)?;
    if !dir.exists() {
        fs::create_dir_all(&dir)?;
    }
    Ok(dir)
}

fn get_cache_ts_dir(args: &SplitArgs) -> Result<PathBuf> {
    let root = Settings::cache();
    let dir = root
        .join("split")
        .join(&args.ep.name)
        .join(format!("{}-{}", args.ep.get_full_title(), args.count));
    println!("cache ts dir: {dir:?}");
    Ok(dir)
}

fn get_cache_ts_list(args: &SplitArgs) -> Result<Vec<PathBuf>> {
    let cache_dir = args.ep.create_cache_dir()?;
    let cache_ts_dir = get_cache_ts_dir(args)?;
    if !cache_ts_dir.exists() {
        return Err(anyhow!("{:?} ts cache not found", &cache_ts_dir));
    }

    let mut results: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(cache_ts_dir)? {
        let entry = entry?;
        let path = entry.path();
        if must_get_filename(&path).ends_with("ts") {
            let ts = cache_dir.join(must_get_filename(&path));
            fs::copy(&path, &ts)?;
            results.push(ts);
        }
    }
    Ok(results)
}
