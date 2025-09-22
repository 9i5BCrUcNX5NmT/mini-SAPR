use bevy::prelude::*;
use std::f32::consts::PI;

/// Режим системы координат
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum CoordinateSystem {
    #[default]
    Cartesian,
    Polar,
}

/// Единицы измерения углов
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum AngleUnit {
    #[default]
    Degrees,
    Radians,
}

/// Настройки системы координат
#[derive(Resource, Clone)]
pub struct CoordinateSettings {
    pub coordinate_system: CoordinateSystem,
    pub angle_unit: AngleUnit,
}

impl Default for CoordinateSettings {
    fn default() -> Self {
        Self {
            coordinate_system: CoordinateSystem::Cartesian,
            angle_unit: AngleUnit::Degrees,
        }
    }
}

/// Структура для представления точки в полярных координатах
#[derive(Clone, Copy, Debug)]
pub struct PolarPoint {
    pub r: f32, // радиус (расстояние от начала координат)
    pub theta: f32, // угол в радианах
}

impl PolarPoint {
    pub fn new(r: f32, theta: f32) -> Self {
        Self { r, theta }
    }

    /// Создать полярную точку с углом в градусах
    pub fn from_degrees(r: f32, theta_degrees: f32) -> Self {
        Self {
            r,
            theta: theta_degrees.to_radians(),
        }
    }

    /// Получить угол в градусах
    pub fn theta_degrees(&self) -> f32 {
        self.theta.to_degrees()
    }
}

/// Структура для представления точки в декартовых координатах
#[derive(Clone, Copy, Debug)]
pub struct CartesianPoint {
    pub x: f32,
    pub y: f32,
}

impl CartesianPoint {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Функции преобразования координат
pub mod conversions {
    use super::*;

    /// Преобразование из полярных в декартовы координаты
    pub fn polar_to_cartesian(polar: PolarPoint) -> CartesianPoint {
        CartesianPoint {
            x: polar.r * polar.theta.cos(),
            y: polar.r * polar.theta.sin(),
        }
    }

    /// Преобразование из декартовых в полярные координаты
    pub fn cartesian_to_polar(cartesian: CartesianPoint) -> PolarPoint {
        let r = (cartesian.x * cartesian.x + cartesian.y * cartesian.y).sqrt();
        let theta = cartesian.y.atan2(cartesian.x);
        PolarPoint { r, theta }
    }

    /// Преобразование Vec3 (мировые координаты Bevy) в декартовы координаты 2D
    pub fn world_to_cartesian(world_pos: Vec3) -> CartesianPoint {
        CartesianPoint {
            x: world_pos.x,
            y: world_pos.z, // В Bevy Z - это глубина, но в 2D это Y
        }
    }

    /// Преобразование декартовых координат 2D в Vec3 (мировые координаты Bevy)
    pub fn cartesian_to_world(cartesian: CartesianPoint) -> Vec3 {
        Vec3::new(cartesian.x, 0.0, cartesian.y)
    }

    /// Преобразование Vec3 в полярные координаты
    pub fn world_to_polar(world_pos: Vec3) -> PolarPoint {
        let cartesian = world_to_cartesian(world_pos);
        cartesian_to_polar(cartesian)
    }

    /// Преобразование полярных координат в Vec3
    pub fn polar_to_world(polar: PolarPoint) -> Vec3 {
        let cartesian = polar_to_cartesian(polar);
        cartesian_to_world(cartesian)
    }
}

/// Компонент для хранения информации о точке в разных системах координат
#[derive(Component, Clone)]
pub struct CoordinatePoint {
    pub world_position: Vec3,
    pub cartesian: CartesianPoint,
    pub polar: PolarPoint,
}

impl CoordinatePoint {
    pub fn from_world(world_pos: Vec3) -> Self {
        let cartesian = conversions::world_to_cartesian(world_pos);
        let polar = conversions::cartesian_to_polar(cartesian);
        Self {
            world_position: world_pos,
            cartesian,
            polar,
        }
    }

    pub fn from_cartesian(cartesian: CartesianPoint) -> Self {
        let world_position = conversions::cartesian_to_world(cartesian);
        let polar = conversions::cartesian_to_polar(cartesian);
        Self {
            world_position,
            cartesian,
            polar,
        }
    }

    pub fn from_polar(polar: PolarPoint) -> Self {
        let cartesian = conversions::polar_to_cartesian(polar);
        let world_position = conversions::cartesian_to_world(cartesian);
        Self {
            world_position,
            cartesian,
            polar,
        }
    }
}

/// Утилиты для форматирования координат
pub mod formatting {
    use super::*;

    /// Форматирование декартовых координат
    pub fn format_cartesian(point: CartesianPoint) -> String {
        format!("({:.2}, {:.2})", point.x, point.y)
    }

    /// Форматирование полярных координат
    pub fn format_polar(point: PolarPoint, angle_unit: AngleUnit) -> String {
        match angle_unit {
            AngleUnit::Degrees => {
                format!("(r: {:.2}, θ: {:.1}°)", point.r, point.theta_degrees())
            }
            AngleUnit::Radians => {
                format!("(r: {:.2}, θ: {:.3} рад)", point.r, point.theta)
            }
        }
    }

    /// Расчет и форматирование длины отрезка
    pub fn format_line_length(start: Vec3, end: Vec3) -> String {
        let length = (end - start).length();
        format!("{:.2}", length)
    }

    /// Расчет и форматирование угла наклона отрезка
    pub fn format_line_angle(start: Vec3, end: Vec3, angle_unit: AngleUnit) -> String {
        let direction = end - start;
        let angle_rad = direction.z.atan2(direction.x);
        match angle_unit {
            AngleUnit::Degrees => format!("{:.1}°", angle_rad.to_degrees()),
            AngleUnit::Radians => format!("{:.3} рад", angle_rad),
        }
    }
}

/// Система для обработки изменений системы координат (использует события из events.rs)
pub fn handle_coordinate_system_events(
    mut coordinate_events: EventReader<crate::events::CoordinateSystemChangeEvent>,
    mut angle_events: EventReader<crate::events::AngleUnitChangeEvent>,
    mut settings: ResMut<CoordinateSettings>,
) {
    for event in coordinate_events.read() {
        settings.coordinate_system = event.new_system;
        info!("Coordinate system changed to: {:?}", event.new_system);
    }

    for event in angle_events.read() {
        settings.angle_unit = event.new_unit;
        info!("Angle unit changed to: {:?}", event.new_unit);
    }
}

/// Клавиатурные команды для переключения систем координат
pub fn keyboard_coordinate_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut coordinate_events: EventWriter<crate::events::CoordinateSystemChangeEvent>,
    mut angle_events: EventWriter<crate::events::AngleUnitChangeEvent>,
    settings: Res<CoordinateSettings>,
) {
    // Клавиша X - переключение системы координат
    if keyboard_input.just_pressed(KeyCode::KeyX) {
        let new_system = match settings.coordinate_system {
            CoordinateSystem::Cartesian => CoordinateSystem::Polar,
            CoordinateSystem::Polar => CoordinateSystem::Cartesian,
        };
        coordinate_events.send(crate::events::CoordinateSystemChangeEvent { new_system });
    }

    // Клавиша U - переключение единиц измерения углов
    if keyboard_input.just_pressed(KeyCode::KeyU) {
        let new_unit = match settings.angle_unit {
            AngleUnit::Degrees => AngleUnit::Radians,
            AngleUnit::Radians => AngleUnit::Degrees,
        };
        angle_events.send(crate::events::AngleUnitChangeEvent { new_unit });
    }
}
