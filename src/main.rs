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

// Импорты модулей (убрали buttons.rs - он устарел)
mod camera_system;
mod coordinate_systems;
mod events; // Новый централизованный модуль событий
mod grid;
mod line_drawing;
mod orbit_camera;
mod render;
mod ui_panels;

// Используем события из централизованного модуля
use events::*;

use camera_system::{
    camera_drag_pan_system, camera_scroll_zoom_system, center_camera_on_lines_system,
    cursor_coordinates_system, handle_camera_reset_events, handle_camera_toggle_events, CameraZoom,
};
use coordinate_systems::{
    handle_coordinate_system_events, keyboard_coordinate_system, CoordinateSettings,
};
use grid::{setup_grid, toggle_grid_visibility, update_grid_system, GridSettings};
use line_drawing::{
    clear_lines_system, handle_line_events, line_drawing_system, line_info_system,
    toggle_line_drawing_system, LineDrawingState, LineSettings,
};
use orbit_camera::{
    orbit_camera_system, orbit_camera_zoom_system, reset_orbit_camera_system,
    toggle_orbit_mode_system, OrbitCamera, OrbitCenter,
};
use render::{
    display_render_info_system, save_render_settings_system, toggle_lighting_system,
    toggle_render_mode_system, update_materials_system, RenderModes,
};
use ui_panels::{
    handle_ui_interactions, setup_ui_panels, track_cursor_position, update_cursor_coordinates,
    update_line_info, CursorInfo,
};

#[derive(Component)]
pub struct Id(pub u32);

#[derive(Resource)]
pub struct CameraState {
    pub moved: bool,
}

fn main() {
    let mut app = App::new();

    app
        // Настройки плагинов для Bevy 0.15+
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
                        title: "Лабораторная №1 - Построение отрезков в координатных системах"
                            .into(),
                        ..default()
                    }),
                    ..default()
                }),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 42.0,
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
        // Регистрируем все события из централизованного модуля
        .add_event::<CameraToggleEvent>()
        .add_event::<CameraZoomEvent>()
        .add_event::<CameraResetEvent>()
        .add_event::<CoordinateSystemChangeEvent>()
        .add_event::<AngleUnitChangeEvent>()
        .add_event::<CreateLineEvent>()
        .add_event::<DeleteAllLinesEvent>()
        .add_event::<LineCreatedEvent>()
        .add_event::<PointSelectedEvent>()
        .add_event::<GridStepChangeEvent>()
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
        .insert_resource(CoordinateSettings::default())
        .insert_resource(CursorInfo::default())
        // Startup системы
        .add_systems(Startup, (setup, setup_grid, setup_ui_panels))
        // Update системы - хорошая организация для Bevy 0.15+
        .add_systems(
            Update,
            (
                // === ВВОД И ГЕНЕРАЦИЯ СОБЫТИЙ ===
                toggle_orbit_mode_system,
                reset_orbit_camera_system,
                orbit_camera_zoom_system,
                toggle_render_mode_system,
                display_render_info_system,
                save_render_settings_system,
                toggle_line_drawing_system,
                clear_lines_system,
                line_info_system,
                keyboard_coordinate_system,
                handle_ui_interactions,
            ),
        )
        .add_systems(
            Update,
            (
                // === ОБРАБОТЧИКИ СОБЫТИЙ ===
                handle_camera_toggle_events,
                handle_camera_reset_events,
                handle_coordinate_system_events,
                handle_line_events,
                grid::handle_grid_step_events,
            ),
        )
        .add_systems(
            Update,
            (
                // === ОСНОВНАЯ ЛОГИКА ===
                orbit_camera_system,
                camera_drag_pan_system,
                camera_scroll_zoom_system,
                cursor_coordinates_system,
                center_camera_on_lines_system,
                track_cursor_position,
                line_drawing_system,
            ),
        )
        .add_systems(
            Update,
            (
                // === РЕНДЕРИНГ И UI ===
                update_grid_system,
                toggle_grid_visibility,
                update_materials_system,
                toggle_lighting_system,
                update_cursor_coordinates,
                update_line_info,
            ),
        )
        .run();
}

// Setup функция для Bevy 0.15+ с новыми Required Components
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Создаем материал для плоскости
    let base_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        ..default()
    });

    // Плоскость - используем новый синтаксис Bevy 0.15+
    commands.spawn((
        // Новый синтаксис: Mesh3d вместо PbrBundle
        Mesh3d(meshes.add(Circle::new(4.0))),
        MeshMaterial3d(base_material),
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        Name::new("GroundPlane"),
        // Transform и Visibility добавляются автоматически через Required Components
    ));

    // Освещение - новый синтаксис Bevy 0.15+
    commands.spawn((
        // Раньше: PointLightBundle, теперь: PointLight + компоненты
        PointLight {
            intensity: 10_000_000.0,
            shadows_enabled: true,
            shadow_depth_bias: 0.02,
            shadow_normal_bias: 0.6,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0),
        Name::new("MainLight"),
        // Visibility добавляется автоматически
    ));

    // Камера - новый синтаксис Bevy 0.15+
    commands.spawn((
        // Раньше: Camera3dBundle, теперь: Camera3d + компоненты
        Camera3d::default(),
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        Name::new("MainCamera"),
        // Visibility добавляется автоматически
    ));

    commands.insert_resource(CameraState { moved: false });
}
