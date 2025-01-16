use std::{process, time::Instant};
use bili_cli::{run, Cli};
use clap::Parser;


// fn main() {
    // let start = Instant::now();
    // let v = bili_video::Video::from("/Volumes/ZhiTai/影片/电视剧/爱情公寓/爱情公寓3/爱情公寓.S03E03.mp4");
    // println!("{v:#?}")

    // bili_video::to_ts("/Volumes/ZhiTai/Downloads/想见你/想见你.全13集.无删减.国语繁中.2019.WEB-DL.4k.H265.DDP.AAC/01.mp4", Some("/Users/wxnacy/.bilibili/cache/01")).unwrap();

    // bili_video::cut(
        // "/Volumes/ZhiTai/影片/漫威里不仅有台词和特效，还有炸裂的眼神！.mp4",
        // Some("/Volumes/ZhiTai/影片/漫威里不仅有台词和特效，还有炸裂的眼神！-cut.mp4"),
        // 0.0,
        // 20.0,
    // ).unwrap();

    // bili_video::concat(
        // [
            // "/Volumes/ZhiTai/影片/test.ts",
            // "/Volumes/ZhiTai/影片/破2.1.1.ts",
        // ],
        // "/Volumes/ZhiTai/影片/concat1.mp4",
    // ).unwrap();

    // split_and_concat(
        // // "/Volumes/ZhiTai/Downloads/想见你/想见你.全13集.无删减.国语繁中.2019.WEB-DL.4k.H265.DDP.AAC/01.mp4",
        // "/Users/wxnacy/.bilibili/cache/test.mp4",
        // // "/Volumes/ZhiTai/Downloads/S1 (2009) 4K/01.mp4",
    // ).unwrap();

    // let p = Path::new("/Volumes/ZhiTai/Downloads/想见你.mp4").with_extension("test1.mp4");
    // println!("{p:?}")

    // bili_video::transcode_1080(
        // "/Volumes/ZhiTai/影片/电视剧/想见你/想见你.全13集.无删减.国语繁中.2019.WEB-DL.4k.H265.DDP.AAC/02.mp4",
        // "/Volumes/ZhiTai/Movies/电视剧/想见你/想见你1/S01E02.mp4"
    // ).unwrap();
    
    // 计算耗时
    // let duration = start.elapsed();

    // // 打印耗时
    // println!("耗时: {:?}", duration);

// }




fn main() {
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


