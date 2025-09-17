use bevy::{pbr::wireframe::WireframeConfig, prelude::*};

// Оптимизированная система переключения режимов рендеринга
pub fn toggle_render_mode_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut render_modes: ResMut<RenderModes>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    let mut changed = false;

    // F1 - переключение wireframe
    if keyboard_input.just_pressed(KeyCode::F1) {
        render_modes.wireframe_mode = !render_modes.wireframe_mode;
        wireframe_config.global = render_modes.wireframe_mode;
        info!("Wireframe mode: {}", render_modes.wireframe_mode);
        changed = true;
    }

    // F2 - переключение освещения
    if keyboard_input.just_pressed(KeyCode::F2) {
        render_modes.lighting_enabled = !render_modes.lighting_enabled;
        info!("Lighting: {}", render_modes.lighting_enabled);
        changed = true;
    }

    // F3 - переключение теней
    if keyboard_input.just_pressed(KeyCode::F3) {
        render_modes.shadows_enabled = !render_modes.shadows_enabled;
        info!("Shadows: {}", render_modes.shadows_enabled);
        changed = true;
    }

    // F4 - переключение сетки
    if keyboard_input.just_pressed(KeyCode::F4) {
        render_modes.grid_visible = !render_modes.grid_visible;
        info!("Grid visible: {}", render_modes.grid_visible);
        changed = true;
    }

    // Отмечаем ресурс как измененный только если были изменения
    if changed {
        render_modes.set_changed();
    }
}

// Оптимизированная система обновления материалов с Change Detection
pub fn update_materials_system(
    render_modes: Res<RenderModes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    // Используем Change Detection для оптимизации
    if !render_modes.is_changed() {
        return;
    }

    // Собираем все уникальные handle материалов
    let mut material_handles: std::collections::HashSet<AssetId<StandardMaterial>> =
        std::collections::HashSet::new();

    material_query.iter().for_each(|material_handle| {
        material_handles.insert(material_handle.0.id());
    });

    // Обновляем только уникальные материалы
    for handle_id in material_handles {
        if let Some(material) = materials.get_mut(handle_id) {
            // Переключение между lit/unlit режимами
            material.unlit = !render_modes.lighting_enabled;
        }
    }
}

// Оптимизированная система управления освещением и тенями
pub fn toggle_lighting_system(
    render_modes: Res<RenderModes>,
    mut point_light_query: Query<&mut PointLight>,
    mut directional_light_query: Query<&mut DirectionalLight>,
) {
    // Используем Change Detection для оптимизации
    if !render_modes.is_changed() {
        return;
    }

    // Параллельная обработка точечных источников света
    point_light_query.par_iter_mut().for_each(|mut light| {
        light.shadows_enabled = render_modes.shadows_enabled && render_modes.lighting_enabled;
        light.intensity = if render_modes.lighting_enabled {
            10_000_000.0 // Bevy 0.16 интенсивность
        } else {
            0.0
        };
    });

    // Управление направленными источниками света
    directional_light_query
        .par_iter_mut()
        .for_each(|mut light| {
            light.shadows_enabled = render_modes.shadows_enabled && render_modes.lighting_enabled;
        });
}

// Ресурс для управления режимами рендеринга с улучшенной структурой
#[derive(Resource)]
pub struct RenderModes {
    wireframe_mode: bool,
    lighting_enabled: bool,
    shadows_enabled: bool,
    pub grid_visible: bool,
}

impl Default for RenderModes {
    fn default() -> Self {
        Self {
            wireframe_mode: false,
            lighting_enabled: true, // Включаем освещение по умолчанию
            shadows_enabled: true,  // Включаем тени по умолчанию
            grid_visible: true,
        }
    }
}

impl RenderModes {
    // // Методы для безопасного доступа к приватным полям
    // pub fn wireframe_mode(&self) -> bool {
    //     self.wireframe_mode
    // }

    // pub fn lighting_enabled(&self) -> bool {
    //     self.lighting_enabled
    // }

    // pub fn shadows_enabled(&self) -> bool {
    //     self.shadows_enabled
    // }

    // Метод для получения информации о всех режимах
    pub fn get_info(&self) -> String {
        format!(
            "Render modes - Wireframe: {}, Lighting: {}, Shadows: {}, Grid: {}",
            self.wireframe_mode, self.lighting_enabled, self.shadows_enabled, self.grid_visible
        )
    }
}

// Дополнительная система для отображения информации о режимах рендеринга
pub fn display_render_info_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    render_modes: Res<RenderModes>,
) {
    if keyboard_input.just_pressed(KeyCode::F5) {
        info!("{}", render_modes.get_info());
    }
}

// Система для сохранения настроек рендеринга
pub fn save_render_settings_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    render_modes: Res<RenderModes>,
) {
    if keyboard_input.just_pressed(KeyCode::F12) {
        // В реальном приложении здесь можно сохранять настройки в файл
        info!("Saved render settings: {}", render_modes.get_info());
    }
}
