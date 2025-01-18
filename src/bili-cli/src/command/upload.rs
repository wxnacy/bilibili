use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};

use clap::{command, Parser};
use lazytool::{path::must_to_string, time};
use media::MediaSettings;
use settings::Settings;

use crate::cache::get_episode_name;

/// `upload` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct UploadArgs {
    #[arg(short, long("type"), default_value = "电视剧", help="类型")]
    pub type_: String,

    // 短命
    #[arg(long, help="短名")]
    pub short_name: String,

    // 剧名
    #[arg(short, long, help="剧名", default_value = "")]
    pub name: String,

    // 季数
    #[arg(short, long, help="季数", default_value = "1")]
    pub season: u16,

    // 集数
    #[arg(short, long, help="集数")]
    pub episode: u16,

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
    #[arg(short, long, help="单视频文件最大并发数", default_value = "8")]
    pub limit: u8,

    // 预发布时间
    #[arg(long, help="预发布时间", default_value = "")]
    pub dtime: String,

    // 描述
    #[arg(short, long, help="描述", default_value = "")]
    pub desc: String,

    // 上传 up
    #[arg(long, help="up mid")]
    pub mid: Option<u64>,
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
        let cache_dir = Settings::cache();
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
    let media = MediaSettings::new(&args.short_name)?;
    let mut args = args.clone();
    if args.name.is_empty() {
        args.name = media.name.clone();
    }
    if let Some(uploader) = media.get_uploader(args.season, args.episode) {
        if args.dtime.is_empty() {
            args.dtime = uploader.dtime.clone();
        }
    }
    let stg = Settings::new()?;
    let up = stg.get_up(args.mid).expect("Get up failed");

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


    let cookie_path = up.get_cookie_path();
    println!("上传 UP: {}({})", &up.name, &up.mid);

    for path in paths {
        // 拼接参数
        let mut cmds = vec![
            "biliup".to_string(),
            "-u".to_string(), must_to_string(&cookie_path),
            "upload".to_string(), must_to_string(&path),
        ];
        cmds.extend(upload_args.clone());

        // 拼接自动截图
        let image = path.with_extension("png");
        if image.exists() {
            cmds.push("--cover".to_string());
            cmds.push(must_to_string(image));
        }
        println!("上传命令: {}", cmds.join(" "));

        // 执行命令
        lazycmd::spawn(cmds)?;
    }

    Ok(())
}

