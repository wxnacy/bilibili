
use anyhow::{anyhow, Result};

use clap::{command, Parser};
use lazytool::path::must_to_string;


use super::upload::Uploader;

/// `upload` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct UploadFileArgs {

    pub filename: String,

    #[command(flatten)]
    pub upload: Uploader,

}


/// `split` 命令入口
pub fn upload_file(args: UploadFileArgs) -> Result<()> {
    let mut upload = args.upload.clone();
    upload.path = args.filename.clone();
    if !upload.path().exists() {
        return Err(anyhow!("path: {} not found", &upload.path));
    }
    if upload.tag.is_empty() {
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

