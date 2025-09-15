use bevy::prelude::*;

use crate::{CameraState, Id};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
// const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
// const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub fn button_system(
    mut interaction_query: Query<(&Interaction, &Id), (Changed<Interaction>, With<Button>)>,
    mut query_camera: Query<&mut Transform, With<Camera>>,
    mut state_camera: ResMut<CameraState>,

    // to spawn cube
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (interaction, id) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => match id.0 {
                1 => {
                    for mut transform in &mut query_camera {
                        if !state_camera.moved {
                            *transform =
                                Transform::from_xyz(0.0, 18.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
                        } else {
                            *transform =
                                Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y);
                        }
                    }

                    state_camera.moved = !state_camera.moved;
                }
                2 => {
                    // cube
                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
                        Transform::from_xyz(0.0, 0.5, 0.0),
                    ));
                }
                _ => {}
            },
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

fn spawn_button(name: String, id: u32, x: f32, y: f32) -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(x),
            height: Val::Percent(y),
            align_items: AlignItems::Baseline,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(75.0),
                height: Val::Px(32.5),
                border: UiRect::all(Val::Px(5.0)),
                // horizontally center child text
                justify_content: JustifyContent::Center,
                // vertically center child text
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new(name),
                TextFont {
                    // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )],
            Id(id),
        )],
    )
}

pub fn setup_buttons(mut commands: Commands) {
    commands.spawn(spawn_button("Camera".into(), 1, 100.0, 100.0));
    commands.spawn(spawn_button("Spawn Cube".into(), 2, 50.0, 50.0));
}
