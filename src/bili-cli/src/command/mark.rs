use std::{fs, path::PathBuf};

use anyhow::Result;

use bili_video::{concat, to_ts, transcode_1080, Remover};
use clap::{command, Parser};
use lazytool::path::must_get_filename;
use media::{get_rand_part_path, MarkSettings, MediaSettings};
use settings::Settings;

use crate::create_cache_dir;

/// `mark` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct MarkArgs {
    name: String,
    id: String,

    // 名称
    #[arg(short, long, help="视频名称", default_value_t)]
    pub title: String,

    // 是否使用快速分离
    #[arg(short('q'), long, help="是否快速分离")]
    pub with_quick: bool,
}

/// `mark` 命令入口
pub fn mark(args: MarkArgs) -> Result<()> {
    // let settings = Settings::new()?;
    let media = MediaSettings::new(&args.name)?;
    let m = media.get_mark(&args.id).expect("mark not found");

    // 设置目标地址
    let title = if args.title.is_empty() { &m.title } else { &args.title };
    let cache_dir = create_cache_dir(title)?;
    let target_path = cache_dir.join(format!("{}.mp4", title));

    // 判断制作类型
    if m.path.is_some() {
        mark_path(args, &m, target_path)?;
    }

    // let mut paths: Vec<PathBuf> = Vec::new();
    // if let Some(parts) = m.parts {
        // for pid in parts {
            // let path = settings.part.get_path(&args.name, &pid);
            // if path.exists() {
                // paths.push(path);
            // } else {
                // let path = PathBuf::from(&pid);
                // if path.exists() {
                    // paths.push(path);
                // }
            // }
        // }
    // }

    // if let Some(suffix_parts) = m.suffix_parts {
        // for name in suffix_parts {
            // let path = get_rand_part_path(vec![name.clone()])?;
            // if path.exists() {
                // paths.push(path);
            // }
        // }
    // }


    // concat(&paths, &target_path)?;

    // println!("{paths:#?}");
    Ok(())
}

pub fn mark_path(
    args: MarkArgs,
    mark_config: &MarkSettings,
    target_path: PathBuf,
) -> Result<()> {

    let path = mark_config.path.clone().expect("Failed get mark path");
    let mut cache_path = target_path.with_extension("cache.mp4");
    fs::copy(&path, &cache_path)?;

    if let Some(exclude) = &mark_config.exclude_segments {
        let remove_path = cache_path.with_extension("remove.mp4");
        let mut r = Remover::new(&cache_path, exclude.to_vec());
        r.with_quick(args.with_quick).output(&remove_path);
        fs::remove_file(&cache_path);
        cache_path = remove_path;
    }

    if mark_config.trans_1080p() {
        let trans_path = cache_path.with_extension("trans.mp4");
        transcode_1080(&cache_path, &trans_path)?;
        fs::remove_file(&cache_path)?;
        cache_path = trans_path
    }

    // 拼接后缀视频
    concat_suffix(mark_config, &cache_path, &target_path)?;
    Ok(())
}

/// 合并后缀视频
pub fn concat_suffix(
    mark_config: &MarkSettings,
    source_path: &PathBuf,
    target_path: &PathBuf,
) -> Result<()> {
    if mark_config.with_suffix() {
        let mut paths: Vec<PathBuf> = Vec::new();

        // 判断是否需要转为 ts
        let filename = must_get_filename(source_path);
        if filename.ends_with("mp4") {
            let cache_ts = to_ts(source_path, None)?;
            paths.push(cache_ts);
        } else {
            paths.push(source_path.to_path_buf());
        }
        if let Some(suffix_parts) = &mark_config.suffix_parts {
            for name in suffix_parts {
                let path = get_rand_part_path(vec![name.clone()])?;
                if path.exists() {
                    paths.push(path);
                }
            }
        }
        concat(&paths, &target_path)?;
    }
    Ok(())
}
