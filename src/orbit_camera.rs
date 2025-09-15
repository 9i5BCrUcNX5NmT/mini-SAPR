use bevy::{input::mouse::MouseMotion, prelude::*};

#[derive(Resource)]
pub struct OrbitCenter {
    pub position: Vec3,
}

impl Default for OrbitCenter {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
        }
    }
}

#[derive(Resource)]
pub struct OrbitCamera {
    radius: f32,
    azimuth: f32,
    elevation: f32,
    sensitivity: f32,
    enabled: bool, // флаг режима вращения
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            radius: 10.0,
            azimuth: 0.0,
            elevation: 0.0,
            sensitivity: 0.01,
            enabled: false, // режим выключен по умолчанию
        }
    }
}

pub fn orbit_camera_system(
    mut orbit: ResMut<OrbitCamera>,
    orbit_center: Res<OrbitCenter>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Camera>>,
) {
    if !orbit.enabled {
        return; // Если режим выключен — не обрабатываем движение мыши
    }

    let mut delta = Vec2::ZERO;
    for event in mouse_motion_events.read() {
        delta += event.delta;
    }

    if delta.length_squared() > 0.0 {
        orbit.azimuth -= delta.x * orbit.sensitivity;
        orbit.elevation += delta.y * orbit.sensitivity;

        orbit.elevation = orbit.elevation.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        ); // ?
    }

    let x = orbit.radius * orbit.elevation.cos() * orbit.azimuth.sin();
    let y = orbit.radius * orbit.elevation.sin();
    let z = orbit.radius * orbit.elevation.cos() * orbit.azimuth.cos();

    let center = orbit_center.position;

    for mut transform in &mut query {
        transform.translation = Vec3::new(x, y, z) + center;
        *transform = transform.looking_at(center, Vec3::Y);
    }
}

pub fn toggle_orbit_mode_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut orbit: ResMut<OrbitCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        orbit.enabled = !orbit.enabled;
    }
}
