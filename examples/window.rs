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
        move |windows, event| {
            match event {
                Event::LoopDestroyed => return Some(ControlFlow::Exit),
                Event::WindowEvent { event, window_id } => {
                    let window = windows.get_mut(&window_id)?;
                    match event {
                        WindowEvent::CursorMoved { .. } => {
                            println!("Mouse: {:?}", window.mouse_position());
                        }
                        WindowEvent::KeyboardInput { input, .. } => {
                            if input.state != ElementState::Pressed {
                                return None;
                            }

                            if let Some(VirtualKeyCode::I) = input.virtual_keycode {
                                window.image_mut().run_in_place(filter::invert());
                                window.mark_as_dirty();
                            }
                        }
                        _ => (),
                    }
                }
                _ => (),
            };
            None
        },
    )
    .unwrap();

    println!("DONE");
}
