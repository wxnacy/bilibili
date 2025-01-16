use std::{fs::{self, File}, path::{Path, PathBuf}};
use rand::seq::SliceRandom;

use anyhow::{anyhow, Result};
use bili_video::Video;
use serde::{Deserialize, Serialize};
use settings::Settings;

#[derive(Debug, Serialize, Deserialize)]
pub struct Part {
    pub name: String,
    pub videos: Vec<Video>,
}

impl Part {
    pub fn new(name: String, videos: Vec<Video>) -> Self {
        Self { name, videos }
    }
}

pub fn init_part() -> Result<()> {
    let stg = Settings::new()?;
    let mut parts: Vec<Part> = Vec::new();
    for part_name in &stg.part.names{
        let dirname = stg.part.home().join(part_name);
        let dir = Path::new(&dirname);
        let mut videos: Vec<Video> = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let filename = entry.file_name().into_string().unwrap();
            // 过滤文件类型
            if !filename.ends_with("ts") {
                continue;
            }

            let video = Video::from(entry.path())?;
            // 过滤时间
            if video.duration > 180.0 {
                continue;
            }
            println!("{} => {:?}", part_name, &video);
            videos.push(video);
        }
        let part = Part::new(part_name.to_string(), videos);
        parts.push(part);
    }

    let writer = File::create(Settings::part())?;
    serde_json::to_writer_pretty(&writer, &parts)?;
    Ok(())
}

pub fn get_rand_part_path(names: Vec<&str>) -> Result<PathBuf> {
    let json_str = fs::read_to_string(Settings::part())?;
    let parts: Vec<Part> = serde_json::from_str(&json_str)?;
    for part in parts {
        if names.contains(&part.name.as_str()) {
            let videos = part.videos;
            let mut rng = rand::thread_rng();
            let video = videos.choose(&mut rng).expect("rand part video failed");
            let path = PathBuf::from(&video.path);
            return Ok(path);
        }
    }

    Err(anyhow!("Part: {:?} not found", names))
}
