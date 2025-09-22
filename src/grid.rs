use crate::{events::GridStepChangeEvent, render::RenderModes};
use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
};

#[derive(Component)]
pub struct GridLine;

#[derive(Component)]
pub struct GridAxis;

#[derive(Resource)]
pub struct GridSettings {
    pub size: f32,       // размер сетки (сколько единиц в каждую сторону)
    pub step: f32,       // шаг сетки
    pub line_width: f32, // толщина линий
    pub color: Color,    // цвет линий сетки
}

impl Default for GridSettings {
    fn default() -> Self {
        Self {
            size: 10.0,
            step: 1.0,
            line_width: 0.05,
            color: Color::srgb(0.3, 0.3, 0.3),
        }
    }
}

// Кэшированные материалы сетки для переиспользования
#[derive(Resource)]
pub struct GridMaterials {
    pub grid_material: Handle<StandardMaterial>,
    pub axis_x_material: Handle<StandardMaterial>,
    pub axis_z_material: Handle<StandardMaterial>,
}

impl GridMaterials {
    fn new(materials: &mut ResMut<Assets<StandardMaterial>>, grid_color: Color) -> Self {
        Self {
            grid_material: materials.add(StandardMaterial {
                base_color: grid_color,
                unlit: true,
                ..default()
            }),
            axis_x_material: materials.add(StandardMaterial {
                base_color: RED.into(),
                unlit: true,
                ..default()
            }),
            axis_z_material: materials.add(StandardMaterial {
                base_color: BLUE.into(),
                unlit: true,
                ..default()
            }),
        }
    }
}

// Оптимизированная система настройки сетки для Bevy 0.15+
pub fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid_settings: Res<GridSettings>,
    render_modes: Res<RenderModes>,
) {
    // Ранний выход если сетка не должна быть видна
    if !render_modes.grid_visible {
        return;
    }

    // Создаем кэшированные материалы
    let grid_materials = GridMaterials::new(&mut materials, grid_settings.color);

    // Предварительно создаем мешы для переиспользования
    let vertical_mesh = meshes.add(Cuboid::new(
        grid_settings.line_width,
        0.01,
        2.0 * grid_settings.size,
    ));
    let horizontal_mesh = meshes.add(Cuboid::new(
        2.0 * grid_settings.size,
        0.01,
        grid_settings.line_width,
    ));
    let axis_x_mesh = meshes.add(Cuboid::new(
        2.0 * grid_settings.size,
        0.02,
        grid_settings.line_width * 2.0,
    ));
    let axis_z_mesh = meshes.add(Cuboid::new(
        grid_settings.line_width * 2.0,
        0.02,
        2.0 * grid_settings.size,
    ));

    let grid_count = (2.0 * grid_settings.size / grid_settings.step) as i32 + 1;

    // Создаем все вертикальные линии сразу - Bevy 0.15+ синтаксис
    for i in 0..=grid_count {
        let x = -grid_settings.size + i as f32 * grid_settings.step;
        commands.spawn((
            // Bevy 0.15+: Используем Mesh3d и MeshMaterial3d
            Mesh3d(vertical_mesh.clone()),
            MeshMaterial3d(grid_materials.grid_material.clone()),
            Transform::from_xyz(x, 0.0, 0.0),
            GridLine,
            Name::new(format!("GridLine_Vertical_{}", i)),
            // Transform и Visibility добавляются автоматически
        ));
    }

    // Создаем все горизонтальные линии сразу - Bevy 0.15+ синтаксис
    for i in 0..=grid_count {
        let z = -grid_settings.size + i as f32 * grid_settings.step;
        commands.spawn((
            // Bevy 0.15+: Используем Mesh3d и MeshMaterial3d
            Mesh3d(horizontal_mesh.clone()),
            MeshMaterial3d(grid_materials.grid_material.clone()),
            Transform::from_xyz(0.0, 0.0, z),
            GridLine,
            Name::new(format!("GridLine_Horizontal_{}", i)),
            // Transform и Visibility добавляются автоматически
        ));
    }

    // Создаем оси координат - Bevy 0.15+ синтаксис
    commands.spawn((
        // Bevy 0.15+: Mesh3d и MeshMaterial3d
        Mesh3d(axis_x_mesh),
        MeshMaterial3d(grid_materials.axis_x_material.clone()),
        Transform::from_xyz(0.0, 0.01, 0.0),
        GridAxis,
        Name::new("GridAxis_X"),
        // Transform и Visibility добавляются автоматически
    ));

    commands.spawn((
        // Bevy 0.15+: Mesh3d и MeshMaterial3d
        Mesh3d(axis_z_mesh),
        MeshMaterial3d(grid_materials.axis_z_material.clone()),
        Transform::from_xyz(0.0, 0.01, 0.0),
        GridAxis,
        Name::new("GridAxis_Z"),
        // Transform и Visibility добавляются автоматически
    ));

    // Сохраняем материалы как ресурс для переиспользования
    commands.insert_resource(grid_materials);
}

// Оптимизированная система обновления сетки с Change Detection
pub fn update_grid_system(
    mut commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    grid_settings: Res<GridSettings>,
    render_modes: Res<RenderModes>,
    grid_query: Query<Entity, Or<(With<GridLine>, With<GridAxis>)>>,
    grid_materials: Option<Res<GridMaterials>>,
) {
    // Проверяем изменения только необходимых ресурсов
    let settings_changed = grid_settings.is_changed();
    let render_changed = render_modes.is_changed();

    if !settings_changed && !render_changed {
        return; // Ранний выход если ничего не изменилось
    }

    // Удаляем старую сетку только при необходимости
    if settings_changed || (render_changed && !render_modes.grid_visible) {
        // Используем batch операции для лучшей производительности
        let entities_to_despawn: Vec<Entity> = grid_query.iter().collect();
        for entity in entities_to_despawn {
            commands.entity(entity).despawn();
        }

        // Удаляем кэшированные материалы если настройки изменились
        if settings_changed && grid_materials.is_some() {
            commands.remove_resource::<GridMaterials>();
        }
    }

    // Создаем новую сетку только если она должна быть видна
    if render_modes.grid_visible {
        setup_grid(commands, meshes, materials, grid_settings, render_modes);
    }
}

// Система для переключения видимости сетки без пересоздания
pub fn toggle_grid_visibility(
    mut grid_query: Query<&mut Visibility, Or<(With<GridLine>, With<GridAxis>)>>,
    render_modes: Res<RenderModes>,
) {
    // Используем Change Detection для оптимизации
    if !render_modes.is_changed() {
        return;
    }

    let visibility = if render_modes.grid_visible {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    // Используем par_iter_mut для лучшей производительности
    grid_query.par_iter_mut().for_each(|mut vis| {
        *vis = visibility;
    });
}

/// Система обработки событий изменения шага сетки (перенесена из main.rs)
pub fn handle_grid_step_events(
    mut grid_events: EventReader<GridStepChangeEvent>,
    mut grid_settings: ResMut<GridSettings>,
) {
    for event in grid_events.read() {
        grid_settings.step = event.new_step;
        info!("Grid step changed to: {}", event.new_step);
    }
}
