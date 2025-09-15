use bevy::{
    color::palettes::css::{BLUE, RED},
    prelude::*,
};

use crate::render::RenderModes;

#[derive(Component)]
pub struct GridLine;

#[derive(Resource)]
pub struct GridSettings {
    size: f32,       // размер сетки (сколько единиц в каждую сторону)
    step: f32,       // шаг сетки
    line_width: f32, // толщина линий
    color: Color,    // цвет линий сетки
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

pub fn setup_grid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid_settings: Res<GridSettings>,
    render_modes: Res<RenderModes>,
) {
    if !render_modes.grid_visible {
        return;
    }

    let material = materials.add(StandardMaterial {
        base_color: grid_settings.color,
        unlit: true,
        ..default()
    });

    // Создаем вертикальные линии (параллельные оси Y)
    for i in 0..=(2.0 * grid_settings.size / grid_settings.step) as i32 {
        let x = -grid_settings.size + i as f32 * grid_settings.step;

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                grid_settings.line_width,
                0.01,
                2.0 * grid_settings.size,
            ))),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(x, 0.0, 0.0),
            GridLine,
        ));
    }

    // Создаем горизонтальные линии (параллельные оси X)
    for i in 0..=(2.0 * grid_settings.size / grid_settings.step) as i32 {
        let z = -grid_settings.size + i as f32 * grid_settings.step;

        commands.spawn((
            Mesh3d(meshes.add(Cuboid::new(
                2.0 * grid_settings.size,
                0.01,
                grid_settings.line_width,
            ))),
            MeshMaterial3d(material.clone()),
            Transform::from_xyz(0.0, 0.0, z),
            GridLine,
        ));
    }

    // Создаем оси координат (более толстые и яркие)
    let axis_material = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..default()
    });

    // Ось X (красная)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            2.0 * grid_settings.size,
            0.02,
            grid_settings.line_width * 2.0,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: RED.into(),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.01, 0.0),
        GridLine,
    ));

    // Ось Z (синяя, представляет Y в 2D)
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(
            grid_settings.line_width * 2.0,
            0.02,
            2.0 * grid_settings.size,
        ))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: BLUE.into(),
            unlit: true,
            ..default()
        })),
        Transform::from_xyz(0.0, 0.01, 0.0),
        GridLine,
    ));
}

// Система для обновления сетки при изменении настроек
pub fn update_grid_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    grid_settings: Res<GridSettings>,
    render_modes: Res<RenderModes>,
    grid_query: Query<Entity, With<GridLine>>,
) {
    if grid_settings.is_changed() || render_modes.is_changed() {
        // Удаляем старую сетку
        for entity in grid_query.iter() {
            commands.entity(entity).despawn();
        }

        // Создаем новую сетку только если она должна быть видна
        if render_modes.grid_visible {
            setup_grid(commands, meshes, materials, grid_settings, render_modes);
        }
    }
}
