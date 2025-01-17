use std::path::{Path, PathBuf};
use anyhow::{Result};

use crate::{cut, cut_quick, Video};


#[derive(Debug)]
pub struct Spliter {
    from: PathBuf,
    parts: usize,
    with_quick: bool,
}

impl Spliter {
    pub fn new<P>(from: P) -> Self
        where P: AsRef<Path>
    {
        Self {
            from: from.as_ref().to_path_buf(),
            parts: 4,
            with_quick: false,
        }
    }

    pub fn set_parts(&mut self, parts: usize) -> &mut Self {
        self.parts = parts;
        self
    }

    pub fn with_quick(&mut self, f: bool) -> &mut Self {
        self.with_quick = f;
        self
    }

    pub fn output<P>(&self, to: P) -> Result<Vec<PathBuf>>
        where P: AsRef<Path>
    {
        // 获取视频的总时长（假设视频时长已知或可通过其他方式获得）
        let total_duration = Video::from(&self.from)?.duration; // 你需要实现这个方法以获取视频时长

        // 计算每个部分的时长
        let part_duration = total_duration / self.parts as f64;

        // 创建一个存储输出路径的向量
        let mut output_paths = Vec::with_capacity(self.parts);

        for i in 0..self.parts {
            let start_time = i as f64 * part_duration;
            let output_path = to.as_ref().with_extension(format!("P{}.mp4", i + 1));

            // 调用切割视频的方法
            if self.with_quick {
                cut_quick(&self.from, &output_path, start_time, part_duration)?;
            } else {
                cut(&self.from, &output_path, start_time, part_duration)?;
            }
            output_paths.push(output_path);
        }

        Ok(output_paths)
    }

}


/// 分割视频
///
/// Examples
///
/// ```ignore
/// use bili_video::split;
///
/// // 拼接视频
/// split("/tmp/test.mp4", "/tmp/target.mp4", 5).unwrap()
/// ```
pub fn split<F, T>(from: F, to: T, parts: usize) -> Result<Vec<PathBuf>>
    where
        F: AsRef<Path>,
        T: AsRef<Path>,
{
    let mut s = Spliter::new(from);
    s.set_parts(parts).output(to)
}
