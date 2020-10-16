use crate::*;
use filter::*;

#[cfg(feature = "oiio")]
use io::*;

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
    let mut image: Image<u8, Rgb> = Image::new((1000, 1000));
    let mut dest = image.new_like();
    image.set_f((3, 15), 0, 1.);

    let index = image.meta.index((3, 15));
    assert_eq!(image.data[index], 255);
    invert().eval(&[&image], &mut dest);
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
    timer("ToGrayscale", || convert().eval(&[&image], &mut dest));
    assert!(dest.save("images/test-grayscale.jpg").is_ok());
}

#[test]
fn test_invert() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    timer("Invert", || invert().eval(&[&image], &mut dest));
    assert!(dest.save("images/test-invert.jpg").is_ok());
}

#[test]
fn test_invert_async() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    timer("Invert async", || {
        smol::block_on(dest.apply_async(AsyncMode::Row, invert(), &[&image]));
    });
    assert!(dest.save("images/test-invert-async.jpg").is_ok());
}

#[test]
fn test_hash() {
    let a: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let b: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    println!("Hash: {}", a.hash());
    timer("Hash", || assert!(a.hash() == b.hash()));
    assert!(a.hash().diff(&b.hash()) == 0);
    let mut c = a.new_like();
    invert().eval(&[&a], &mut c);

    println!("Hash: {}", c.hash());
    assert!(c.hash() != a.hash());
    assert!(c.hash().diff(&a.hash()) != 0);
}

#[test]
fn test_kernel() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::from([[-1.0, -1.0, -1.0], [-1.0, 8.0, -1.0], [-1.0, -1.0, -1.0]]);
    timer("Kernel", || k.eval(&[&image], &mut dest));
    assert!(dest.save("images/test-simple-kernel.jpg").is_ok());
}

#[test]
fn test_gaussian_blur() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::gaussian_5x5();
    timer("Gaussian blur", || k.eval(&[&image], &mut dest));
    assert!(dest.save("images/test-gaussian-blur.jpg").is_ok());
}

#[test]
fn test_sobel() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    let k = Kernel::sobel();
    timer("Sobel", || k.eval(&[&image], &mut dest));
    assert!(dest.save("images/test-sobel.jpg").is_ok());
}

#[test]
fn test_sobel_rotate180() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest = image.new_like();
    let k = Pipeline::new()
        .then(rotate180(dest.size()))
        .then(Kernel::sobel());
    timer("Sobel rotate", || k.execute(&[&image], &mut dest));
    assert!(dest.save("images/test-sobel-rotate180-1.jpg").is_ok());

    let k = Pipeline::new()
        .then(Kernel::sobel())
        .then(rotate180(dest.size()));
    timer("Sobel rotate 2", || k.execute(&[&image], &mut dest));
    assert!(dest.save("images/test-sobel-rotate180-2.jpg").is_ok());

    let mut dest = Image::<f32, Rgb>::new((image.width() - 40, image.height() - 40));
    let k = Pipeline::new()
        .then(Kernel::sobel())
        .then(rotate180(dest.size()))
        .then(crop(Region::new(
            Point::new(20, 20),
            Size::new(image.width() - 20, image.height() - 20),
        )));
    timer("Sobel rotate crop", || k.execute(&[&image], &mut dest));
    assert!(dest.save("images/test-sobel-rotate180-crop.jpg").is_ok());

    let mut dest = Image::<f32, Rgb>::new((image.width() - 40, image.height() - 40));
    let k = Pipeline::new()
        .then(Kernel::sobel())
        .then(crop(Region::new(
            Point::new(10, 10),
            Size::new(image.width() - 10, image.height() - 10),
        )))
        .then(rotate180(dest.size()))
        .then(crop(Region::new(
            Point::new(10, 10),
            Size::new(image.width() - 10, image.height() - 10),
        )));
    timer("Sobel crop rotate crop", || k.execute(&[&image], &mut dest));
    assert!(dest
        .save("images/test-sobel-crop-rotate180-crop.jpg")
        .is_ok());
}

#[test]
fn test_crop() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest: Image<f32, Rgb> = Image::new((250, 200));
    let k = crop(Region::new(Point::new(100, 200), Size::new(250, 200)));
    timer("Crop", || k.eval(&[&image], &mut dest));
    assert!(dest.save("images/test-crop.jpg").is_ok());
}

#[test]
fn test_brightness_contrast() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let mut dest: Image<f32, Rgb> = image.new_like();
    let f = contrast(1.25);
    timer("contrast", || f.eval(&[&image], &mut dest));
    assert!(dest.save("images/test-contrast.jpg").is_ok());

    let f = Pipeline::new().then(contrast(1.25)).then(brightness(1.5));
    timer("contrast", || f.execute(&[&image], &mut dest));
    assert!(dest.save("images/test-contrast-brightness-1.jpg").is_ok());

    let f = Pipeline::new().then(brightness(1.5)).then(contrast(1.25));
    timer("contrast", || f.execute(&[&image], &mut dest));
    assert!(dest.save("images/test-contrast-brightness-2.jpg").is_ok());
}

#[test]
fn test_saturation() {
    let mut image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();

    assert!(image.save("images/test-saturation0.jpg").is_ok());

    image.run_in_place(saturation(1.25));

    assert!(image.save("images/test-saturation1.jpg").is_ok());
}

#[test]
fn test_xyz() {
    let image: Image<f32, Xyz> = Image::open("images/A.exr").unwrap();
    assert!(image.save("images/test-xyz.jpg").is_ok());
}

#[cfg(feature = "oiio")]
#[test]
fn test_convert_colorspace() {
    let image: Image<f32, Rgb> = Image::open("images/A.exr").unwrap();
    let image2 = image.convert_colorspace("srgb", "lnf").unwrap();
    let image3 = image.convert_colorspace("lnf", "srgb").unwrap();

    assert!(image2.save("images/test-convert-color1.jpg").is_ok());
    assert!(image3.save("images/test-convert-color2.jpg").is_ok());
}

#[cfg(feature = "oiio")]
#[test]
fn test_metadata() {
    let input = ImageInput::open("images/A.exr").unwrap();
    let a = input.spec().attrs();
    println!("KEYS: {:?}", a);

    let image: Image<f32, Rgb> = input.read().unwrap();
    let mut output = ImageOutput::create("images/test.exr").unwrap();
    output.spec_mut().set_attr("testing", "123");
    output.write(&image).unwrap();

    let input2 = ImageInput::open("images/test.exr").unwrap();
    let b = input2.spec().attrs();
    assert!(b.contains_key(&"testing"));
    assert!(input2.spec().get_attr("testing") == Some(Attr::String("123")));
}

#[test]
fn test_type_and_color_name() {
    assert!(f32::type_name() != f64::type_name());
    assert!(u8::type_name() == u8::type_name());
}
