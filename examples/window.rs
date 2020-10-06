use image2::window::*;
use image2::*;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let arg = if !args.is_empty() {
        args[0].clone()
    } else {
        "images/A.exr".to_string()
    };

    let mut event_loop = EventLoop::new();
    let mut image = Image::<f32, Rgb>::open(&arg).unwrap();

    image.draw_line(
        (10, 10),
        (400, 250),
        1,
        &Pixel::from_slice(&[1.0, 0.0, 0.0]),
    );

    let mut windows = WindowSet::new();

    windows
        .create(&event_loop, image, WindowBuilder::new().with_title(arg))
        .unwrap();

    windows.run(&mut event_loop, move |windows, event, _, _| match event {
        Event::LoopDestroyed => return,
        Event::WindowEvent { event, window_id } => {
            if let Some(window) = windows.get(&window_id) {
                match event {
                    WindowEvent::CursorMoved { position, .. } => {
                        println!("Mouse: {:?}", window.mouse_position(position));
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    });

    println!("DONE");
}
