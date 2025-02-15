use std::{fs, path::PathBuf};

use anyhow::{anyhow, Result};

use clap::{command, Parser};
use lazytool::{path::must_to_string, time};
use media::MediaSettings;
use settings::Settings;
use regex::Regex;
use serde_json::Value;
use std::collections::HashMap;


use super::model::EpisodeArgs;
/// `upload` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct Uploader {
    // 地址
    #[arg(short, long, help="地址", default_value_t)]
    pub path: String,

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

    // 上传 up
    #[arg(long, help="up mid")]
    pub mid: Option<u64>,

    // 是否添加到
    #[arg(short('A'), long, help="是否快速分离")]
    pub with_append: bool,

    // vid bvid or aid
    #[arg(short, long, help="视频id", default_value_t)]
    pub vid: String,
}

impl Uploader {

    pub fn path(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    pub fn fill_with_media(&mut self, media: &MediaSettings, season: u16, episode: u16) -> &mut Self {
        if let Some(uploader) = media.get_uploader(season, episode) {
            if self.dtime.is_empty() {
                if let Some(dtime) = uploader.dtime {
                    self.dtime = dtime.clone();
                }
            }

            if self.tag.is_empty() {
                if let Some(tag) = uploader.tag {
                    self.tag = tag.clone();
                }
            }
        }
        if self.tag.is_empty() {
            self.tag = media.title.clone();
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

    pub fn to_cmds(&self, with_append: bool) -> Result<Vec<String>> {
        let settings = Settings::new()?;
        let up = settings.get_up(self.mid).expect("Failed get up");
        println!("上传 UP: {}({})", &up.name, &up.mid);
        let cookie_path = up.get_cookie_path();
        let mut subcmd = "upload".to_string();
        if with_append {
            subcmd = "append".to_string();
        }
        let mut cmds = vec![
            "biliup".to_string(),
            "-u".to_string(), must_to_string(&cookie_path),
            subcmd, must_to_string(&self.path),
        ];
        if with_append {
            cmds.push("-v".to_string());
            cmds.push(self.vid.clone());
        }
        cmds.extend(self.to_args()?);
        println!("上传命令: {}", cmds.join(" "));
        Ok(cmds)
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


}


/// `split` 命令入口
pub fn upload(args: UploadArgs) -> anyhow::Result<()> {
    // 名称和短名必须有一个
    if args.ep.name.is_empty() && args.ep.title.is_empty() {
        return Err(anyhow!("name or title must has value"));
    }

    let media = MediaSettings::new(&args.ep.get_name().expect("failed get name"))?;
    let mut upload = args.upload.clone();
    upload.fill_with_media(&media, args.ep.season, args.ep.episode);

    let mut ep = args.ep.clone();
    if ep.title.is_empty() {
        ep.title = media.title.clone();
    }

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


    for (i, path) in paths.into_iter().enumerate() {
        println!("bvid: {}", &upload.vid);
        if i > 0 && upload.vid.is_empty() && args.upload.with_append {
            return Err(anyhow!("not found bvid"));
        }
        upload.path = must_to_string(&path);

        // 拼接自动截图
        let image = path.with_extension("png");
        if image.exists() {
            upload.cover = must_to_string(image);
        }

        println!("upload: {:#?}", &upload);
        // 执行命令
        let mut cmds = upload.to_cmds(false)?;
        if i > 0 && args.upload.with_append {
            cmds = upload.to_cmds(true)?;
        }

        let results = lazycmd::spawn(cmds)?;
        // println!("{results:#?}");

        if upload.vid.is_empty() {
            for line in results {
                if line.contains("code") && line.contains("message") {
                    if let Some(res) = extract_json_from_log(&line) {
                        if let Some(data) = res.get("data") {
                            if let Some(b) = data.get("bvid") {
                                upload.vid = b.to_string().replace("\"", "");
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}


fn extract_json_from_log(log: &str) -> Option<HashMap<String, Value>> {
    // 定义正则表达式来匹配 JSON 部分
    let re = Regex::new(r"\{.*\}").unwrap();

    // 使用正则表达式查找 JSON
    if let Some(captures) = re.captures(log) {
        // 获取匹配的字符串
        let json_str = &captures[0];

        // 解析 JSON 字符串为 HashMap
        let parsed: HashMap<String, Value> = serde_json::from_str(json_str).ok()?;

        Some(parsed)
    } else {
        None
    }
}
