use bevy::prelude::*;
use image2::*;

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let arg = if !args.is_empty() {
        args[0].as_str()
    } else {
        "images/A.exr"
    };

    let mut image = Image::<f32, Rgba>::open(arg).unwrap().scale(0.5, 0.5);
    if args.is_empty() {
        image.set_gamma_lin();
    }

    App::build()
        .add_default_plugins()
        .add_plugin(ui::ImageView::new(image))
        .run();
}
