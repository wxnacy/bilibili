use std::{env, fs, path::{Path, PathBuf}, process, time::Instant};
use bili_cli::{run, Cli};
use clap::Parser;

use bili_cli::create_cache_dir;


fn split_and_concat<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    let filename = path.as_ref().file_name().unwrap();
    let cache = create_cache_dir(filename)?;
    let split_target = cache.join(filename);
    let split_paths = bili_video::split(&path, &split_target, 4)?;
    println!("{split_paths:#?}");
    let mut concat_rs: Vec<String> = Vec::new();
    for sp in split_paths {
        let ts = bili_video::to_ts(sp, None)?;
        concat_rs.push(ts.to_str().unwrap().to_string());
        concat_rs.push("/Volumes/Getea/bili_cli/part/longmen/龙1.14.1.ts".to_string());
    }

    let target = cache.join("concat.mp4");
    bili_video::concat(&concat_rs, &target.to_str().unwrap().to_string())?;
    Ok(())
}

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


