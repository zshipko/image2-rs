use std::io::{Read, Write};
use std::process::{Child, Command, Stdio};

use crate::Image;

#[derive(Debug)]
pub enum Error {
    CannotSpawnChild,
    FileDoesNotExist,
    InvalidImageShape,
    InvalidFrameCount,
}

pub struct FFmpeg {
    command: Command,
    child: Option<Child>,
}

impl std::fmt::Debug for FFmpeg {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.command, fmt)
    }
}

impl FFmpeg {
    pub fn new() -> FFmpeg {
        let mut command = Command::new("ffmpeg");
        command.args(&["-v", "error", "-y"]);
        FFmpeg {
            command,
            child: None,
        }
    }

    pub fn is_running(&self) -> bool {
        match self.child {
            Some(_) => true,
            None => false,
        }
    }

    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut FFmpeg {
        self.command.arg(arg);
        self
    }

    pub fn input_file<S: AsRef<std::ffi::OsStr>>(&mut self, filename: S) -> &mut FFmpeg {
        self.arg("-i").arg(filename)
    }

    pub fn input_pipe(&mut self) -> &mut FFmpeg {
        self.arg("-f").arg("rawvideo").input_file("-")
    }

    pub fn fps(&mut self, i: usize) -> &mut FFmpeg {
        self.arg("-r").arg(format!("{}", i))
    }

    pub fn output_file<S: AsRef<std::ffi::OsStr>>(&mut self, filename: S) -> &mut FFmpeg {
        self.arg(filename)
    }

    pub fn output_pipe(&mut self) -> &mut FFmpeg {
        self.arg("-f").arg("rawvideo").output_file("-")
    }

    pub fn pix_fmt(&mut self, fmt: &str) -> &mut FFmpeg {
        self.arg("-pix_fmt").arg(fmt)
    }

    pub fn video_size(&mut self, width: usize, height: usize) -> &mut FFmpeg {
        self.arg("-video_size").arg(format!("{}x{}", width, height))
    }

    pub fn start_read(&mut self) -> Result<(), Error> {
        let child = match self
            .command
            .stderr(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return Err(Error::CannotSpawnChild),
        };

        self.child = Some(child);

        Ok(())
    }

    pub fn start_write(&mut self) -> Result<(), Error> {
        let child = match self
            .command
            .stderr(Stdio::null())
            .stdin(Stdio::piped())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => return Err(Error::CannotSpawnChild),
        };

        self.child = Some(child);

        Ok(())
    }

    pub fn stop(&mut self, wait: bool) {
        let child = self.child.take();

        match child {
            Some(mut c) => {
                if wait {
                    c.wait().unwrap();
                } else {
                    c.kill().unwrap();
                }
            }
            None => (),
        }
    }

    pub fn read<T: crate::Type, C: crate::Color, I: Image<T, C>>(
        &mut self,
        image: &mut I,
    ) -> std::io::Result<()> {
        let len = image.data().len() * std::mem::size_of::<T>();
        let buf =
            unsafe { std::slice::from_raw_parts_mut(image.data_mut().as_ptr() as *mut u8, len) };
        match &mut self.child {
            Some(c) => match &mut c.stdout {
                Some(x) => x.read_exact(buf),
                None => Ok(()),
            },
            None => Ok(()),
        }
    }

    pub fn write<T: crate::Type, C: crate::Color, I: Image<T, C>>(
        &mut self,
        image: &I,
    ) -> std::io::Result<()> {
        let len = image.data().len() * std::mem::size_of::<T>();
        let buf = unsafe { std::slice::from_raw_parts(image.data().as_ptr() as *const u8, len) };
        match &mut self.child {
            Some(c) => match &mut c.stdin {
                Some(x) => x.write_all(buf),
                None => Ok(()),
            },
            None => Ok(()),
        }
    }
}
