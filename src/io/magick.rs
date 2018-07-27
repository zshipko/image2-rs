use std::io::Write;
use std::num::ParseIntError;
use std::path::Path;
use std::process::{Command, Stdio};
use std::usize;

use color::Color;
use image::{Image, ImageBuf};
use ty::Type;

#[derive(Debug)]
pub enum Error {
    InvalidImageShape,
    InvalidColor,
    FileDoesNotExist,
    InvalidImageData,
    UnableToExecuteCommand,
    ErrorWritingImage,
}

pub struct Magick {
    identify: &'static [&'static str],
    convert: &'static [&'static str],
}

pub fn kind<C: Color>() -> &'static str {
    match C::channels() {
        1 => "gray:-",
        3 => "rgb:-",
        4 => "rgba:-",
        n => panic!("Invalid number of channels for io::magick image: {}", n),
    }
}

pub const IMAGEMAGICK: Magick = Magick {
    identify: &["identify"],
    convert: &["convert"],
};

pub const GRAPHICSMAGICK: Magick = Magick {
    identify: &["gm", "identify"],
    convert: &["gm", "convert"],
};

impl Magick {
    pub fn get_image_shape<P: AsRef<Path>>(&self, path: P) -> Result<(usize, usize), Error> {
        let identify = Command::new(self.identify[0])
            .args(self.identify[1..].iter())
            .arg(path.as_ref())
            .output();

        let shape = match identify {
            Ok(shape) => shape,
            Err(_) => return Err(Error::InvalidImageShape),
        };

        let shape = match String::from_utf8(shape.stdout) {
            Ok(shape) => shape,
            Err(_) => return Err(Error::InvalidImageShape),
        };

        let t = shape
            .split(" ")
            .skip(2)
            .take(1)
            .collect::<String>()
            .split("x")
            .map(|a| a.parse::<usize>())
            .collect::<Vec<Result<usize, ParseIntError>>>();

        let a = t[0].clone();
        let b = t[1].clone();

        match (a, b) {
            (Ok(a), Ok(b)) => Ok((a, b)),
            (Err(_), _) | (_, Err(_)) => Err(Error::InvalidImageShape),
        }
    }

    pub fn read<P: AsRef<Path>, T: Type, C: Color>(
        &self,
        path: P,
    ) -> Result<ImageBuf<T, C>, Error> {
        let (width, height) = match self.get_image_shape(&path) {
            Ok((width, height)) => (width, height),
            Err(e) => return Err(e),
        };
        let kind = kind::<C>();

        let cmd = Command::new(self.convert[0])
            .args(self.convert[1..].iter())
            .arg(path.as_ref())
            .args(&["-depth", "8"])
            .arg(kind)
            .output();

        let cmd = match cmd {
            Ok(cmd) => cmd,
            Err(_) => return Err(Error::InvalidImageData),
        };

        let data = cmd.stdout.iter().map(|x| x.convert()).collect::<Vec<T>>();

        Ok(ImageBuf::new_from(width, height, data))
    }

    pub fn write<P: AsRef<Path>, T: Type, C: Color, I: Image<T, C>>(
        &self,
        path: P,
        image: &I,
    ) -> Result<(), Error> {
        let kind = kind::<C>();
        let (width, height, _) = image.shape();
        let size = format!("{}x{}", width, height);
        let cmd = Command::new(self.convert[0])
            .args(self.convert[1..].iter())
            .stdin(Stdio::piped())
            .args(&["-depth", "8"])
            .args(&["-size", size.as_str()])
            .arg(kind)
            .arg(path.as_ref())
            .spawn();

        let mut proc = match cmd {
            Ok(c) => c,
            Err(_) => return Err(Error::UnableToExecuteCommand),
        };

        {
            let mut stdin = proc.stdin.take().unwrap();
            let wdata = image
                .data()
                .iter()
                .map(|x| x.convert())
                .collect::<Vec<u8>>();
            match stdin.write_all(&wdata) {
                Ok(()) => (),
                Err(_) => return Err(Error::ErrorWritingImage),
            }
            let _ = stdin.flush();
        }

        let res = match proc.wait() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::UnableToExecuteCommand),
        };
        res
    }
}

pub fn read<P: AsRef<Path>, T: Type, C: Color>(path: P) -> Result<ImageBuf<T, C>, Error> {
    IMAGEMAGICK.read(path)
}

pub fn write<P: AsRef<Path>, T: Type, C: Color, I: Image<T, C>>(
    path: P,
    image: &I,
) -> Result<(), Error> {
    IMAGEMAGICK.write(path, image)
}
