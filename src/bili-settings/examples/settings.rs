use std::time::Instant;

use settings::Settings;

fn main() {
    let start = Instant::now();
    let s = Settings::new().unwrap();
    println!("{s:#?}");
    // 计算耗时
    let duration = start.elapsed();

    // 打印耗时
    println!("耗时: {:?}", duration);
}
