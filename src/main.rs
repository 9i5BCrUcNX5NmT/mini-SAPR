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
    camera_system::{
        camera_drag_pan_system, camera_scroll_zoom_system, center_camera_on_lines_system,
        cursor_coordinates_system, handle_camera_reset_events, handle_camera_toggle_events,
        CameraResetEvent, CameraToggleEvent, CameraZoom, CameraZoomEvent,
    },
    grid::{setup_grid, toggle_grid_visibility, update_grid_system, GridSettings},
    line_drawing::{
        clear_lines_system, line_drawing_system, line_info_system, toggle_line_drawing_system,
        LineDrawingState, LineSettings,
    },
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
mod camera_system;
mod grid;
mod line_drawing;
mod orbit_camera;
mod render;

fn main() {
    let mut app = App::new();

    app
        // Оптимизированные плагины
        .add_plugins((
            DefaultPlugins
                .set(RenderPlugin {
                    render_creation: RenderCreation::Automatic(WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }),
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Событийный САПР - Построение отрезков".into(),
                        present_mode: bevy::window::PresentMode::default(),
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
        // ВАЖНО: Регистрируем события
        .add_event::<CameraToggleEvent>()
        .add_event::<CameraZoomEvent>()
        .add_event::<CameraResetEvent>()
        // Ресурсы
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(OrbitCenter::default())
        .insert_resource(OrbitCamera::default())
        .insert_resource(GridSettings::default())
        .insert_resource(RenderModes::default())
        .insert_resource(WireframeConfig::default())
        .insert_resource(LineDrawingState::default())
        .insert_resource(LineSettings::default())
        .insert_resource(CameraZoom::default())
        // Startup системы
        .add_systems(Startup, (setup, setup_grid, setup_buttons))
        // Update системы с событийной архитектурой
        .add_systems(
            Update,
            (
                // === СИСТЕМЫ ВВОДА И ОТПРАВКИ СОБЫТИЙ ===
                (
                    // Клавиатурные системы, которые отправляют события
                    toggle_orbit_mode_system,
                    reset_orbit_camera_system,
                    orbit_camera_zoom_system,
                    toggle_render_mode_system,
                    display_render_info_system,
                    save_render_settings_system,
                    toggle_line_drawing_system,
                    clear_lines_system,
                    line_info_system,
                )
                    .chain(),
                // === СИСТЕМЫ UI И ВЗАИМОДЕЙСТВИЯ ===
                (
                    // UI системы, которые отправляют события
                    button_system, // Теперь отправляет события вместо прямых вызовов
                    update_button_visuals,
                    line_drawing_system,
                )
                    .chain(),
                // === ОБРАБОТЧИКИ СОБЫТИЙ ===
                (
                    // Системы, которые слушают и обрабатывают события
                    handle_camera_toggle_events, // Обрабатывает CameraToggleEvent
                    handle_camera_reset_events,  // Обрабатывает CameraResetEvent
                )
                    .chain(),
                // === ОБЫЧНЫЕ СИСТЕМЫ ЛОГИКИ ===
                (
                    orbit_camera_system,
                    camera_drag_pan_system,
                    camera_scroll_zoom_system,
                    cursor_coordinates_system,
                    center_camera_on_lines_system,
                )
                    .chain(),
                // === СИСТЕМЫ РЕНДЕРИНГА ===
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

// Setup функция (без изменений)
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let base_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    commands.spawn((
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(base_material),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Name::new("GroundPlane"),
    ));

    commands.spawn((
        PointLight {
            intensity: 10_000_000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        Name::new("MainLight"),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("MainCamera"),
    ));

    commands.insert_resource(CameraState { moved: false });
}
