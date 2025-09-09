use bevy::{color::palettes::css::RED, prelude::*, winit::WinitSettings};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(WinitSettings::desktop_app())
        .add_systems(Startup, setup)
        .add_systems(Update, button_system)
        .run();
}

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

#[derive(Resource)]
struct CameraState {
    moved: bool,
}

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
    mut query_camera: Query<&mut Transform, With<Camera>>,
    mut state_camera: ResMut<CameraState>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                **text = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = RED.into();

                for mut transform in &mut query_camera {
                    if !state_camera.moved {
                        *transform =
                            Transform::from_xyz(0.0, 0.0, 18.0).looking_at(Vec3::ZERO, Vec3::Y);
                    } else {
                        *transform =
                            Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y);
                    }
                }

                state_camera.moved = !state_camera.moved;
            }
            Interaction::Hovered => {
                **text = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
            }
            Interaction::None => {
                **text = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets: Res<AssetServer>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    ));
    // cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb_u8(124, 144, 255))),
        Transform::from_xyz(0.0, 0.5, 0.0),
    ));
    // light
    commands.spawn((
        PointLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
    ));

    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.insert_resource(CameraState { moved: false });

    commands.spawn(button(&assets));
}

fn button(asset_server: &AssetServer) -> impl Bundle + use<> {
    (
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
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
                Text::new("Button"),
                TextFont {
                    // font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )]
        )],
    )
}

// #[derive(Component)]
// struct Object;

// #[derive(Component)]
// struct Name(String);

// pub struct HelloPlugin;

// impl Plugin for HelloPlugin {
//     fn build(&self, app: &mut App) {
//         app.insert_resource(ObjectTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
//         app.add_systems(Startup, add_objects);
//         app.add_systems(Update, (update_objects, init_objects).chain());
//     }
// }

// fn add_objects(mut commands: Commands) {
//     commands.spawn((Object, Name("Line".to_string())));
//     commands.spawn((Object, Name("Point".to_string())));
//     commands.spawn((Object, Name("Cube".to_string())));
// }

// #[derive(Resource)]
// struct ObjectTimer(Timer);

// fn init_objects(
//     time: Res<Time>,
//     mut timer: ResMut<ObjectTimer>,
//     query: Query<&Name, With<Object>>,
// ) {
//     if timer.0.tick(time.delta()).just_finished() {
//         for name in &query {
//             println!("Initializing *{}*...", name.0);
//         }
//     }
// }

// fn update_objects(mut query: Query<&mut Name, With<Object>>) {
//     for mut name in &mut query {
//         if name.0 == "Line Point" {
//             name.0 = "Point on Line".to_string();
//             break;
//         }
//     }
// }
