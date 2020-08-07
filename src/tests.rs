use crate::*;
use filter::*;

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
    let mut image: Image<u8, Rgb> = Image::new(1000, 1000);
    let mut dest = image.new_like();
    image.set_f(3, 15, 0, 1.);

    let index = image.index(3, 15);
    assert_eq!(image.data[index], 255);
    Invert.eval(&mut dest, &[&image]);
}

#[test]
fn test_read_write() {
    let a: Image<u8, Rgb> = Image::open("images/A.exr").unwrap();
    assert!(a.save("images/test-read-write0.jpg").is_ok());
    assert!(a.save("images/test-read-write1.png").is_ok());

    let b: Image<u8, Rgb> = Image::open("images/test-read-write1.png").unwrap();
    assert!(b.save("images/test-read-write2.png").is_ok());
}

#[test]
fn test_read_write_rgba() {
    let a: Image<u16, Rgba> = Image::open("images/A.exr").unwrap();
    assert!(a.save("images/test-read-write-rgba0.jpg").is_ok());
    assert!(a.save("images/test-read-write-rgba1.png").is_ok());

    let b: Image<u16, Rgb> = Image::open("images/test-read-write-rgba1.png").unwrap();
    assert!(b.save("images/test-read-write-rgba2.png").is_ok());
}

#[test]
fn test_to_grayscale() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest: Image<f32, Gray> = image.new_like_with_type_and_color::<f32, Gray>();
    timer("ToGrayscale", || {
        Convert::<Gray>::new().eval(&mut dest, &[&image])
    });
    assert!(dest.save("images/test-grayscale.jpg").is_ok());
}

#[test]
fn test_invert() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    timer("Invert", || Invert.eval(&mut dest, &[&image]));
    assert!(dest.save("images/test-invert.jpg").is_ok());
}

#[test]
fn test_invert_async() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    timer("Invert async", || {
        smol::run(filter::eval_async(
            &Invert,
            filter::AsyncMode::Row,
            &mut dest,
            &[&image],
        ))
    });
    assert!(dest.save("images/test-invert-async.jpg").is_ok());
}

#[test]
fn test_hash() {
    let a: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let b: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    timer("Hash", || assert!(a.hash() == b.hash()));
    assert!(a.hash().diff(&b.hash()) == 0);
    let mut c = a.new_like();
    Invert.eval(&mut c, &[&a]);
    assert!(c.hash() != a.hash());
    assert!(c.hash().diff(&a.hash()) != 0);
}

#[test]
fn test_kernel() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);
    timer("Kernel", || k.eval(&mut dest, &[&image]));
    assert!(dest.save("images/test-simple-kernel.jpg").is_ok());
}

#[test]
fn test_gaussian_blur() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    let k = kernel::gaussian_5x5();
    timer("Gaussian blur", || k.eval(&mut dest, &[&image]));
    assert!(dest.save("images/test-gaussian-blur.jpg").is_ok());
}

#[test]
fn test_sobel() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    let k = kernel::sobel();
    timer("Sobel", || k.eval(&mut dest, &[&image]));
    assert!(dest.save("images/test-sobel.jpg").is_ok());
}

#[test]
fn test_convert_color() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let image2 = image.convert_colorspace("srgb", "lnf").unwrap();
    let image3 = image.convert_colorspace("lnf", "srgb").unwrap();

    assert!(image2.save("images/test-convert-color1.jpg").is_ok());
    assert!(image3.save("images/test-convert-color2.jpg").is_ok());
}

/*#[test]
fn test_diff() {
    let image: ImageBuf<u8, Rgb> = read("images/A.exr").unwrap();
    let mut image2: ImageBuf<u8, Rgb> = image.new_like();
    let diff = image.diff(&image2);
    assert!(diff.len() > 0);
    diff.apply(&mut image2);
    let diff2 = image.diff(&image2);
    assert!(diff2.len() == 0);
    assert!(image == image2);
    write("images/test-diff.png", &image2).unwrap()
}

#[test]
fn test_colorspace() {
    let image: ImageBuf<u8, Rgb> = read("images/A.exr").unwrap();
    let mut px = image.empty_pixel();
    image.get_pixel(10, 10, &mut px);
    let rgb = Pixel::<u8, Rgb>::to_rgb(&px);
    println!("{:?}", rgb);
}*/
