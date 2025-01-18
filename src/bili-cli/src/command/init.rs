use anyhow::Result;

use media::init_part;
use clap::{command, Parser};

/// `init` 命令的参数
#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct InitArgs {
    name: String,

}

/// `init` 命令入口
pub fn init(args: InitArgs) -> Result<()> {
    match args.name.as_str() {
        "part" => init_part()?,
        _ => eprintln!("not match init {}", &args.name),
    }
    Ok(())
}
