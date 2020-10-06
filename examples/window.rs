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
    let image = Image::<f32, Rgb>::open(&arg).unwrap();

    let mut windows = WindowSet::new();

    windows
        .create(&event_loop, image, WindowBuilder::new().with_title(arg))
        .unwrap();

    windows.run(&mut event_loop, move |windows, event, _, _| match event {
        Event::LoopDestroyed => return,
        Event::WindowEvent { event, window_id } => {
            if let Some(window) = windows.get_mut(&window_id) {
                match event {
                    WindowEvent::CursorMoved { .. } => {
                        println!("Mouse: {:?}", window.position);
                    }
                    WindowEvent::KeyboardInput { input, .. } => {
                        if input.state == ElementState::Pressed {
                            if let Some(VirtualKeyCode::I) = input.virtual_keycode {
                                window.image.run_in_place(filter::Invert);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        _ => (),
    });

    println!("DONE");
}
