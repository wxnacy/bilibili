use std::{fs::{self, File}, path::{Path, PathBuf}};
use rand::seq::SliceRandom;

use anyhow::{anyhow, Result};
use bili_video::Video;
use serde::{Deserialize, Serialize};

const PART_NAMES: [&str; 4] = ["longmen", "lord_loser", "ipartment", "feichai"];
const PART_DIR: &str = "/Volumes/Getea/bili_cli/part";
const PART_PATH: &str = "~/.bilibili/part.json";

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
    let mut parts: Vec<Part> = Vec::new();
    for part_name in PART_NAMES{
        let dirname = Path::new(&PART_DIR).join(part_name);
        let dir = Path::new(&dirname);
        let mut videos: Vec<Video> = Vec::new();
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let filename = entry.file_name().into_string().unwrap();
            if !filename.ends_with("ts") {
                continue;
            }

            let video = Video::from(entry.path())?;
            if video.duration > 180.0 {
                continue;
            }
            println!("{} => {:?}", part_name, &video);
            videos.push(video);
        }
        let part = Part::new(part_name.to_string(), videos);
        parts.push(part);
    }

    let part_path = lazytool::expand_user(PART_PATH);
    let writer = File::create(part_path)?;
    serde_json::to_writer_pretty(&writer, &parts)?;
    Ok(())
}

pub fn get_rand_part_path(name: &str) -> Result<PathBuf> {
    let part_path = lazytool::expand_user(PART_PATH);
    let json_str = fs::read_to_string(part_path)?;
    let parts: Vec<Part> = serde_json::from_str(&json_str)?;
    for part in parts {
        if part.name == name {
            let videos = part.videos;
            let mut rng = rand::thread_rng();
            let video = videos.choose(&mut rng).expect("rand part video failed");
            let path = PathBuf::from(&video.path);
            return Ok(path);
        }
    }

    Err(anyhow!("Part: {} not found", name))
}
