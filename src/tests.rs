#![cfg(test)]
#![cfg(feature = "io")]

use crate::color::{Gray, Rgb};
use crate::filter::{Filter, Invert, ToGrayscale};
use crate::io::{magick, read, write};
use crate::kernel::{gaussian_5x5, sobel, Kernel};
use crate::{Image, ImageBuf, Pixel};

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
    Invert.eval(&mut dest, &[&image]);
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
fn test_read_write_magick() {
    let a: ImageBuf<u16, Rgb> = magick::read("test/test.jpg").unwrap();
    magick::write("test/test-read-write-magick0.jpg", &a).unwrap();
    magick::write("test/test-read-write-magick1.png", &a).unwrap();

    let b: ImageBuf<u16, Rgb> = magick::read("test/test-read-write-magick1.png").unwrap();
    magick::write("test/test-read-write-magick2.png", &b).unwrap();
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
    timer("Invert", || Invert.eval(&mut dest, &[&image]));
    write("test/test-invert.jpg", &dest).unwrap();
}

#[test]
fn test_hash() {
    let a: ImageBuf<f32, Rgb> = read("test/test.jpg").unwrap();
    let b: ImageBuf<f32, Rgb> = read("test/test.jpg").unwrap();
    timer("Hash", || assert!(a.hash() == b.hash()));
    assert!(a.hash().diff(&b.hash()) == 0);
    let mut c = a.new_like();
    Invert.eval(&mut c, &[&a]);
    assert!(c.hash() != a.hash());
    assert!(c.hash().diff(&a.hash()) != 0);
}

#[test]
fn test_kernel() {
    let image: ImageBuf<f32, Gray> = read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);
    timer("Kernel", || k.eval(&mut dest, &[&image]));
    write("test/test-simple-kernel.jpg", &dest).unwrap();
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
fn test_diff() {
    let image: ImageBuf<u8, Rgb> = read("test/test.jpg").unwrap();
    let mut image2: ImageBuf<u8, Rgb> = image.new_like();
    let diff = image.diff(&image2);
    assert!(diff.len() > 0);
    diff.apply(&mut image2);
    let diff2 = image.diff(&image2);
    assert!(diff2.len() == 0);
    assert!(image == image2);
    write("test/test-diff.png", &image2).unwrap()
}

#[test]
fn test_colorspace() {
    let image: ImageBuf<u8, Rgb> = read("test/test.jpg").unwrap();
    let mut px = image.empty_pixel();
    image.get_pixel(10, 10, &mut px);
    let rgb = Pixel::<u8, Rgb>::to_rgb(&px);
    println!("{:?}", rgb);
}
