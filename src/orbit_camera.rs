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
    enabled: bool,
    // Кэшированные значения для оптимизации
    cached_position: Vec3,
    needs_update: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            radius: 10.0,
            azimuth: 0.0,
            elevation: 0.0,
            sensitivity: 0.01,
            enabled: false,
            cached_position: Vec3::ZERO,
            needs_update: true,
        }
    }
}

impl OrbitCamera {
    // Вычисляем позицию камеры только при необходимости
    fn calculate_position(&self, center: Vec3) -> Vec3 {
        let x = self.radius * self.elevation.cos() * self.azimuth.sin();
        let y = self.radius * self.elevation.sin();
        let z = self.radius * self.elevation.cos() * self.azimuth.cos();
        Vec3::new(x, y, z) + center
    }

    fn mark_dirty(&mut self) {
        self.needs_update = true;
    }
}

// Оптимизированная система орбитальной камеры
pub fn orbit_camera_system(
    mut orbit: ResMut<OrbitCamera>,
    orbit_center: Res<OrbitCenter>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<&mut Transform, With<Camera3d>>,
) {
    // Ранний выход если режим выключен
    if !orbit.enabled {
        // Очищаем события чтобы они не накапливались
        mouse_motion_events.clear();
        return;
    }

    // Проверяем есть ли движение мыши
    let mut delta = Vec2::ZERO;
    let mut has_movement = false;

    for event in mouse_motion_events.read() {
        delta += event.delta;
        has_movement = true;
    }

    // Обновляем углы только если есть движение
    if has_movement {
        orbit.azimuth -= delta.x * orbit.sensitivity;
        orbit.elevation += delta.y * orbit.sensitivity;

        // Ограничиваем elevation для предотвращения переворота камеры
        orbit.elevation = orbit.elevation.clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );

        orbit.mark_dirty();
    }

    // Обновляем позицию камеры только если нужно
    if orbit.needs_update || orbit_center.is_changed() {
        let new_position = orbit.calculate_position(orbit_center.position);

        // Используем single_mut для оптимизации единичной камеры
        if let Ok(mut transform) = query.single_mut() {
            transform.translation = new_position;
            *transform = transform.looking_at(orbit_center.position, Vec3::Y);
        }

        orbit.cached_position = new_position;
        orbit.needs_update = false;
    }
}

// Оптимизированная система переключения режима орбиты
pub fn toggle_orbit_mode_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut orbit: ResMut<OrbitCamera>,
) {
    // Используем just_pressed для предотвращения множественных срабатываний
    if keyboard_input.just_pressed(KeyCode::KeyC) {
        orbit.enabled = !orbit.enabled;

        // Сбрасываем кэш при переключении режима
        orbit.mark_dirty();

        // Логирование для отладки
        info!(
            "Orbit camera mode: {}",
            if orbit.enabled { "enabled" } else { "disabled" }
        );
    }
}

// Дополнительная система для сброса камеры
pub fn reset_orbit_camera_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut orbit: ResMut<OrbitCamera>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyR) {
        orbit.azimuth = 0.0;
        orbit.elevation = 0.0;
        orbit.radius = 10.0;
        orbit.mark_dirty();

        info!("Orbit camera reset to default position");
    }
}

// Система для изменения расстояния камеры с помощью колеса мыши
pub fn orbit_camera_zoom_system(
    mut scroll_events: EventReader<bevy::input::mouse::MouseWheel>,
    mut orbit: ResMut<OrbitCamera>,
) {
    if !orbit.enabled {
        scroll_events.clear();
        return;
    }

    for event in scroll_events.read() {
        // Логарифмическое масштабирование для более плавного зума
        let zoom_factor = 1.0 + event.y * 0.1;
        orbit.radius = (orbit.radius * zoom_factor).clamp(1.0, 50.0);
        orbit.mark_dirty();
    }
}
