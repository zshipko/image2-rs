use bevy::prelude::*;
use image2::*;

fn system(
    mut commands: Commands,
    mut assets: ResMut<Assets<Texture>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let image = Image::<f32, Rgba>::open("images/A.exr").unwrap();
    let texture: Texture = image.into();
    let texture = assets.add(texture);

    commands
        .spawn(Camera2dComponents::default())
        .spawn(SpriteComponents {
            material: materials.add(texture.into()),
            ..Default::default()
        });
}

fn main() {
    App::build()
        .add_default_plugins()
        .add_startup_system(system.system())
        .run();
}
