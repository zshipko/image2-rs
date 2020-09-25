use bevy::prelude::*;
use image2::*;

pub struct Window<T: 'static + Type, C: 'static + image2::Color> {
    image: Image<T, C>,
    handle: Option<Handle<Texture>>,
    timer: Timer,
}

impl<T: 'static + Type, C: 'static + image2::Color> Window<T, C> {
    pub fn new(image: Image<T, C>) -> Window<T, C> {
        Window {
            image,
            handle: None,
            timer: Timer::from_seconds(1.0, true),
        }
    }
}

impl<T: 'static + Type, C: 'static + image2::Color> Plugin for Window<T, C> {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(startup.system())
            .add_system(update.system());
    }
}

fn startup(
    mut commands: Commands,
    assets: ResMut<Assets<Texture>>,
    materials: ResMut<Assets<ColorMaterial>>,
    image: Res<Image<f32, Rgba>>,
) {
    let (handle, system) = image.show_clone(assets, materials);
    commands
        .spawn(Camera2dComponents::default())
        .spawn(system)
        .insert_resource(handle);
}

pub struct Refresh(Timer);

fn update(
    time: Res<Time>,
    mut assets: ResMut<Assets<Texture>>,
    mut timer: ResMut<Refresh>,
    image: Res<Image<f32, Rgba>>,
    handle: Res<Handle<Texture>>,
) {
    timer.0.tick(time.delta_seconds);

    if timer.0.finished {
        if let Some(texture) = assets.get_mut(&handle) {
            image.update_texture(texture);
        }
    }
}

fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let arg = if !args.is_empty() {
        args[0].as_str()
    } else {
        "images/A.exr"
    };

    let mut image = Image::<f32, Rgba>::open(arg).unwrap();
    if args.is_empty() {
        image.gamma_lin();
    }

    App::build()
        .add_resource(image)
        .add_resource(Refresh(Timer::from_seconds(1.0, true)))
        .add_default_plugins()
        .add_startup_system(startup.system())
        .add_system(update.system())
        .run();
}
