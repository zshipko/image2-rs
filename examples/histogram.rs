use std::io::Write;

fn plot_hist(i: usize, hist: &image2::Histogram, max: usize) {
    use std::process::Command;

    let filename = match i {
        0 => "examples/red",
        1 => "examples/green",
        2 => "examples/blue",
        _ => unreachable!(),
    };

    let mut f = std::fs::File::create(filename).unwrap();
    for (index, value) in hist.bins() {
        write!(f, "{}, {}\n", index, value).unwrap();
    }

    let filename = format!(
        r#"set terminal png size 400,300;
        set output 'examples/histogram{}.png';
        set yrange [0:{}];
        set style histogram rowstacked gap 0;
        set style fill solid 0.5 border lt -1;
        plot "{}" smooth freq with boxes;"#,
        i, max, filename
    );

    Command::new("gnuplot")
        .arg("-e")
        .arg(&filename)
        .status()
        .unwrap();
}

fn main() {
    let arg: Vec<_> = std::env::args().skip(1).collect();

    let image = image2::Image::<f32, image2::Rgb>::open(&arg[0]).unwrap();
    let histogram = image.histogram(255);

    let mut max = 0;

    for h in histogram.iter() {
        max = h.bin(h.max_index()).max(max);
    }

    max += 10;

    for (i, h) in histogram.iter().enumerate() {
        plot_hist(i, h, max)
    }
}
