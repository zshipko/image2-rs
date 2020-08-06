use image2::*;

fn main() {
    let arg: Vec<_> = std::env::args().skip(1).collect();

    let image = Image::<f32, Rgb>::open(&arg[0]).unwrap();
    let histogram = image.histogram(255);

    for (i, h) in histogram.iter().enumerate() {
        println!("{}: {:?}", i, h.bins);
    }
}
