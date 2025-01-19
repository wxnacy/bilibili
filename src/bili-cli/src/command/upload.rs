use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};

use clap::{command, Parser};
use lazytool::{path::must_to_string, time};
use media::MediaSettings;
use settings::Settings;


use super::model::EpisodeArgs;
/// `upload` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Uploader {
    // 封面
    #[arg(short, long, help="封面", default_value_t)]
    pub cover: String,

    // 标签
    #[arg(long, help="标签。用逗号隔开", default_value_t)]
    pub tag: String,

    // 分区
    #[arg(long, help="分区。默认影视剪辑", default_value = "183")]
    pub tid: u32,

    // 单视频文件最大并发数
    #[arg(short, long, help="单视频文件最大并发数", default_value = "8")]
    pub limit: u8,

    // 预发布时间
    #[arg(long, help="预发布时间", default_value_t)]
    pub dtime: String,

    // 描述
    #[arg(short, long, help="描述", default_value_t)]
    pub desc: String,
}

impl Uploader {

    pub fn fill_with_media(&mut self, media: &MediaSettings, season: u16, episode: u16) -> &mut Self {
        if self.tag.is_empty() {
            self.tag = media.title.clone();
        }
        if let Some(uploader) = media.get_uploader(season, episode) {
            if self.dtime.is_empty() {
                self.dtime = uploader.dtime.clone();
            }
        }
        self
    }

    pub fn to_args(&self) -> Result<Vec<String>> {
        let mut args = vec![
            String::from("--limit"), format!("{}", self.limit),
            String::from("--tid"), format!("{}", self.tid),
        ];
        if !self.cover.is_empty() {
            args.push("--cover".to_string());
            args.push(self.cover.to_string());
        }

        if !self.tag.is_empty() {
            args.push("--tag".to_string());
            args.push(self.tag.to_string());
        }

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

/// `upload` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct UploadArgs {

    #[command(flatten)]
    pub ep: EpisodeArgs,

    #[command(flatten)]
    pub upload: Uploader,

    // 上传 up
    #[arg(long, help="up mid")]
    pub mid: Option<u64>,
}


/// `split` 命令入口
pub fn upload(args: UploadArgs) -> anyhow::Result<()> {
    // 名称和短名必须有一个
    if args.ep.name.is_empty() && args.ep.title.is_empty() {
        return Err(anyhow!("name or title must has value"));
    }

    let media = MediaSettings::new(&args.ep.name)?;
    let mut upload = args.upload.clone();
    upload.fill_with_media(&media, args.ep.season, args.ep.episode);

    let mut ep = args.ep.clone();
    if ep.title.is_empty() {
        ep.title = media.title.clone();
    }
    let stg = Settings::new()?;
    let up = stg.get_up(args.mid).expect("Get up failed");

    let cache_dir = ep.get_cache_dir()?;
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
        cmds.extend(upload.to_args()?);

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

