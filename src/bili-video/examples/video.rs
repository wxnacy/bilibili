use bili_video::Video;
use anyhow::Result;
fn main() -> Result<()>{
    let video = Video::from("examples/data/trailer.mp4")?;
    println!("{video:#?}");
    Ok(())
}
