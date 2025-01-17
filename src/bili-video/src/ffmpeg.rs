use std::{ffi::OsStr, fs, path::{Path, PathBuf}, time::{SystemTime, UNIX_EPOCH}};

use anyhow::{anyhow, Result};
use lazytool::path::must_to_string;

use crate::Video;

/// 视频转为 ts
///
/// Examples
///
/// ```ignore
/// use bili_video::to_ts;
///
/// to_ts("/tmp/test.mp4", None).unwrap()
/// ```
pub fn to_ts<P: AsRef<Path>>(from: P, to: Option<P>) -> Result<PathBuf> {
    let from_path = get_path_string(&from)?;
    let to_path = get_to(&from, to.as_ref(), "ts")?;
    let codec = get_codec(from)?;
    let bsf_filter = match codec.as_str() {
            "h264" => "h264_mp4toannexb",
            "hevc" => "hevc_mp4toannexb",
            _ => {
                return Err(anyhow!("Unsupported codec: {}", codec));
            }
        };
    let cmds = [
        "ffmpeg",
        "-i", &from_path,
        "-codec", "copy",
        "-bsf:v", bsf_filter,
        "-f", "mpegts",
        &to_path
    ];
    lazycmd::spawn(cmds)?;
    Ok(PathBuf::from(to_path))
}

/// 截取视频
///
/// Examples
///
/// ```ignore
/// use bili_video::cut;
///
/// // 截取开始的 10 秒
/// cut("/tmp/test.mp4", None, 0, 10).unwrap()
/// ```
pub fn cut<F, T>(from: F, to: T, start: f64, time: f64) -> Result<PathBuf>
    where
        F: AsRef<Path>,
        T: AsRef<Path>,
{
    let from_path = get_path_string(&from)?;
    let to_path = to.as_ref().to_str().unwrap().to_string();
    let cmds = [
        "ffmpeg",
        "-i", &from_path,
        "-ss", &float_to_time_format(start),
        "-t", &float_to_time_format(time),
        "-c", "copy",
        &to_path];
    let cmds = [
        "ffmpeg",
        "-ss", &float_to_time_format(start),
        "-t", &float_to_time_format(time),
        "-i", &from_path,
        "-copyts",
        &to_path];
    lazycmd::spawn(cmds)?;
    Ok(to.as_ref().to_path_buf())
}

/// 合并视频
///
/// Examples
///
/// ```ignore
/// use bili_video;
///
/// // 拼接视频
/// bili_video::concat(
///     [
///         "/tmp/test1.ts",
///         "/tmp/test2.ts",
///     ],
///     "/tmp/test.mp4",
/// ).unwrap();
/// ```
pub fn concat<I, P>(from: I, to: P) -> Result<PathBuf>
    where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let to_path = to.as_ref().to_str().unwrap().to_string();
    let cache_dir = to.as_ref().parent().unwrap();
    let concat_path = cache_dir.join(format!("concat-{}", get_timestamp()));
    let mut concat = String::new();
    for f in from {
        let fp = get_path_string(&f)?;
        concat.push_str(format!("file '{}'\n", &fp).as_str());
    }
    fs::write(&concat_path, concat)?;
    // cmd = f"ffmpeg -f concat -safe 0 -i {tmpfile} -c copy -bsf:a aac_adtstoasc {output}"
    let cmds = [
        "ffmpeg",
        "-f", "concat",
        "-safe", "0",
        "-i", &get_path_string(&concat_path)?,
        "-c", "copy",
        "-bsf:a", "aac_adtstoasc",
        // "-movflags", "+faststart",
        &to_path];
    lazycmd::spawn(cmds)?;
    fs::remove_file(concat_path)?;
    Ok(to.as_ref().to_path_buf())
}

/// 转码成 1080p 固定格式视频
pub fn transcode_1080<F, T>(from: F, to: T) -> Result<()>
    where
        F: AsRef<Path>,
        T: AsRef<Path>,
{
    let from_path = lazytool::path::must_to_string(from);
    let to_path = lazytool::path::must_to_string(to);

    let cmds = [
        "ffmpeg",
        "-i", &from_path,
        "-c:v", "libx264",
        "-preset", "veryfast",
        "-maxrate", "17185k",
        "-bufsize", "34370k",
        "-crf", "23",
        "-r", "25",
        "-s", "1920x1080",
        "-c:a", "aac",
        "-b:a", "319k",
        "-ar", "48000",
        "-ac", "2",
        &to_path,
    ];
    lazycmd::spawn(cmds)?;
    Ok(())
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
pub fn split<F, T>(from: F, to: T, num_parts: usize) -> Result<Vec<PathBuf>>
    where
        F: AsRef<Path>,
        T: AsRef<Path>,
{
    // 获取视频的总时长（假设视频时长已知或可通过其他方式获得）
    let total_duration = Video::from(&from)?.duration; // 你需要实现这个方法以获取视频时长

    // 计算每个部分的时长
    let part_duration = total_duration / num_parts as f64;

    // 创建一个存储输出路径的向量
    let mut output_paths = Vec::with_capacity(num_parts);

    for i in 0..num_parts {
        let start_time = i as f64 * part_duration;
        let output_path = to.as_ref().with_extension(format!("P{}.mp4", i + 1));

        // 调用切割视频的方法
        cut(&from, &output_path, start_time, part_duration)?;
        output_paths.push(output_path);
    }

    Ok(output_paths)
}

/// 将时间浮点数转为时间格式字符串
///
/// Examples
///
/// ```
/// let out = bili_video::float_to_time_format(1.1);
/// assert_eq!(out, "00:00:01");
///
/// let out = bili_video::float_to_time_format(67.1);
/// assert_eq!(out, "00:01:07");
///
/// let out = bili_video::float_to_time_format(3671.1);
/// assert_eq!(out, "01:01:11");
/// ```
pub fn float_to_time_format(seconds: f64) -> String {
    let total_seconds = seconds.abs() as u64; // 取绝对值并转换为无符号整数
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let secs = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}

/// 截图
///
/// Examples
///
/// ```
/// ```
pub fn screenshot<F, T>(from: F, to: T, start: f64) -> Result<PathBuf>
    where
        F: AsRef<Path>,
        T: AsRef<Path>,
{

    let cmd = format!(
        "ffmpeg -i {} -ss {} -vframes 1 -q:v 1 {}",
        must_to_string(from),
        float_to_time_format(start),
        must_to_string(&to),
    );
    lazycmd::spawn(cmd.split(" "))?;
    Ok(to.as_ref().to_path_buf())
}


/// 获取视频 codec_name
pub fn get_codec<P: AsRef<Path>>(path: P) -> Result<String> {
    let path = get_path_string(path)?;
    let args = [
        "ffprobe",
        "-v", "error",
        "-select_streams", "v:0",
        "-show_entries", "stream=codec_name",
        "-of", "default=noprint_wrappers=1:nokey=1",
        &path,
    ];
    lazycmd::output(args)
}

fn get_path_string<P: AsRef<Path>>(from: P) -> Result<String>{
    let from = from.as_ref();
    if !from.exists() {
        return Err(anyhow!("{:?} not foud", from));
    }
    Ok(from.to_str().unwrap().to_string())
}

fn get_to<P: AsRef<Path>>(from: P, to: Option<P>, ext: &str) -> Result<String>{
    let from = from.as_ref();
    if !from.exists() {
        return Err(anyhow!("{:?} not foud", from));
    }
    let to_path: String;
    if let Some(_to) = to {
        to_path = _to.as_ref().to_str().unwrap().to_string();
    } else {
        to_path = from.with_extension(ext).to_str().unwrap().to_string();
    }

    Ok(to_path)
}

fn get_timestamp() -> u64 {
    let start = SystemTime::now();

    // 计算自 UNIX 纪元以来的持续时间
    let duration = start.duration_since(UNIX_EPOCH)
        .expect("时间错误");

    // 获取时间戳（秒和毫秒）
    duration.as_secs()
}
