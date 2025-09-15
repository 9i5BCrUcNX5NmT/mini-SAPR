use bevy::{pbr::wireframe::WireframeConfig, prelude::*};

pub fn toggle_render_mode_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut render_modes: ResMut<RenderModes>,
    mut wireframe_config: ResMut<WireframeConfig>,
) {
    // F1 - переключение wireframe
    if keyboard_input.just_pressed(KeyCode::F1) {
        render_modes.wireframe_mode = !render_modes.wireframe_mode;
        wireframe_config.global = render_modes.wireframe_mode;

        println!("Wireframe mode: {}", render_modes.wireframe_mode);
    }

    // F2 - переключение освещения
    if keyboard_input.just_pressed(KeyCode::F2) {
        render_modes.lighting_enabled = !render_modes.lighting_enabled;
        println!("Lighting: {}", render_modes.lighting_enabled);
    }

    // F3 - переключение теней
    if keyboard_input.just_pressed(KeyCode::F3) {
        render_modes.shadows_enabled = !render_modes.shadows_enabled;
        println!("Shadows: {}", render_modes.shadows_enabled);
    }

    // F4 - переключение сетки
    if keyboard_input.just_pressed(KeyCode::F4) {
        render_modes.grid_visible = !render_modes.grid_visible;
        println!("Grid visible: {}", render_modes.grid_visible);
    }
}

// Система для обновления материалов при изменении режимов
pub fn update_materials_system(
    render_modes: Res<RenderModes>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material_query: Query<&MeshMaterial3d<StandardMaterial>>,
) {
    if render_modes.is_changed() {
        for material_handle in material_query.iter_mut() {
            if let Some(material) = materials.get_mut(&material_handle.0) {
                // Переключение между lit/unlit режимами
                material.unlit = !render_modes.lighting_enabled;
            }
        }
    }
}

// Система для управления освещением и тенями
pub fn toggle_lighting_system(
    render_modes: Res<RenderModes>,
    mut light_query: Query<&mut PointLight>,
    mut directional_light_query: Query<&mut DirectionalLight>,
) {
    if render_modes.is_changed() {
        // Управление точечными источниками света
        for mut light in light_query.iter_mut() {
            light.shadows_enabled = render_modes.shadows_enabled && render_modes.lighting_enabled;
            // Можно также изменить интенсивность
            light.intensity = if render_modes.lighting_enabled {
                10_000_000.0
            } else {
                0.0
            };
        }

        // Управление направленными источниками света
        for mut light in directional_light_query.iter_mut() {
            light.shadows_enabled = render_modes.shadows_enabled && render_modes.lighting_enabled;
        }
    }
}

// Ресурс для управления режимами рендеринга
#[derive(Resource)]
pub struct RenderModes {
    wireframe_mode: bool,
    lighting_enabled: bool,
    shadows_enabled: bool,
    pub(crate) grid_visible: bool,
}

impl Default for RenderModes {
    fn default() -> Self {
        Self {
            wireframe_mode: false,
            lighting_enabled: false,
            shadows_enabled: false,
            grid_visible: true,
        }
    }
}
