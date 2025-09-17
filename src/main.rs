use bevy::{
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin},
    pbr::wireframe::{WireframeConfig, WireframePlugin},
    prelude::*,
    render::{
        settings::{RenderCreation, WgpuFeatures, WgpuSettings},
        RenderPlugin,
    },
    text::FontSmoothing,
    winit::WinitSettings,
};

use crate::{
    buttons::{button_system, setup_buttons, update_button_visuals},
    grid::{setup_grid, toggle_grid_visibility, update_grid_system, GridSettings},
    orbit_camera::{
        orbit_camera_system, orbit_camera_zoom_system, reset_orbit_camera_system,
        toggle_orbit_mode_system, OrbitCamera, OrbitCenter,
    },
    render::{
        display_render_info_system, save_render_settings_system, toggle_lighting_system,
        toggle_render_mode_system, update_materials_system, RenderModes,
    },
};

mod buttons;
mod grid;
mod orbit_camera;
mod render;

fn main() {
    let mut app = App::new();

    app
        // Оптимизированные плагины с conditional features
        .add_plugins((
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        // Включаем поддержку wireframe для desktop платформ
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Оптимизированный САПР".into(),
                        // Отключаем VSync для лучшей производительности в дебаге
                        present_mode: bevy::window::PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 42.0,
                        font: default(),
                        font_smoothing: FontSmoothing::default(),
                        ..default()
                    },
                    text_color: Color::srgb(0.0, 1.0, 0.0),
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                },
            },
            WireframePlugin::default(),
        ))
        // Оптимизированные настройки приложения
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(OrbitCenter::default())
        .insert_resource(OrbitCamera::default())
        .insert_resource(GridSettings::default())
        .insert_resource(RenderModes::default())
        .insert_resource(WireframeConfig::default())
        // Отключаем многопоточность для Update schedule если проект простой
        // Раскомментируйте следующие строки для очень простых проектов:
        /*
        .edit_schedule(Update, |schedule| {
            schedule.set_executor_kind(ExecutorKind::SingleThreaded);
        })
        */
        // Startup системы - выполняются один раз при запуске
        .add_systems(Startup, (setup, setup_grid, setup_buttons))
        // Оптимизированная организация систем с явным порядком
        .add_systems(
            Update,
            (
                // Системы ввода - выполняются первыми
                toggle_orbit_mode_system,
                reset_orbit_camera_system,
                orbit_camera_zoom_system,
                toggle_render_mode_system,
                display_render_info_system,
                save_render_settings_system,
                // Системы обработки логики
                (button_system, orbit_camera_system, update_button_visuals).chain(), // Явный порядок выполнения
                // Системы рендеринга - выполняются последними
                (
                    update_grid_system,
                    toggle_grid_visibility,
                    update_materials_system,
                    toggle_lighting_system,
                )
                    .chain(),
            ),
        )
        .run();
}

#[derive(Component)]
pub struct Id(pub u32);

#[derive(Resource)]
pub struct CameraState {
    pub moved: bool,
}

// Оптимизированная система настройки сцены
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Создаем материал один раз и переиспользуем
    let base_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    // Circular base
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(base_material),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Name::new("GroundPlane"),
    ));

    // Оптимизированное освещение
    commands.spawn((
        PointLight {
            intensity: 10_000_000.0, // Bevy 0.16 интенсивность
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        Name::new("MainLight"),
    ));

    // Camera с оптимизированными настройками
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("MainCamera"),
    ));

    // Инициализируем состояние камеры
    commands.insert_resource(CameraState { moved: false });
}
