use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::{error, ImageBuf, Rgb};

#[derive(Debug)]
pub enum Error {
    FileDoesNotExist,
    InvalidImageShape,
    InvalidFrameCount,
}

/// Stores information about a video file
pub struct Ffmpeg {
    path: PathBuf,
    width: usize,
    height: usize,
    args: Vec<String>,
    pub(crate) frames: usize,
    pub(crate) index: usize,
}

impl Ffmpeg {
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
        let ffprobe_num_frames = Command::new("ffprobe")
            .args(&[
                "-v",
                "error",
                "-hide_banner",
                "-count_frames",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=nb_frames",
                "-of",
                "default=nokey=1:noprint_wrappers=1",
            ])
            .arg(&path)
            .output()?;

        let frames = match String::from_utf8(ffprobe_num_frames.stdout) {
            Ok(f) => f,
            Err(_) => return Err(error::Error::FFmpeg(Error::InvalidFrameCount)),
        };

        let frames = match frames.trim().parse::<usize>() {
            Ok(n) => n,
            Err(_) => return Err(error::Error::FFmpeg(Error::InvalidFrameCount)),
        };

        Ok(frames)
    }

    /// Returns the number of frames in a video file
    pub fn num_frames(&self) -> usize {
        self.frames
    }

    /// Returns the size of each frame of the video
    pub fn shape(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Open a video file using FFmpeg
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Ffmpeg, error::Error> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(error::Error::FFmpeg(Error::FileDoesNotExist));
        }

        let (width, height) = Self::get_shape(&path)?;
        let frames = Self::get_frames(&path)?;
        let args = Vec::new();

        Ok(Ffmpeg {
            path,
            width,
            height,
            frames,
            index: 0,
            args,
        })
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
    pub fn skip(&mut self, n: usize) {
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

        self.index += 1;

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
}
