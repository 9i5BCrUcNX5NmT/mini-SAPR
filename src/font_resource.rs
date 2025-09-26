use bevy::prelude::*;

/// Глобальный ресурс для шрифта
#[derive(Resource)]
pub struct GlobalFont {
    pub handle: Handle<Font>,
}

/// Система инициализации глобального шрифта
pub fn setup_global_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    let font = asset_server.load("../assets/fonts/Hack/HackNerdFont-Bold.ttf");
    commands.insert_resource(GlobalFont { handle: font });
}
