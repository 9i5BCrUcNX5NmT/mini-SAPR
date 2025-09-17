use bevy::{
    color::palettes::css::WHITE,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        RenderPlugin,
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
    },
    text::FontSmoothing,
    winit::WinitSettings,
};

use crate::{
    buttons::{button_system, setup_buttons},
    grid::{GridSettings, setup_grid, update_grid_system},
    orbit_camera::{OrbitCamera, OrbitCenter, orbit_camera_system, toggle_orbit_mode_system},
    render::{
        RenderModes, toggle_lighting_system, toggle_render_mode_system, update_materials_system,
    },
};

mod buttons;
mod grid;
mod orbit_camera;
mod render;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(RenderPlugin {
                render_creation: RenderCreation::Automatic(WgpuSettings {
                    // Включаем поддержку wireframe для desktop платформ
                    features: WgpuFeatures::POLYGON_MODE_LINE,
                    ..default()
                }),
                ..default()
            }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        // Here we define size of our overlay
                        font_size: 42.0,
                        // If we want, we can use a custom font
                        font: default(),
                        // We could also disable font smoothing,
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    // We can also set the refresh interval for the FPS counter
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
            WireframePlugin::default(),
        ))
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(OrbitCenter::default())
        .insert_resource(OrbitCamera::default())
        .insert_resource(GridSettings::default())
        .insert_resource(RenderModes::default())
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })
        .add_systems(Startup, (setup, setup_grid, setup_buttons))
        .add_systems(
            Update,
            (
                button_system,
                orbit_camera_system,
                toggle_orbit_mode_system,
                update_grid_system,
                toggle_render_mode_system,
                update_materials_system,
                toggle_lighting_system,
            ),
        )
        .run();
}

#[derive(Component)]
struct Id(u32);

#[derive(Resource)]
struct CameraState {
    moved: bool,
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(materials.add(Color::WHITE)),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
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
}
