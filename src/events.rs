use crate::coordinate_systems::{AngleUnit, CoordinateSystem};
use bevy::prelude::*;

// === СОБЫТИЯ ДЛЯ КАМЕРЫ ===
#[derive(Event)]
pub struct CameraToggleEvent;

#[derive(Event)]
pub struct CameraZoomEvent {
    pub zoom_delta: f32,
}

#[derive(Event)]
pub struct CameraResetEvent;

// === СОБЫТИЯ ДЛЯ КООРДИНАТНЫХ СИСТЕМ ===
#[derive(Event)]
pub struct CoordinateSystemChangeEvent {
    pub new_system: CoordinateSystem,
}

#[derive(Event)]
pub struct AngleUnitChangeEvent {
    pub new_unit: AngleUnit,
}

// === СОБЫТИЯ ДЛЯ ЛИНИЙ ===
#[derive(Event)]
pub struct CreateLineEvent;

#[derive(Event)]
pub struct DeleteAllLinesEvent;

#[derive(Event)]
pub struct LineCreatedEvent {
    pub line_id: u32,
    pub start: Vec3,
    pub end: Vec3,
}

#[derive(Event)]
pub struct PointSelectedEvent {
    pub point: Vec3,
    pub is_start: bool,
}

// === СОБЫТИЯ ДЛЯ СЕТКИ ===
#[derive(Event)]
pub struct GridStepChangeEvent {
    pub new_step: f32,
}

// === ДЕЙСТВИЯ UI (унифицированы с событиями) ===
#[derive(Component, Clone)]
pub enum UIAction {
    // Действия с линиями
    CreateLine,
    DeleteAll,

    // Переключение систем координат
    ToggleCoordinateSystem,
    SetCartesian,
    SetPolar,

    // Переключение единиц углов
    ToggleAngleUnit,
    SetDegrees,
    SetRadians,

    // Сетка
    SetGridStep(f32),

    // Камера
    ToggleCamera,
    ResetCamera,
}

impl UIAction {
    /// Конвертирует UI действие в соответствующие события
    pub fn emit_events(
        &self,
        coordinate_events: &mut EventWriter<CoordinateSystemChangeEvent>,
        angle_events: &mut EventWriter<AngleUnitChangeEvent>,
        line_events: &mut EventWriter<CreateLineEvent>,
        delete_events: &mut EventWriter<DeleteAllLinesEvent>,
        grid_events: &mut EventWriter<GridStepChangeEvent>,
        camera_toggle_events: &mut EventWriter<CameraToggleEvent>,
        camera_reset_events: &mut EventWriter<CameraResetEvent>,
    ) {
        match self {
            UIAction::CreateLine => {
                line_events.write(CreateLineEvent);
            }
            UIAction::DeleteAll => {
                delete_events.write(DeleteAllLinesEvent);
            }
            UIAction::ToggleCoordinateSystem => {
                // Логика переключения будет в системе обработки
            }
            UIAction::SetCartesian => {
                coordinate_events.write(CoordinateSystemChangeEvent {
                    new_system: CoordinateSystem::Cartesian,
                });
            }
            UIAction::SetPolar => {
                coordinate_events.write(CoordinateSystemChangeEvent {
                    new_system: CoordinateSystem::Polar,
                });
            }
            UIAction::ToggleAngleUnit => {
                // Логика переключения будет в системе обработки
            }
            UIAction::SetDegrees => {
                angle_events.write(AngleUnitChangeEvent {
                    new_unit: AngleUnit::Degrees,
                });
            }
            UIAction::SetRadians => {
                angle_events.write(AngleUnitChangeEvent {
                    new_unit: AngleUnit::Radians,
                });
            }
            UIAction::SetGridStep(step) => {
                grid_events.write(GridStepChangeEvent { new_step: *step });
            }
            UIAction::ToggleCamera => {
                camera_toggle_events.write(CameraToggleEvent);
            }
            UIAction::ResetCamera => {
                camera_reset_events.write(CameraResetEvent);
            }
        }
    }
}
