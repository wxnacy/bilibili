use clap::{Parser, Subcommand};
use anyhow::Result;

use std::{fmt, path::PathBuf};

use crate::command::{
    trans, TransArgs,
    split, SplitArgs,
    init, InitArgs,
    upload, UploadArgs,
    upload_file, UploadFileArgs,
    mark, MarkArgs,
};

// `brew-cli` 客户端参数
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {

    #[clap(subcommand)]
    pub command: Command,

    /// 登录信息文件
    #[arg(short, long, default_value = "cookies.json")]
    pub user_cookie: PathBuf,

    // #[arg(long, default_value = "sqlx=debug,tower_http=debug,info")]
    #[arg(long, default_value = "tower_http=debug,info")]
    pub rust_log: String,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// 转码
    Trans {
        #[command(flatten)]
        args: TransArgs,
    },

    /// 分割
    Split {
        #[command(flatten)]
        args: SplitArgs,
    },

    /// 初始化
    Init {
        #[command(flatten)]
        args: InitArgs,
    },

    /// 上传
    Upload {
        #[command(flatten)]
        args:  UploadArgs,
    },

    /// 上传
    UploadFile {
        #[command(flatten)]
        args:  UploadFileArgs,
    },

    /// 制作
    Mark {
        #[command(flatten)]
        args:  MarkArgs,
    },

}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Trans { .. } => write!(f, "trans"),
            Command::Split { .. } => write!(f, "split"),
            Command::Init { .. } => write!(f, "init"),
            Command::Upload { .. } => write!(f, "upload"),
            Command::UploadFile { .. } => write!(f, "upload_file"),
            Command::Mark { .. } => write!(f, "mark"),
        }
    }
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Trans { args } => trans(args),
        Command::Split { args } => split(args),
        Command::Init { args } => init(args),
        Command::Upload { args } => upload(args),
        Command::UploadFile { args } => upload_file(args),
        Command::Mark { args } => mark(args),
    }
}


