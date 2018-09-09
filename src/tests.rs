#![cfg(test)]

use color::{Gray, Rgb};
use filter::{Filter, Invert, ToGrayscale};
use io::{png, jpg, magick};
use kernel::{sobel, Kernel};
use {Image, ImageBuf, Layout};

#[test]
fn test_image_buffer_new() {
    let mut image: ImageBuf<u8, Rgb> = ImageBuf::new(1000, 1000);
    let mut dest = image.new_like();
    image.set_f(3, 15, 0, 1.);
    assert_eq!(image.get(3, 15, 0), 255);
    Invert.eval_s(&mut dest, &[&image]);
}

#[test]
fn test_read_write() {
    let a: ImageBuf<u8, Rgb> = jpg::read("test/test.jpg").unwrap();
    magick::write("test0.jpg", &a).unwrap();
    png::write("test0a.png", &a).unwrap();

    let b: ImageBuf<u8, Rgb> = png::read("test0.png").unwrap();
    png::write("test0b.png", &b).unwrap();
}

#[test]
fn test_to_grayscale() {
    let image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    ToGrayscale.eval(&mut dest, &[&image]);
    magick::write("test1.jpg", &dest).unwrap();
}

#[test]
fn test_invert() {
    let image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    Invert.eval_s(&mut dest, &[&image]);
    magick::write("test2.jpg", &dest).unwrap();
}

#[test]
fn test_invert_parallel() {
    let image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    Invert.eval(&mut dest, &[&image]);
    magick::write("test2p.jpg", &dest).unwrap();
}

#[test]
fn test_kernel() {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);
    k.eval_s(&mut dest, &[&image]);
    magick::write("test3.jpg", &dest).unwrap();
}

#[test]
fn test_kernel_parallel() {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);
    k.eval(&mut dest, &[&image]);
    magick::write("test3p.jpg", &dest).unwrap();
}

#[test]
fn test_sobel() {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = sobel();
    k.eval(&mut dest, &[&image]);
    magick::write("test4.jpg", &dest).unwrap();
}

#[test]
fn test_mean_stddev() {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    println!("{:?}", image.mean_stddev());
}

#[test]
fn test_convert_to_planar() {
    let image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    image.clone().convert_layout(Layout::Planar);
    magick::write("test5.jpg", &image).unwrap();
}

#[test]
fn test_convert_layout_rountrip() {
    let mut image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    image.convert_layout(Layout::Planar);
    magick::write("test_layout_planar.jpg", &image).unwrap();
    image.convert_layout(Layout::Interleaved);
    magick::write("test_layout_rountrip.jpg", &image).unwrap();
}


