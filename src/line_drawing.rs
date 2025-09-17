use bevy::color::palettes::css::{GREEN, YELLOW};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// Компоненты для системы рисования линий
#[derive(Component)]
pub struct DrawableLine {
    pub start: Vec3,
    pub end: Vec3,
    pub id: u32,
}

#[derive(Component)]
pub struct LineEndpoint;

// Ресурсы для состояния рисования
#[derive(Resource, Default)]
pub struct LineDrawingState {
    pub is_drawing: bool,
    pub start_point: Option<Vec3>,
    pub line_counter: u32,
    pub is_enabled: bool, // Включено ли рисование линий
}

#[derive(Resource)]
pub struct LineSettings {
    pub line_color: Color,
    pub line_thickness: f32,
    pub endpoint_size: f32,
    pub endpoint_color: Color,
}

impl Default for LineSettings {
    fn default() -> Self {
        Self {
            line_color: Color::srgb(1.0, 0.2, 0.2), // Красный цвет
            line_thickness: 0.1,
            endpoint_size: 0.2,
            endpoint_color: Color::srgb(0.2, 0.8, 0.2), // Зеленый цвет
        }
    }
}

// Оптимизированная система для обработки кликов мыши
pub fn line_drawing_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_state: ResMut<LineDrawingState>,
    line_settings: Res<LineSettings>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut gizmos: Gizmos,
) {
    // Ранний выход если система отключена
    if !line_state.is_enabled {
        return;
    }

    // Получаем камеру и окно
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };

    let Ok(window) = window_query.single() else {
        return;
    };

    // Обрабатываем клик левой кнопкой мыши
    if mouse_input.just_pressed(MouseButton::Left) {
        if let Some(cursor_position) = window.cursor_position() {
            // Преобразуем курсор в мировые координаты на плоскости Y=0
            if let Some(world_position) = screen_to_world_plane(
                cursor_position,
                camera,
                camera_transform,
                Vec3::Y, // Нормаль плоскости сетки
                0.0,     // Y-координата плоскости сетки
            ) {
                handle_line_click(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut line_state,
                    &line_settings,
                    world_position,
                );
            }
        }
    }

    // Отображаем preview линии при рисовании
    if line_state.is_drawing {
        if let Some(start) = line_state.start_point {
            if let Some(cursor_position) = window.cursor_position() {
                if let Some(current_position) =
                    screen_to_world_plane(cursor_position, camera, camera_transform, Vec3::Y, 0.0)
                {
                    // Рисуем preview линию
                    gizmos.line(start, current_position, Color::srgb(1.0, 1.0, 0.0));

                    // Показываем координаты в gizmos
                    gizmos.sphere(start, 0.1, GREEN);
                    gizmos.sphere(current_position, 0.1, YELLOW);
                }
            }
        }
    }
}

// Функция для преобразования экранных координат в мировые на плоскости
pub(crate) fn screen_to_world_plane(
    cursor_position: Vec2,
    camera: &Camera,
    camera_transform: &GlobalTransform,
    plane_normal: Vec3,
    plane_distance: f32,
) -> Option<Vec3> {
    // Создаем луч от камеры через курсор
    let ray = camera
        .viewport_to_world(camera_transform, cursor_position)
        .ok()?;

    // Вычисляем пересечение с плоскостью
    let denominator = ray.direction.dot(plane_normal);
    if denominator.abs() > 1e-6 {
        let t = (plane_distance - ray.origin.dot(plane_normal)) / denominator;
        if t >= 0.0 {
            return Some(ray.origin + ray.direction * t);
        }
    }
    None
}

// Обработка клика для создания линии
fn handle_line_click(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    line_state: &mut ResMut<LineDrawingState>,
    line_settings: &Res<LineSettings>,
    world_position: Vec3,
) {
    if !line_state.is_drawing {
        // Начинаем рисование - сохраняем стартовую точку
        line_state.start_point = Some(world_position);
        line_state.is_drawing = true;
        info!(
            "Start point: ({:.2}, {:.2}, {:.2})",
            world_position.x, world_position.y, world_position.z
        );

        // Создаем маркер начальной точки
        spawn_endpoint(commands, meshes, materials, world_position, line_settings);
    } else {
        // Завершаем рисование - создаем линию
        if let Some(start) = line_state.start_point {
            create_line(
                commands,
                meshes,
                materials,
                line_state,
                line_settings,
                start,
                world_position,
            );

            // Создаем маркер конечной точки
            spawn_endpoint(commands, meshes, materials, world_position, line_settings);

            // Сбрасываем состояние
            line_state.is_drawing = false;
            line_state.start_point = None;

            info!(
                "Line created from ({:.2}, {:.2}, {:.2}) to ({:.2}, {:.2}, {:.2})",
                start.x, start.y, start.z, world_position.x, world_position.y, world_position.z
            );
        }
    }
}

// Создание линии между двумя точками
fn create_line(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    line_state: &mut ResMut<LineDrawingState>,
    line_settings: &Res<LineSettings>,
    start: Vec3,
    end: Vec3,
) {
    let direction = end - start;
    let length = direction.length();
    let center = (start + end) / 2.0;

    // Создаем материал линии
    let line_material = materials.add(StandardMaterial {
        base_color: line_settings.line_color,
        unlit: true,
        ..default()
    });

    // Создаем меш цилиндра для линии
    let line_mesh = meshes.add(Cylinder::new(line_settings.line_thickness, length));

    // Вычисляем поворот для ориентации цилиндра
    let rotation = if direction.length() > 1e-6 {
        let normalized_direction = direction.normalize();
        let default_up = Vec3::Y;

        if (normalized_direction - default_up).length() < 1e-6 {
            Quat::IDENTITY
        } else if (normalized_direction + default_up).length() < 1e-6 {
            Quat::from_rotation_x(std::f32::consts::PI)
        } else {
            Quat::from_rotation_arc(default_up, normalized_direction)
        }
    } else {
        Quat::IDENTITY
    };

    // Спавним линию
    let line_id = line_state.line_counter;
    line_state.line_counter += 1;

    commands.spawn((
        Mesh3d(line_mesh),
        MeshMaterial3d(line_material),
        Transform {
            translation: center,
            rotation,
            scale: Vec3::ONE,
        },
        DrawableLine {
            start,
            end,
            id: line_id,
        },
        Name::new(format!("Line_{}", line_id)),
    ));
}

// Создание маркера конечной точки
fn spawn_endpoint(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    line_settings: &Res<LineSettings>,
) {
    let endpoint_material = materials.add(StandardMaterial {
        base_color: line_settings.endpoint_color,
        unlit: true,
        ..default()
    });

    let endpoint_mesh = meshes.add(Sphere::new(line_settings.endpoint_size));

    commands.spawn((
        Mesh3d(endpoint_mesh),
        MeshMaterial3d(endpoint_material),
        Transform::from_translation(position),
        LineEndpoint,
        Name::new("LineEndpoint"),
    ));
}

// Система для переключения режима рисования линий
pub fn toggle_line_drawing_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut line_state: ResMut<LineDrawingState>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        line_state.is_enabled = !line_state.is_enabled;

        // Сбрасываем состояние при отключении
        if !line_state.is_enabled {
            line_state.is_drawing = false;
            line_state.start_point = None;
        }

        info!(
            "Line drawing mode: {}",
            if line_state.is_enabled {
                "enabled"
            } else {
                "disabled"
            }
        );
    }
}

// Система для очистки всех линий
pub fn clear_lines_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    line_query: Query<Entity, Or<(With<DrawableLine>, With<LineEndpoint>)>>,
    mut line_state: ResMut<LineDrawingState>,
) {
    if keyboard_input.just_pressed(KeyCode::Delete) {
        // Удаляем все линии и точки
        for entity in line_query.iter() {
            commands.entity(entity).despawn();
        }

        // Сбрасываем состояние
        line_state.is_drawing = false;
        line_state.start_point = None;
        line_state.line_counter = 0;

        info!("All lines cleared");
    }
}

// Система для отображения информации о линиях
pub fn line_info_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    line_query: Query<&DrawableLine>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        info!("=== LINE INFORMATION ===");
        for line in line_query.iter() {
            let length = (line.end - line.start).length();
            let angle = (line.end.z - line.start.z)
                .atan2(line.end.x - line.start.x)
                .to_degrees();

            info!(
                "Line {}: Start({:.2}, {:.2}), End({:.2}, {:.2}), Length: {:.2}, Angle: {:.1}°",
                line.id, line.start.x, line.start.z, line.end.x, line.end.z, length, angle
            );
        }
        info!("Total lines: {}", line_query.iter().count());
    }
}
