use bevy::color::palettes::css::{GREEN, YELLOW};
use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::{
    coordinate_systems::{
        conversions, formatting, CoordinatePoint, CoordinateSettings, CoordinateSystem,
    },
    events::*, // Используем централизованные события
};

// Компоненты для системы рисования линий
#[derive(Component)]
pub struct DrawableLine {
    pub start: Vec3,
    pub end: Vec3,
    pub id: u32,
    // Добавляем информацию о координатах
    pub start_coord: CoordinatePoint,
    pub end_coord: CoordinatePoint,
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
    // Добавляем поддержку полярного режима ввода
    pub polar_input_mode: bool, // Ввод второй точки в полярных координатах
    pub polar_start_point: Option<CoordinatePoint>,
}

#[derive(Resource)]
pub struct LineSettings {
    pub line_color: Color,
    pub line_thickness: f32,
    pub endpoint_size: f32,
    pub endpoint_color: Color,
    // Добавляем настройки для полярного режима
    pub polar_preview_color: Color,
}

impl Default for LineSettings {
    fn default() -> Self {
        Self {
            line_color: Color::srgb(1.0, 0.2, 0.2), // Красный цвет
            line_thickness: 0.1,
            endpoint_size: 0.2,
            endpoint_color: Color::srgb(0.2, 0.8, 0.2), // Зеленый цвет
            polar_preview_color: Color::srgb(0.2, 0.2, 1.0), // Синий для полярного режима
        }
    }
}

/// Обновленная система для обработки кликов мыши с поддержкой полярных координат
pub fn line_drawing_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_state: ResMut<LineDrawingState>,
    line_settings: Res<LineSettings>,
    coordinate_settings: Res<CoordinateSettings>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window, With<PrimaryWindow>>,
    mut gizmos: Gizmos,
    mut line_events: EventWriter<LineCreatedEvent>,
    mut point_events: EventWriter<PointSelectedEvent>,
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
                handle_line_click_enhanced(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    &mut line_state,
                    &line_settings,
                    &coordinate_settings,
                    world_position,
                    &mut line_events,
                    &mut point_events,
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
                    draw_line_preview(
                        &mut gizmos,
                        start,
                        current_position,
                        &coordinate_settings,
                        &line_settings,
                        &line_state,
                    );
                }
            }
        }
    }
}

/// Отрисовка preview линии
fn draw_line_preview(
    gizmos: &mut Gizmos,
    start: Vec3,
    current: Vec3,
    coordinate_settings: &CoordinateSettings,
    line_settings: &LineSettings,
    line_state: &LineDrawingState,
) {
    // Выбираем цвет в зависимости от режима
    let preview_color = match coordinate_settings.coordinate_system {
        CoordinateSystem::Cartesian => Color::srgb(1.0, 1.0, 0.0), // Желтый
        CoordinateSystem::Polar => line_settings.polar_preview_color, // Синий
    };

    // Рисуем preview линию
    gizmos.line(start, current, preview_color);

    // Показываем координаты в gizmos
    gizmos.sphere(start, 0.1, GREEN);
    gizmos.sphere(current, 0.1, YELLOW);

    // В полярном режиме показываем дополнительную информацию
    if coordinate_settings.coordinate_system == CoordinateSystem::Polar {
        if let Some(_start_coord) = &line_state.polar_start_point {
            let current_coord = CoordinatePoint::from_world(current);

            // Рисуем радиус-вектор
            gizmos.line(Vec3::ZERO, current, Color::srgb(0.5, 0.5, 1.0));

            // Можно добавить дугу для показа угла (опционально)
            draw_angle_arc(
                gizmos,
                current_coord.polar.theta,
                current_coord.polar.r.min(2.0),
            );
        }
    }
}

/// Улучшенная обработка клика для создания линии с поддержкой координатных систем
fn handle_line_click_enhanced(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    line_state: &mut ResMut<LineDrawingState>,
    line_settings: &Res<LineSettings>,
    coordinate_settings: &Res<CoordinateSettings>,
    world_position: Vec3,
    line_events: &mut EventWriter<LineCreatedEvent>,
    point_events: &mut EventWriter<PointSelectedEvent>,
) {
    if !line_state.is_drawing {
        // Начинаем рисование - сохраняем стартовую точку
        line_state.start_point = Some(world_position);
        line_state.is_drawing = true;

        // Сохраняем координатную информацию
        let coord_point = CoordinatePoint::from_world(world_position);
        line_state.polar_start_point = Some(coord_point.clone());

        // Выводим информацию в зависимости от системы координат
        match coordinate_settings.coordinate_system {
            CoordinateSystem::Cartesian => {
                info!(
                    "Start point (Cartesian): {}",
                    formatting::format_cartesian(coord_point.cartesian)
                );
            }
            CoordinateSystem::Polar => {
                info!(
                    "Start point (Polar): {}",
                    formatting::format_polar(coord_point.polar, coordinate_settings.angle_unit)
                );
            }
        }

        // Создаем маркер начальной точки
        spawn_endpoint(commands, meshes, materials, world_position, line_settings);

        // Отправляем событие выбора точки
        point_events.write(PointSelectedEvent {
            point: world_position,
            is_start: true,
        });
    } else {
        // Завершаем рисование - создаем линию
        if let Some(start) = line_state.start_point {
            if let Some(start_coord) = &line_state.polar_start_point {
                let end_coord = CoordinatePoint::from_world(world_position);

                // Выводим информацию о конечной точке
                match coordinate_settings.coordinate_system {
                    CoordinateSystem::Cartesian => {
                        info!(
                            "End point (Cartesian): {}",
                            formatting::format_cartesian(end_coord.cartesian)
                        );
                    }
                    CoordinateSystem::Polar => {
                        info!(
                            "End point (Polar): {}",
                            formatting::format_polar(
                                end_coord.polar,
                                coordinate_settings.angle_unit
                            )
                        );
                    }
                }

                // Создаем линию с расширенной информацией
                create_line_enhanced(
                    commands,
                    meshes,
                    materials,
                    line_state,
                    line_settings,
                    start,
                    world_position,
                    start_coord.clone(),
                    end_coord,
                    line_events,
                );

                // Создаем маркер конечной точки
                spawn_endpoint(commands, meshes, materials, world_position, line_settings);

                // Выводим информацию о созданной линии
                info!(
                    "Line created: Length = {}, Angle = {}",
                    formatting::format_line_length(start, world_position),
                    formatting::format_line_angle(
                        start,
                        world_position,
                        coordinate_settings.angle_unit
                    )
                );

                // Отправляем событие выбора конечной точки
                point_events.write(PointSelectedEvent {
                    point: world_position,
                    is_start: false,
                });
            }
        }

        // Сбрасываем состояние
        line_state.is_drawing = false;
        line_state.start_point = None;
        line_state.polar_start_point = None;
    }
}

/// Создание линии с расширенной информацией о координатах - Bevy 0.15+ синтаксис
fn create_line_enhanced(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    line_state: &mut ResMut<LineDrawingState>,
    line_settings: &Res<LineSettings>,
    start: Vec3,
    end: Vec3,
    start_coord: CoordinatePoint,
    end_coord: CoordinatePoint,
    line_events: &mut EventWriter<LineCreatedEvent>,
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

    // Спавним линию с расширенной информацией - новый Bevy 0.15+ синтаксис
    let line_id = line_state.line_counter;
    line_state.line_counter += 1;

    commands.spawn((
        // Bevy 0.15+: Используем Mesh3d и MeshMaterial3d
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
            start_coord,
            end_coord,
        },
        Name::new(format!("Line_{}", line_id)),
        // Transform и Visibility добавляются автоматически через Required Components
    ));

    // Отправляем событие создания линии
    line_events.write(LineCreatedEvent {
        line_id,
        start,
        end,
    });
}

/// Вспомогательная функция для рисования дуги угла в полярном режиме
fn draw_angle_arc(gizmos: &mut Gizmos, angle: f32, radius: f32) {
    let segments = 16;
    let step = angle / segments as f32;
    for i in 0..segments {
        let angle1 = i as f32 * step;
        let angle2 = (i + 1) as f32 * step;
        let point1 = Vec3::new(radius * angle1.cos(), 0.0, radius * angle1.sin());
        let point2 = Vec3::new(radius * angle2.cos(), 0.0, radius * angle2.sin());
        gizmos.line(point1, point2, Color::srgb(0.7, 0.7, 1.0));
    }
}

// Функция для преобразования экранных координат в мировые на плоскости
pub fn screen_to_world_plane(
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

/// Создание маркера конечной точки - Bevy 0.15+ синтаксис
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
        // Bevy 0.15+: Используем Mesh3d и MeshMaterial3d
        Mesh3d(endpoint_mesh),
        MeshMaterial3d(endpoint_material),
        Transform::from_translation(position),
        LineEndpoint,
        Name::new("LineEndpoint"),
        // Transform и Visibility добавляются автоматически
    ));
}

/// Система для переключения режима рисования линий (обновленная)
pub fn toggle_line_drawing_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut line_state: ResMut<LineDrawingState>,
    coordinate_settings: Res<CoordinateSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyL) {
        line_state.is_enabled = !line_state.is_enabled;

        // Сбрасываем состояние при отключении
        if !line_state.is_enabled {
            line_state.is_drawing = false;
            line_state.start_point = None;
            line_state.polar_start_point = None;
        }

        info!(
            "Line drawing mode: {} (System: {:?})",
            if line_state.is_enabled {
                "enabled"
            } else {
                "disabled"
            },
            coordinate_settings.coordinate_system
        );
    }
}

/// Система для очистки всех линий (обновленная)
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
        line_state.polar_start_point = None;
        line_state.line_counter = 0;
        info!("All lines cleared");
    }
}

/// Система для отображения информации о линиях (обновленная)
pub fn line_info_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    line_query: Query<&DrawableLine>,
    coordinate_settings: Res<CoordinateSettings>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyI) {
        info!("=== LINE INFORMATION ===");
        info!(
            "Current coordinate system: {:?}",
            coordinate_settings.coordinate_system
        );
        info!("Current angle unit: {:?}", coordinate_settings.angle_unit);

        for line in line_query.iter() {
            let length = (line.end - line.start).length();
            match coordinate_settings.coordinate_system {
                CoordinateSystem::Cartesian => {
                    info!(
                        "Line {}: Start {}, End {}, Length: {:.2}",
                        line.id,
                        formatting::format_cartesian(line.start_coord.cartesian),
                        formatting::format_cartesian(line.end_coord.cartesian),
                        length
                    );
                }
                CoordinateSystem::Polar => {
                    info!(
                        "Line {}: Start {}, End {}, Length: {:.2}",
                        line.id,
                        formatting::format_polar(
                            line.start_coord.polar,
                            coordinate_settings.angle_unit
                        ),
                        formatting::format_polar(
                            line.end_coord.polar,
                            coordinate_settings.angle_unit
                        ),
                        length
                    );
                }
            }
        }
        info!("Total lines: {}", line_query.iter().count());
    }
}

/// Система обработки событий создания и удаления линий (перенесена из main.rs)
pub fn handle_line_events(
    mut create_events: EventReader<CreateLineEvent>,
    mut delete_events: EventReader<DeleteAllLinesEvent>,
    mut line_state: ResMut<LineDrawingState>,
    mut commands: Commands,
    line_query: Query<Entity, Or<(With<DrawableLine>, With<LineEndpoint>)>>,
) {
    // Обработка событий создания линии
    for _event in create_events.read() {
        line_state.is_enabled = true;
        info!("Line drawing mode enabled");
    }

    // Обработка событий удаления всех линий
    for _event in delete_events.read() {
        // Удаляем все линии и точки
        for entity in line_query.iter() {
            commands.entity(entity).despawn();
        }

        // Сбрасываем состояние
        line_state.is_drawing = false;
        line_state.start_point = None;
        line_state.line_counter = 0;
        info!("All lines cleared via UI");
    }
}
