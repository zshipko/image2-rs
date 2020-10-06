use image2::window::*;
use image2::*;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let arg = if !args.is_empty() {
        args[0].as_str()
    } else {
        "images/A.exr"
    };

    let image = Image::<f32, Rgb>::open(arg).unwrap();

    let evloop = EventLoop::new();

    let mut window = Window::new(&evloop, image, WindowBuilder::new().with_title(arg)).unwrap();

    evloop.run(move |event, _, cf| {
        *cf = ControlFlow::Poll;
        match event {
            Event::LoopDestroyed => return,
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => {
                    *cf = ControlFlow::Exit;
                }
                WindowEvent::CursorMoved { position, .. } => {
                    println!(
                        "Mouse: {:?}",
                        window.mouse_position((position.x as usize, position.y as usize))
                    );
                }
                _ => (),
            },
            Event::RedrawRequested(_) => window.draw().unwrap(),
            _ => (),
        }
    });
}
