use bevy::prelude::*;
use image2::*;

use bevy::prelude::Color as BColor;

struct ButtonMaterials {
    normal: Handle<ColorMaterial>,
    hovered: Handle<ColorMaterial>,
    pressed: Handle<ColorMaterial>,
}

impl FromResources for ButtonMaterials {
    fn from_resources(resources: &Resources) -> Self {
        let mut materials = resources.get_mut::<Assets<ColorMaterial>>().unwrap();
        ButtonMaterials {
            normal: materials.add(BColor::rgb(1., 1., 1.).into()),
            hovered: materials.add(BColor::rgb(0.8, 0.8, 0.8).into()),
            pressed: materials.add(BColor::rgb(0.0, 0.0, 0.0).into()),
        }
    }
}

fn invert(
    button_materials: Res<ButtonMaterials>,
    mut window: ResMut<ui::ImageView<f32, Rgba>>,
    mut query: Query<(
        &mut Button,
        &mut Handle<ColorMaterial>,
        Mutated<Interaction>,
    )>,
) {
    for (_button, mut material, interaction) in &mut query.iter() {
        match *interaction {
            Interaction::Clicked => {
                let src = window.image.clone();
                window.image_mut().apply(filter::Invert, &[&src]);
                window.mark_as_dirty();
                *material = button_materials.pressed;
            }
            Interaction::Hovered => {
                *material = button_materials.hovered;
            }
            Interaction::None => {
                *material = button_materials.normal;
            }
        }
    }
}

fn button(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    button_materials: Res<ButtonMaterials>,
) {
    commands
        .spawn(ButtonComponents {
            style: Style {
                size: ui::bevy::Size::new(Val::Px(100.), Val::Px(65.0)),
                position_type: PositionType::Absolute,
                margin: Rect::all(Val::Auto),
                position: Rect {
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..Default::default()
                },
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..Default::default()
            },

            material: button_materials.normal,
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextComponents {
                text: Text {
                    value: "Invert".to_string(),
                    font: asset_server
                        .load("./examples/OpenSans-Regular.ttf")
                        .unwrap(),
                    style: TextStyle {
                        font_size: 24.0,
                        color: bevy::prelude::Color::rgb(0., 0., 0.),
                    },
                    ..Default::default()
                },
                ..Default::default()
            });
        });
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
        image.set_gamma_lin();
    }

    App::build()
        .add_default_plugins()
        .init_resource::<ButtonMaterials>()
        .add_startup_system(ui::init.system())
        .add_startup_system(button.system())
        .add_plugin(ui::ImageView::new(image))
        .add_system(invert.system())
        .run();
}
