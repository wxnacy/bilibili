extern crate ffmpeg_next as ffmpeg;
use serde::{Serialize, Deserialize};

use std::{error::Error, fmt, fs, path::Path};

#[derive(Debug)]
/// 定义视频的错误类型
pub struct VideoError(String);

impl fmt::Display for VideoError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for VideoError {}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Video {
    pub width: u32,
    pub height: u32,
    pub size: u64,
    pub duration: f64,
    pub format: Option<String>,
    pub path: String,
}

impl fmt::Display for Video {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Video[{} {} {} {} {:?}]",
            self.width, self.height, self.size, self.duration, self.format
        )
    }
}

/// 视频信息结构
impl Video {
    /// 通过地址新建 Video 对象
    ///
    /// Examples
    ///
    /// ```
    /// use bili_video::Video;
    ///
    /// Video::from("examples/data/trailer.mp4")
    /// ```
    pub fn from<P: AsRef<Path>>(path: P) -> anyhow::Result<Video> {

        let file = path.as_ref();

        if !file.exists() {
            return Err(anyhow::anyhow!("file: {:?} not found", file));
        }

        // 通过 ffmpeg 获取视频信息
        let mut video = Self::by_ffmpeg(file)?;

        let meta = fs::metadata(file)?;
        video.size = meta.len();
        if let Some(_path) = file.to_str() {
            video.path = _path.to_string();
        }
        Ok(video)
    }

    /// 通过 ffmpeg 获取视频信息
    fn by_ffmpeg<P: AsRef<Path>>(file: P) -> Result<Video, ffmpeg::Error> {
        use ffmpeg::error::Error::Other as OtherErr;
        use ffmpeg::media::Type::Video as TypeVideo;
        ffmpeg::init()?;
        let content = ffmpeg::format::input(&file)?;
        // 如果没有视频流直接报错
        if let Some(_stream) = content.streams().best(TypeVideo) {
        } else {
            return Err(OtherErr { errno: 0 });
        }
        let mut video = Video::default();

        for stream in content.streams() {
            // 获取最长的时间
            let dur = stream.duration() as f64 * f64::from(stream.time_base());
            if dur > video.duration {
                video.duration = dur;
            }

            // 获取视频基本信息
            let codec = ffmpeg::codec::context::Context::from_parameters(stream.parameters())?;
            if codec.medium() == TypeVideo {
                if let Ok(v) = codec.decoder().video() {
                    video.width = v.width();
                    video.height = v.height();
                    let format = v.format().descriptor().unwrap().name();
                    video.format = Some(format.to_string());
                }
            }
        }

        Ok(video)
    }
}
