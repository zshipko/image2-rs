use std::io::Write;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::{error, Image, ImageBuf, Rgb};

#[derive(Debug)]
pub enum Error {
    FileDoesNotExist,
    InvalidImageShape,
    InvalidFrameCount,
}

/// Stores information about a video file
pub struct FfmpegIn {
    path: PathBuf,
    width: usize,
    height: usize,
    args: Vec<String>,
    pub(crate) frames: usize,
    pub(crate) index: usize,
}

#[derive(Debug)]
pub struct FfmpegOut {
    path: PathBuf,
    proc: std::process::Child,
}

impl FfmpegOut {
    pub fn open<P: AsRef<Path>>(
        path: P,
        width: usize,
        height: usize,
        framerate: usize,
        args: Option<Vec<&str>>,
    ) -> Result<FfmpegOut, std::io::Error> {
        let path = path.as_ref().to_path_buf();

        let size = format!("{}x{}", width, height);
        let framerate_s = format!("{}", framerate);
        let proc = Command::new("ffmpeg")
            .args(&[
                "-y",
                "-v",
                "error",
                "-hide_banner",
                "-f",
                "rawvideo",
                "-pixel_format",
                "rgb24",
                "-video_size",
                &size,
                "-framerate",
                &framerate_s,
                "-i",
                "-",
                path.to_str().expect("Invalid output file"),
            ])
            .args(args.unwrap_or(Vec::new()))
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(FfmpegOut { path, proc })
    }

    pub fn output_file(&self) -> &PathBuf {
        &self.path
    }

    pub fn write<I: Image<u8, Rgb>>(&mut self, image: &I) -> Result<(), std::io::Error> {
        let mut stdin = self.proc.stdin.take().unwrap();
        stdin.write_all(image.data())?;
        self.proc.stdin.replace(stdin);
        Ok(())
    }

    pub fn finish(mut self) -> Result<(), std::io::Error> {
        let mut stdin = self.proc.stdin.take().unwrap();
        stdin.flush()?;
        Ok(())
    }
}

impl FfmpegIn {
    fn get_shape(path: &PathBuf) -> Result<(usize, usize), error::Error> {
        let ffprobe_size = Command::new("ffprobe")
            .args(&[
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=s=x:p=0",
            ])
            .arg(path)
            .output()?;

        let shape = match String::from_utf8(ffprobe_size.stdout) {
            Ok(shape) => shape,
            Err(_) => return Err(error::Error::FFmpeg(Error::InvalidImageShape)),
        };

        let t = shape
            .split('x')
            .map(|a| a.trim().parse::<usize>())
            .collect::<Vec<Result<usize, ParseIntError>>>();

        if t.len() < 2 {
            return Err(error::Error::FFmpeg(Error::InvalidImageShape));
        }

        let x = match (&t[0], &t[1]) {
            (Ok(w), Ok(h)) => (w.clone(), h.clone()),
            (_, _) => return Err(error::Error::FFmpeg(Error::InvalidImageShape)),
        };

        Ok(x)
    }

    fn get_frames(path: &PathBuf) -> Result<usize, error::Error> {
        let ffprobe_num_frames = Command::new("ffmpeg")
            .args(&[
                "-i",
                path.to_str().expect("Invalid Filename"),
                "-map",
                "0:v:0",
                "-c",
                "copy",
                "-f",
                "null",
                "-y",
                "/dev/null",
            ])
            .output()?;

        let frames = match String::from_utf8(ffprobe_num_frames.stderr) {
            Ok(f) => f,
            Err(_) => return Err(error::Error::FFmpeg(Error::InvalidFrameCount)),
        };

        let frames: Vec<&str> = frames.split("\r").map(|x| x.trim()).collect();

        let mut nframes = 0;

        for f in frames.iter() {
            if !f.contains("frame=") {
                continue;
            }

            let line: Vec<&str> = f.split("frame=").collect();
            let count: String = line[1].split(" ").take(1).collect();

            let frames = match count.parse::<usize>() {
                Ok(n) => n,
                Err(_) => return Err(error::Error::FFmpeg(Error::InvalidFrameCount)),
            };

            if frames > nframes {
                nframes = frames
            }
        }

        if nframes == 0 {
            return Err(error::Error::FFmpeg(Error::InvalidFrameCount));
        }

        Ok(nframes)
    }

    /// Returns the number of frames in a video file
    pub fn num_frames(&self) -> usize {
        self.frames
    }

    /// Returns the size of each frame of the video
    pub fn shape(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub fn limit_frames(&mut self, n: usize) {
        if self.frames > n {
            self.frames = n
        }
    }

    /// Open a video file using FFmpeg
    pub fn open<P: AsRef<Path>>(path: P) -> Result<FfmpegIn, error::Error> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(error::Error::FFmpeg(Error::FileDoesNotExist));
        }

        let (width, height) = Self::get_shape(&path)?;
        let frames = Self::get_frames(&path)?;
        let args = Vec::new();

        Ok(FfmpegIn {
            path,
            width,
            height,
            frames,
            index: 0,
            args,
        })
    }

    pub fn input_file(&self) -> &PathBuf {
        &self.path
    }

    /// Add FFmpeg argument
    pub fn arg<S: AsRef<str>>(&mut self, arg: S) {
        self.args.push(String::from(arg.as_ref()))
    }

    /// Set frame index to 0
    pub fn reset(&mut self) {
        self.index = 0;
    }

    /// Skip `n` frames
    pub fn skip_frames(&mut self, n: usize) {
        self.index += n;
    }

    /// Rewind `n`n frames
    pub fn rewind(&mut self, n: usize) {
        if n < self.index {
            self.index = 0;
        } else {
            self.index -= 1;
        }
    }

    /// Get next frame
    pub fn next(&mut self) -> Option<ImageBuf<u8, Rgb>> {
        if self.index >= self.frames {
            return None;
        }

        self.index += 1;

        let cmd = Command::new("ffmpeg")
            .args(&["-v", "error", "-hide_banner", "-i"])
            .arg(&self.path)
            .arg("-vf")
            .arg(format!("select=gte(n\\,{})", self.index))
            .args(&[
                "-vframes", "1", "-pix_fmt", "rgb24", "-f", "rawvideo", "-an", "-",
            ])
            .args(&self.args)
            .output();

        let cmd = match cmd {
            Ok(x) => x,
            _ => return None,
        };

        Some(ImageBuf::new_from(self.width, self.height, cmd.stdout))
    }

    /// Get next `n` frames
    pub fn next_n(&mut self, n: usize) -> Vec<ImageBuf<u8, Rgb>> {
        let mut v = Vec::new();

        for _ in 0..n {
            match self.next() {
                Some(x) => v.push(x),
                None => break,
            }
        }

        v
    }

    pub fn process_to<F: Fn(usize, ImageBuf<u8, Rgb>) -> ImageBuf<u8, Rgb>>(
        &mut self,
        mut dest: FfmpegOut,
        f: F,
    ) -> Result<(), crate::Error> {
        for (i, frame) in self.enumerate() {
            let frame = f(i, frame);
            dest.write(&frame)?;
        }
        dest.finish()?;
        Ok(())
    }
}

impl Iterator for FfmpegIn {
    type Item = ImageBuf<u8, Rgb>;

    fn next(&mut self) -> Option<Self::Item> {
        FfmpegIn::next(self)
    }
}

pub fn open_in<P: AsRef<Path>>(path: P) -> Result<FfmpegIn, crate::Error> {
    FfmpegIn::open(path)
}

pub fn open_out<P: AsRef<Path>>(
    path: P,
    width: usize,
    height: usize,
    framerate: usize,
    args: Option<Vec<&str>>,
) -> Result<FfmpegOut, std::io::Error> {
    FfmpegOut::open(path, width, height, framerate, args)
}
