use std::{process, time::Instant};
use bili_cli::{run, Cli};
use clap::Parser;


#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let start = Instant::now();
    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        process::exit(1);
    }
    // 计算耗时
    let duration = start.elapsed();

    // 打印耗时
    println!("耗时: {:?}", duration);
}


