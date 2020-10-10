use std::io::{Read, Write};
use std::num::ParseIntError;
use std::path::Path;
use std::process::{Command, Stdio};
use std::usize;

use crate::{Color, Image, Rgb, Type};

/// Magick I/O errors
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error("Invalid image shape")]
    InvalidImageShape,

    #[error("Invalid color")]
    InvalidColor,

    #[error("File does not exist")]
    FileDoesNotExist,

    #[error("Invalid image data")]
    InvalidImageData,

    #[error("Unable to execute command")]
    UnableToExecuteCommand,

    #[error("Error writing image")]
    ErrorWritingImage,
}

/// Magick command struct
pub struct Magick {
    identify: &'static [&'static str],
    convert: &'static [&'static str],
}

fn kind<C: Color>() -> String {
    format!("{}:-", C::NAME)
}

fn depth<T: Type, C: Color>(cmd: &mut Command) {
    let depth = std::mem::size_of::<T>() * 8;
    cmd.arg("-depth");
    cmd.arg(format!("{}", depth));

    if T::is_float() {
        cmd.args(&["-define", "quantum:format=floating-point"]);
    }
}

/// ImageMagick
pub const IM: Magick = Magick {
    identify: &["identify"],
    convert: &["convert"],
};

/// GraphicsMagick
pub const GM: Magick = Magick {
    identify: &["gm", "identify"],
    convert: &["gm", "convert"],
};

/// Default Magick implementation
pub static mut DEFAULT: Magick = IM;

/// Change default command
pub fn set_default(magick: Magick) {
    unsafe {
        DEFAULT = magick;
    }
}

impl Magick {
    /// Get size of image using identify command
    pub fn get_image_shape<P: AsRef<Path>>(&self, path: P) -> Result<(usize, usize), Error> {
        let identify = Command::new(self.identify[0])
            .args(self.identify[1..].iter())
            .args(&["-format", "%w %h"])
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
            .split(' ')
            .map(|a| a.trim().parse::<usize>())
            .collect::<Vec<Result<usize, ParseIntError>>>();

        if t.len() < 2 {
            return Err(Error::InvalidImageShape);
        }

        match (&t[0], &t[1]) {
            (Ok(a), Ok(b)) => Ok((a.clone(), b.clone())),
            (Err(_), _) | (_, Err(_)) => Err(Error::InvalidImageShape),
        }
    }

    /// Read image from disk using ImageMagick/GraphicsMagick
    pub fn read<P: AsRef<Path>, T: Type, C: Color>(&self, path: P) -> Result<Image<T, C>, Error> {
        if !["gray", "rgb", "rgba"].contains(&C::NAME) {
            return Ok(self.read::<P, f32, Rgb>(path)?.convert());
        }

        let (width, height) = match self.get_image_shape(&path) {
            Ok((width, height)) => (width, height),
            Err(e) => return Err(e),
        };

        let kind = kind::<C>();
        let mut cmd = Command::new(self.convert[0]);
        cmd.args(self.convert[1..].iter()).arg(path.as_ref());
        depth::<T, C>(&mut cmd);
        cmd.arg(kind);

        let cmd = match cmd.output() {
            Ok(cmd) => cmd,
            Err(_) => return Err(Error::InvalidImageData),
        };

        if cmd.stdout.len() != std::mem::size_of::<T>() * width * height * C::CHANNELS {
            return Err(Error::InvalidImageData);
        }

        Ok(Image {
            meta: crate::Meta::new((width, height)),
            data: unsafe {
                let mut data: Vec<T> = std::mem::transmute(cmd.stdout);
                data.set_len(width * height * C::CHANNELS);
                data.into()
            },
        })
    }

    /// Write image to disk using ImageMagick/GraphicsMagick
    pub fn write<P: AsRef<Path>, T: Type, C: Color>(
        &self,
        path: P,
        image: &Image<T, C>,
    ) -> Result<(), Error> {
        if !["gray", "rgb", "rgba"].contains(&C::NAME) {
            let image = image.convert::<T, Rgb>();
            return self.write(path, &image);
        }
        let kind = kind::<C>();
        let (width, height, _) = image.shape();
        let size = format!("{}x{}", width, height);
        let mut cmd = Command::new(self.convert[0]);
        cmd.args(self.convert[1..].iter()).stdin(Stdio::piped());
        depth::<T, C>(&mut cmd);
        cmd.args(&["-size", size.as_str()])
            .arg(kind)
            .arg(path.as_ref());

        let mut proc = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => return Err(Error::UnableToExecuteCommand),
        };

        {
            let mut stdin = proc.stdin.take().unwrap();
            match stdin.write_all(image.buffer()) {
                Ok(()) => (),
                Err(_) => return Err(Error::ErrorWritingImage),
            }
            let _ = stdin.flush();
        }

        match proc.wait() {
            Ok(_) => Ok(()),
            Err(_) => Err(Error::UnableToExecuteCommand),
        }
    }

    /// Encode image to an im-memory buffer using ImageMagick/GraphicsMagick
    pub fn encode<T: Type, C: Color>(
        &self,
        format: &str,
        image: &Image<T, C>,
    ) -> Result<Vec<u8>, Error> {
        let kind = kind::<C>();
        let (width, height, _) = image.shape();
        let size = format!("{}x{}", width, height);
        let mut cmd = Command::new(self.convert[0]);
        cmd.args(self.convert[1..].iter())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped());
        depth::<T, C>(&mut cmd);
        cmd.args(&["-size", size.as_str()])
            .arg(&kind)
            .arg(format!("{}:-", format));

        let mut proc = match cmd.spawn() {
            Ok(c) => c,
            Err(_) => return Err(Error::UnableToExecuteCommand),
        };

        {
            let mut stdin = proc.stdin.take().unwrap();
            match stdin.write_all(image.buffer()) {
                Ok(()) => (),
                Err(_) => return Err(Error::ErrorWritingImage),
            }
            let _ = stdin.flush();
        }

        match proc.wait() {
            Ok(_) => {
                let mut buffer: Vec<u8> = Vec::new();
                match proc.stdout.unwrap().read_to_end(&mut buffer) {
                    Ok(_) => (),
                    Err(_) => return Err(Error::InvalidImageData),
                }
                Ok(buffer)
            }
            Err(_) => Err(Error::UnableToExecuteCommand),
        }
    }
}

/// Read image from disk using default command-line tool
pub fn read<P: AsRef<Path>, T: Type, C: Color>(path: P) -> Result<Image<T, C>, Error> {
    unsafe { DEFAULT.read(path) }
}

/// Write image to disk using default command-line tool
pub fn write<P: AsRef<Path>, T: Type, C: Color>(path: P, image: &Image<T, C>) -> Result<(), Error> {
    unsafe { DEFAULT.write(path, image) }
}
