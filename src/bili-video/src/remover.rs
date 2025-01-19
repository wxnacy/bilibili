use std::{fs, path::{Path, PathBuf}};
use anyhow::Result;

use crate::{concat, cut, cut_quick, to_ts, Video};

#[derive(Debug)]
pub struct Remover {
    path: PathBuf,
    segments: Vec<(u64, u64)>,
    with_quick: bool,
}

impl Remover {
    pub fn new<P>(path: P, segments: Vec<(u64, u64)>) -> Self
        where P: AsRef<Path>
    {
        Self { path: path.as_ref().to_path_buf(), segments, with_quick: false }
    }

    pub fn with_quick(&mut self, f: bool) -> &mut Self {
        self.with_quick = f;
        self
    }

    /// 删除指定片段留下其余片段
    ///
    /// Examples
    ///
    /// ```
    /// use bili_video::Remover;
    ///
    /// let parts = Remover::remove_segments(1000, vec![(0, 200), (800, 1100)]);
    /// assert_eq!(parts, vec![(200, 800)]);
    ///
    /// let parts = Remover::remove_segments(1000, vec![(0, 200)]);
    /// assert_eq!(parts, vec![(200, 1000)]);
    ///
    /// let parts = Remover::remove_segments(1000, vec![(800, 1100)]);
    /// assert_eq!(parts, vec![(0, 800)]);
    ///
    /// let parts = Remover::remove_segments(1000, vec![(15, 200), (800, 900)]);
    /// assert_eq!(parts, vec![(0, 15), (200, 800), (900, 1000)]);
    /// ```
    pub fn remove_segments(video_length: u64, segments: Vec<(u64, u64)>) -> Vec<(u64, u64)> {
        let mut remaining_segments = Vec::new();
        let mut last_end = 0;

        for (start, end) in segments {
            if start > last_end {
                remaining_segments.push((last_end, start));
            }
            last_end = end;
        }

        if last_end < video_length {
            remaining_segments.push((last_end, video_length));
        }

        remaining_segments
    }

    pub fn output<P>(&self, to: P) -> Result<PathBuf>
        where P: AsRef<Path>
    {
        // 获取视频的总时长（假设视频时长已知或可通过其他方式获得）
        let total_duration = Video::from(&self.path)?.duration;
        let leave_parts = Self::remove_segments(total_duration as u64, self.segments.clone());

        let mut ts_slice: Vec<PathBuf> = Vec::new();
        for (index, part) in leave_parts.iter().enumerate() {
            let to_part = to.as_ref().with_extension(format!("{}.mp4", index));
            if self.with_quick {
                cut_quick(&self.path, &to_part, part.0 as f64, (part.1 - part.0) as f64)?;
            } else {
                cut(&self.path, &to_part, part.0 as f64, (part.1 - part.0) as f64)?;
            }

            let ts = to_ts(&to_part, None)?;
            fs::remove_file(to_part)?;

            ts_slice.push(ts);
        }

        concat(&ts_slice, &to.as_ref().to_path_buf())?;

        for ts in ts_slice {
            fs::remove_file(ts)?;
        }

        Ok(to.as_ref().to_path_buf())
    }
}


