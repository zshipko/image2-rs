use image2::window::*;
use image2::*;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let arg = if !args.is_empty() {
        args[0].clone()
    } else {
        "images/A.exr".to_string()
    };

    let image = Image::<f32, Rgb>::open(&arg).unwrap();

    println!("Press 'i' to invert the image");
    show_all(
        vec![("A", image.clone()), ("B", image)],
        move |window, event| {
            match event {
                Some(Event::CursorPos(x, y)) => {
                    println!("Mouse: {x} {y}");
                }
                Some(Event::Scroll(x, y)) => {
                    println!("Scroll: {x} {y}");
                }
                Some(Event::Key(Key::I, _, Action::Press, _)) => {
                    window.image_mut().run(filter::invert());
                }
                _ => (),
            }

            Ok(())
        },
    )
    .unwrap();

    println!("DONE");
}
