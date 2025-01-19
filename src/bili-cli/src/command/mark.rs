use std::path::PathBuf;

use anyhow::Result;

use bili_video::concat;
use clap::{command, Parser};
use media::{get_rand_part_path, MediaSettings};
use settings::Settings;

use crate::create_cache_dir;

/// `init` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct MarkArgs {
    name: String,
    id: String,

    // 名称
    #[arg(short, long, help="视频名称", default_value_t)]
    pub title: String,

}

/// `init` 命令入口
pub fn mark(args: MarkArgs) -> Result<()> {
    let settings = Settings::new()?;
    let media = MediaSettings::new(&args.name)?;
    let m = media.get_mark(&args.id).expect("mark not found");

    let mut paths: Vec<PathBuf> = Vec::new();
    if let Some(parts) = m.parts {
        for pid in parts {
            let path = settings.part.get_path(&args.name, &pid);
            if path.exists() {
                paths.push(path);
            } else {
                let path = PathBuf::from(&pid);
                if path.exists() {
                    paths.push(path);
                }
            }
        }
    }

    if let Some(suffix_parts) = m.suffix_parts {
        for name in suffix_parts {
            let path = get_rand_part_path(vec![name.clone()])?;
            if path.exists() {
                paths.push(path);
            }
        }
    }

    let title: &str;
    if args.title.is_empty() {
        title = &m.title;
    } else {
        title = &args.title;
    }

    let cache_dir = create_cache_dir(title)?;
    let video_path = cache_dir.join(format!("{}.mp4", title));

    concat(&paths, &video_path)?;

    println!("{paths:#?}");
    Ok(())
}
