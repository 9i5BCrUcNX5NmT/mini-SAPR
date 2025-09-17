use bevy::prelude::*;

// Определяем события для различных действий камеры
#[derive(Event)]
pub struct CameraToggleEvent;

#[derive(Event)]
pub struct CameraZoomEvent {
    pub zoom_delta: f32,
}

#[derive(Event)]
pub struct CameraResetEvent;

// Ресурс для управления масштабированием камеры
#[derive(Resource)]
pub struct CameraZoom {
    pub zoom_level: f32,
    pub min_zoom: f32,
    pub max_zoom: f32,
    pub zoom_speed: f32,
    pub is_top_view: bool,
}

impl Default for CameraZoom {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            min_zoom: 0.1,
            max_zoom: 10.0,
            zoom_speed: 0.1,
            is_top_view: false,
        }
    }
}

// Система для обработки сброса камеры
pub fn handle_camera_reset_events(
    mut reset_events: EventReader<CameraResetEvent>,
    mut camera_zoom: ResMut<crate::camera_system::CameraZoom>,
    mut query_camera: Query<&mut Transform, With<Camera3d>>,
) {
    for _event in reset_events.read() {
        camera_zoom.zoom_level = 1.0;

        if let Ok(mut transform) = query_camera.single_mut() {
            crate::camera_system::apply_zoom_to_camera(&mut transform, &camera_zoom);
            info!("Camera reset to default zoom");
        }
    }
}

// Функция применения масштаба к камере (делаем публичной)
pub fn apply_zoom_to_camera(transform: &mut Transform, camera_zoom: &CameraZoom) {
    if camera_zoom.is_top_view {
        // В режиме вида сверху изменяем высоту камеры
        let base_height = 18.0;
        let new_height = base_height * camera_zoom.zoom_level;
        transform.translation.y = new_height;
    } else {
        // В обычном режиме изменяем расстояние от центра
        let direction = transform.translation.normalize();
        let base_distance = 10.0;
        let new_distance = base_distance * camera_zoom.zoom_level;
        transform.translation = direction * new_distance;
    }
}

// Система для обработки событий переключения камеры
pub fn handle_camera_toggle_events(
    mut toggle_events: EventReader<CameraToggleEvent>,
    mut query_camera: Query<&mut Transform, With<Camera3d>>,
    mut state_camera: ResMut<crate::CameraState>,
    mut camera_zoom: ResMut<crate::camera_system::CameraZoom>,
) {
    // Обрабатываем все события переключения камеры
    for _event in toggle_events.read() {
        if let Ok(mut transform) = query_camera.single_mut() {
            // Логика переключения камеры
            if !state_camera.moved {
                *transform = Transform::from_xyz(0.0, 18.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y);
                camera_zoom.is_top_view = true;
                info!("Switched to top view");
            } else {
                *transform = Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y);
                camera_zoom.is_top_view = false;
                info!("Switched to perspective view");
            }

            // Применяем текущий масштаб
            crate::camera_system::apply_zoom_to_camera(&mut transform, &camera_zoom);

            state_camera.moved = !state_camera.moved;
        }
    }
}

pub fn camera_drag_pan_system(
    mut mouse_motion_events: EventReader<bevy::input::mouse::MouseMotion>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    camera_zoom: Res<CameraZoom>,
) {
    // Работает только в виде сверху
    if !camera_zoom.is_top_view {
        return;
    }
    // Проверяем что зажата правая кнопка мыши
    if !mouse_buttons.pressed(MouseButton::Right) {
        return;
    }
    if let Ok(mut camera_transform) = camera_query.single_mut() {
        let mut total_delta = Vec2::ZERO;
        for event in mouse_motion_events.read() {
            total_delta += event.delta;
        }
        if total_delta.length_squared() > 0.0 {
            // Коэффициент скорости с учётом масштаба (подберите под UX)
            let pan_speed = 0.02 * camera_zoom.zoom_level;
            camera_transform.translation.x -= total_delta.y * pan_speed;
            camera_transform.translation.z += total_delta.x * pan_speed;
        }
    }
}

pub fn camera_scroll_zoom_system(
    mut mouse_wheel_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut camera_zoom: ResMut<CameraZoom>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
) {
    // Только в режиме вида сверху
    if !camera_zoom.is_top_view {
        return;
    }
    let mut zoom_delta = 0.0;
    for event in mouse_wheel_events.read() {
        zoom_delta += event.y;
    }
    if zoom_delta.abs() > 0.0 {
        let zoom_factor = 1.0 - zoom_delta * 0.08; // Подберите коэффициент для UX
        camera_zoom.zoom_level = (camera_zoom.zoom_level * zoom_factor)
            .clamp(camera_zoom.min_zoom, camera_zoom.max_zoom);
        if let Ok(mut transform) = camera_query.single_mut() {
            apply_zoom_to_camera(&mut transform, &camera_zoom);
            info!("Camera zoom changed to: {:.2}", camera_zoom.zoom_level);
        }
    }
}

// Система для отображения координат курсора в режиме вида сверху
pub fn cursor_coordinates_system(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<bevy::window::PrimaryWindow>>,
    camera_zoom: Res<CameraZoom>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    // Показываем координаты только в режиме вида сверху при нажатии Tab
    if !camera_zoom.is_top_view || !keyboard_input.pressed(KeyCode::Tab) {
        return;
    }

    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = window_query.single() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        if let Some(world_position) = crate::line_drawing::screen_to_world_plane(
            cursor_position,
            camera,
            camera_transform,
            Vec3::Y,
            0.0,
        ) {
            // В реальном приложении здесь можно отображать в UI
            // Пока выводим в консоль при удерживании Tab
            if keyboard_input.just_pressed(KeyCode::Tab) {
                info!(
                    "Cursor position: X: {:.2}, Z: {:.2}",
                    world_position.x, world_position.z
                );
            }
        }
    }
}

// Система для автоматического центрирования камеры на объектах
// ?
pub fn center_camera_on_lines_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    line_query: Query<&crate::line_drawing::DrawableLine>,
    mut camera_query: Query<&mut Transform, With<Camera3d>>,
    camera_zoom: Res<CameraZoom>,
) {
    if !keyboard_input.just_pressed(KeyCode::KeyF) || !camera_zoom.is_top_view {
        return;
    }

    let lines: Vec<_> = line_query.iter().collect();
    if lines.is_empty() {
        return;
    }

    // Вычисляем центр всех линий
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_z = f32::INFINITY;
    let mut max_z = f32::NEG_INFINITY;

    for line in &lines {
        min_x = min_x.min(line.start.x).min(line.end.x);
        max_x = max_x.max(line.start.x).max(line.end.x);
        min_z = min_z.min(line.start.z).min(line.end.z);
        max_z = max_z.max(line.start.z).max(line.end.z);
    }

    let center_x = (min_x + max_x) / 2.0;
    let center_z = (min_z + max_z) / 2.0;

    if let Ok(mut camera_transform) = camera_query.single_mut() {
        camera_transform.translation.x = center_x;
        camera_transform.translation.z = center_z;
        info!(
            "Camera centered on lines at ({:.2}, {:.2})",
            center_x, center_z
        );
    }
}
