#![cfg(test)]

use color::{Gray, Rgb, Yuv};
use filter::{Filter, Invert, ToGrayscale};
use io::magick;
use kernel::{sobel, Kernel};
use test::Bencher;
use {Image, ImageBuf};

#[test]
fn test_image_buffer_new() {
    let mut image: ImageBuf<u8, Rgb> = ImageBuf::new(1000, 1000);
    let mut dest = image.new_like();
    image.set(3, 15, 0, 1.);
    assert_eq!(image.get(3, 15, 0), 1.);
    Invert.eval_s(&mut dest, &[&image]);
}

#[bench]
fn bench_magick_read(b: &mut Bencher) {
    b.iter(|| {
        let _: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    });
}

#[bench]
fn bench_magick_write(b: &mut Bencher) {
    let a: ImageBuf<f32, Yuv> = magick::read("test/test.jpg").unwrap();
    b.iter(|| {
        magick::write("test0.jpg", &a).unwrap();
    });
}

#[bench]
fn test_to_grayscale(b: &mut Bencher) {
    let image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    b.iter(|| ToGrayscale.eval(&mut dest, &[&image]));
    magick::write("test1.jpg", &dest).unwrap();
}

#[bench]
fn bench_invert(b: &mut Bencher) {
    let image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    b.iter(|| Invert.eval_s(&mut dest, &[&image]));
    magick::write("test2.jpg", &dest).unwrap();
}

#[bench]
fn bench_invert_parallel(b: &mut Bencher) {
    let image: ImageBuf<f32, Rgb> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    b.iter(|| Invert.eval(&mut dest, &[&image]));
    magick::write("test2p.jpg", &dest).unwrap();
}

#[bench]
fn bench_kernel(b: &mut Bencher) {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);

    b.iter(|| k.eval_s(&mut dest, &[&image]));
    magick::write("test3.jpg", &dest).unwrap();
}

#[bench]
fn bench_kernel_parallel(b: &mut Bencher) {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);

    b.iter(|| k.eval(&mut dest, &[&image]));
    magick::write("test3p.jpg", &dest).unwrap();
}

#[bench]
fn bench_sobel(b: &mut Bencher) {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    let mut dest = image.new_like();
    let k = sobel();
    b.iter(|| k.eval(&mut dest, &[&image]));
    magick::write("test4.jpg", &dest).unwrap();
}

#[bench]
fn bench_mean_stddev(b: &mut Bencher) {
    let image: ImageBuf<f32, Gray> = magick::read("test/test.jpg").unwrap();
    b.iter(|| {
        image.mean_stddev();
    });
}
