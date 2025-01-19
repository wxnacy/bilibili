
use anyhow::{anyhow, Result};

use clap::{command, Parser};
use lazytool::path::must_to_string;


use super::{model::EpisodeArgs, upload::Uploader};

/// `upload` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct UploadFileArgs {

    #[command(flatten)]
    pub upload: Uploader,

    // 上传 up
    // #[arg(long, help="up mid")]
    // pub mid: Option<u64>,
}


/// `split` 命令入口
pub fn upload_file(args: UploadFileArgs) -> anyhow::Result<()> {
    let mut upload = args.upload.clone();
    if !upload.path().exists() {
        return Err(anyhow!("path: {} not found", &upload.path));
    }
    if upload.cover.is_empty() {
        let image = upload.path().with_extension("png");
        if image.exists() {
            upload.cover = must_to_string(&image);
        }
    }

    let cmds = upload.to_cmds()?;
    lazycmd::spawn(cmds)?;

    Ok(())
}

