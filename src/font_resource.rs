use bevy::prelude::*;

/// Глобальный ресурс для шрифта
#[derive(Resource)]
pub struct GlobalFont {
    pub handle: Handle<Font>,
}

/// Система инициализации глобального шрифта с проверками
pub fn setup_global_font(mut commands: Commands, asset_server: Res<AssetServer>) {
    // ВАРИАНТ 1: Пытаемся загрузить внешний шрифт
    let font_path = r"fonts/Hack/HackNerdFont-Regular.ttf";
    let font_handle = asset_server.load(font_path);

    // Выводим отладочную информацию
    info!("🔤 Загружаем шрифт: {}", font_path);
    info!("🔗 Handle шрифта: {:?}", font_handle);

    commands.insert_resource(GlobalFont {
        handle: font_handle,
    });
}

// /// АЛЬТЕРНАТИВНАЯ система с встроенным шрифтом Bevy
// pub fn setup_builtin_font(mut commands: Commands) {
//     // Используем встроенный шрифт Bevy (FiraSans)
//     let builtin_font = Handle::<Font>::default();

//     info!("🔤 Используем встроенный шрифт Bevy");

//     commands.insert_resource(GlobalFont {
//         handle: builtin_font,
//     });
// }

// /// Система с загрузкой шрифта из массива байт (встраиваем в бинарь)
// pub fn setup_embedded_font(mut commands: Commands, asset_server: Res<AssetServer>) {
//     // Встраиваем шрифт прямо в исполняемый файл
//     // Сначала поместите Roboto-Regular.ttf в src/assets/
//     const FONT_DATA: &[u8] =
//         include_bytes!("../assets/fonts/Roboto/Roboto-Italic-VariableFont_wdth,wght.ttf");

//     // Создаем шрифт из байтов
//     let font =
//         Font::try_from_bytes(FONT_DATA.to_vec()).expect("Не удалось загрузить встроенный шрифт");

//     // Добавляем в Assets вручную
//     let mut font_assets = commands
//         .insert_resource::<Assets<Font>>()
//         .expect("Assets<Font> не найден");
//     let font_handle = font_assets.add(font);

//     info!("🔤 Встроенный шрифт загружен из байтов");

//     commands.insert_resource(GlobalFont {
//         handle: font_handle,
//     });
// }
