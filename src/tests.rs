#![cfg(test)]

use crate::color::{Gray, Rgb};
use crate::filter::{Filter, Invert, ToGrayscale};
use crate::io::{ffmpeg, read, write};
use crate::kernel::{gaussian_5x5, sobel, Kernel};
use crate::{Image, ImageBuf};

use std::time::Instant;

fn timer<F: FnMut()>(name: &str, mut f: F) {
    let now = Instant::now();
    f();
    let t = now.elapsed();
    println!(
        "BENCHMARK {}: {}s",
        name,
        t.as_secs() as f64 + (t.subsec_millis() as f64 * 0.001)
    )
}

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
    let a: ImageBuf<u8, Rgb> = read("test/test.jpg").unwrap();
    write("test/test-read-write0.jpg", &a).unwrap();
    write("test/test-read-write1.png", &a).unwrap();

    let b: ImageBuf<u8, Rgb> = read("test/test-read-write1.png").unwrap();
    write("test/test-read-write2.png", &b).unwrap();
}

#[test]
fn test_to_grayscale() {
    let image: ImageBuf<f32, Rgb> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    timer("ToGrayscale", || ToGrayscale.eval(&mut dest, &[&image]));
    write("test/test-grayscale.jpg", &dest).unwrap();
}

#[test]
fn test_invert() {
    let image: ImageBuf<f32, Rgb> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    timer("Invert", || Invert.eval_s(&mut dest, &[&image]));
    write("test/test-invert.jpg", &dest).unwrap();
}

#[test]
fn test_invert_parallel() {
    let image: ImageBuf<f32, Rgb> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    timer("Invert parallel", || Invert.eval(&mut dest, &[&image]));
    write("test/test-invert-parallel.jpg", &dest).unwrap();
}

#[test]
fn test_kernel() {
    let image: ImageBuf<f32, Gray> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);
    timer("Kernel", || k.eval_s(&mut dest, &[&image]));
    write("test/test-simple-kernel.jpg", &dest).unwrap();
}

#[test]
fn test_kernel_parallel() {
    let image: ImageBuf<f32, Gray> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);
    timer("Kernel parallel", || k.eval(&mut dest, &[&image]));
    write("test/test-simple-kernel-parallel.jpg", &dest).unwrap();
}

#[test]
fn test_gaussian_blur() {
    let image: ImageBuf<f32, Rgb> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = gaussian_5x5();
    timer("Gaussian blur", || k.eval(&mut dest, &[&image]));
    write("test/test-gaussian-blur.jpg", &dest).unwrap();
}

#[test]
fn test_sobel() {
    let image: ImageBuf<f32, Gray> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = sobel();
    timer("Sobel", || k.eval(&mut dest, &[&image]));
    write("test/test-sobel.jpg", &dest).unwrap();
}

#[test]
fn test_ffmpeg() {
    let path = std::path::PathBuf::from("test/test.mp4");

    if !path.exists() {
        return;
    }

    let mut ffmpeg = ffmpeg::Ffmpeg::open("test/test.mp4").unwrap();

    let image = ffmpeg.next();

    assert!(image != None);
    assert!(ffmpeg.index == 1);

    match image {
        Some(image) => write("test/test-ffmpeg.png", &image).unwrap(),
        None => (),
    }

    ffmpeg.skip(ffmpeg.frames - 1);

    assert!(ffmpeg.next() == None);
}
