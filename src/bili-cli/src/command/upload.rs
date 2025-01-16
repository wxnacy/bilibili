use std::{env, fs, os, path::{Path, PathBuf}};
use bili_media::get_rand_part_path;

use anyhow::{anyhow, Result};

use clap::{command, Parser};
use lazytool::{expand_user, path::must_to_string, time};

use crate::{cache::{get_episode_name, CACHE_DIR}, create_cache_dir, get_episode_path};

/// `upload` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct UploadArgs {
    #[arg(short, long("type"), default_value = "电视剧", help="类型")]
    pub type_: String,

    // 剧名
    #[arg(short, long, help="剧名")]
    pub name: String,

    // 季数
    #[arg(short, long, help="季数", default_value = "1")]
    pub season: u8,

    // 集数
    #[arg(short, long, help="集数")]
    pub episode: u8,

    // 数量
    #[arg(short, long, help="分割数量", default_value = "4")]
    pub part_num: usize,

    // 封面
    #[arg(short, long, help="封面", default_value = "")]
    pub cover: String,

    // 标签
    #[arg(long, help="标签。用逗号隔开", default_value = "")]
    pub tag: String,

    // 分区
    #[arg(long, help="分区。默认影视剪辑", default_value = "183")]
    pub tid: u32,

    // 单视频文件最大并发数
    #[arg(short, long, help="单视频文件最大并发数", default_value = "4")]
    pub limit: u8,

    // 预发布时间
    #[arg(long, help="预发布时间")]
    pub dtime: String,

    // 描述
    #[arg(short, long, help="描述", default_value = "")]
    pub desc: String,
}


impl UploadArgs {
    // pub fn get_path(&self) -> Result<PathBuf> {
        // let path = get_episode_path(&self.type_, &self.name, self.season, self.episode);
        // Ok(path)
    // }

    /// 获取分割视频的缓存目录
    pub fn get_cache_dir(&self) -> Result<PathBuf> {
        let name = self.get_name();
        let mut names: Vec<String> = Vec::new();
        let cache_dir = expand_user(CACHE_DIR);
        for entry in fs::read_dir(&cache_dir)? {
            let entry = entry?;
            let path = entry.path();
            // 添加符合名称的目录
            if path.file_name().unwrap().to_str().unwrap().starts_with(&name) {
                names.push(must_to_string(path));
            }
        }
        if names.is_empty() {
            return Err(anyhow!("{} can not found cache_dir", &name));
        }
        // 按照名称倒序
        names.sort_by(|a, b| b.cmp(a));
        Ok(cache_dir.join(&names[0]))
    }

    pub fn get_name(&self) -> String {
        get_episode_name(&self.name, self.season, self.episode)
    }

    pub fn to_upload_args(&self) -> Result<Vec<String>> {
        let mut args = vec![
            String::from("--limit"), format!("{}", self.limit),
            String::from("--tid"), format!("{}", self.tid),
        ];
        if !self.cover.is_empty() {
            args.push("--cover".to_string());
            args.push(self.cover.to_string());
        }

        let mut tag = self.tag.clone();
        if tag.is_empty() {
            tag = self.name.clone();
        }
        args.push("--tag".to_string());
        args.push(tag);

        if !self.desc.is_empty() {
            args.push("--desc".to_string());
            args.push(self.desc.to_string());
        }

        if !self.dtime.is_empty() {
            args.push("--dtime".to_string());
            let ts = time::to_timestamp(&self.dtime, "%Y-%m-%d %H:%M:%S")?;
            args.push(format!("{}", ts));
        }
        Ok(args)
    }
}

/// `split` 命令入口
pub fn upload(args: UploadArgs) -> anyhow::Result<()> {


    let upload_args = args.to_upload_args()?;
    println!("{upload_args:?}");

    let cache_dir = args.get_cache_dir()?;
    println!("{cache_dir:?}");

    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(&cache_dir)? {
        let entry = entry?;
        let path = entry.path();
        // 添加符合名称的目录
        if path.file_name().unwrap().to_str().unwrap().ends_with(".mp4") {
            paths.push(path);
        }
    }
    // paths.sort_by(|a, b| a.age.cmp(&b.age));
    paths.sort();
    println!("{paths:#?}");

    for path in paths {
        let mut cmds = vec![
            "biliup".to_string(),
            "-u".to_string(), "/Users/wxnacy/.bilibili/cookie/3493118657694567.json".to_string(),
            "upload".to_string(), must_to_string(&path),
        ];
        cmds.extend(upload_args.clone());
        let image = path.with_extension("png");
        cmds.push("--cover".to_string());
        cmds.push(must_to_string(image));
        println!("{cmds:#?}");
        lazycmd::spawn(cmds)?;
    }

    // let split_ts = split_and_to_ts(&args)?;
    // println!("{split_ts:#?}");
    // for ts in split_ts {
        // bili_video::concat([
            // ts.clone(),
            // get_rand_part_path("ipartment")?,
            // get_rand_part_path("longmen")?,
        // ], ts.with_extension("mp4"))?;
        // fs::remove_file(ts)?;
    // }
    Ok(())
}

// pub fn split_and_to_ts(args: &SplitArgs) -> Result<Vec<PathBuf>> {
    // let cache = args.get_cache_dir()?;

    // let split_target = cache.join(args.get_name());
    // let path = args.get_path()?;

    // let split_paths = bili_video::split(&path, &split_target, args.count)?;
    // let mut concat_rs: Vec<PathBuf> = Vec::new();
    // for sp in split_paths {
        // let ts = bili_video::to_ts(&sp, None)?;
        // fs::remove_file(sp)?;
        // concat_rs.push(ts);
    // }
    // Ok(concat_rs)
// }
