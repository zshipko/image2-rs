use bevy::prelude::*;
use image2::*;

fn invert(
    mut window: ResMut<ui::ImageView<f32, Rgba>>,
    mut query: Query<(&mut Button, Mutated<Interaction>)>,
) {
    for (_button, interaction) in &mut query.iter() {
        match *interaction {
            Interaction::Clicked => {
                let src = window.image.clone();
                window.image_mut().apply(filter::Invert, &[&src]);
                window.mark_as_dirty();
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn button(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands
        .spawn(ButtonComponents {
            style: Style {
                size: Size::new(Val::Px(100.), Val::Px(65.0)),
                position_type: PositionType::Absolute,
                // center button
                margin: Rect::all(Val::Auto),
                position: Rect {
                    right: Val::Px(0.0),
                    bottom: Val::Px(0.0),
                    ..Default::default()
                },
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|parent| {
            parent.spawn(TextComponents {
                text: Text {
                    value: "Invert".to_string(),
                    font: asset_server.load("examples/ubuntu.ttf").unwrap(),
                    style: TextStyle {
                        font_size: 28.0,
                        color: bevy::prelude::Color::rgb(0.1, 0.1, 0.1),
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
        .add_startup_system(ui::init.system())
        .add_startup_system(button.system())
        .add_plugin(ui::ImageView::new(image))
        .add_system(invert.system())
        .run();
}
