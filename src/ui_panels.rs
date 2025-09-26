use crate::{
    coordinate_systems::{
        formatting, AngleUnit, CoordinatePoint, CoordinateSettings, CoordinateSystem,
    },
    events::*,                 // Используем централизованный модуль событий
    font_resource::GlobalFont, // ИМПОРТ глобального шрифта
    line_drawing::DrawableLine,
};
use bevy::prelude::*;

/// Ресурс для отслеживания текущей позиции курсора
#[derive(Resource, Default)]
pub struct CursorInfo {
    pub world_position: Option<Vec3>,
    pub is_valid: bool,
}

/// Компоненты для различных панелей UI
#[derive(Component)]
pub struct ToolPanel;

#[derive(Component)]
pub struct SettingsPanel;

#[derive(Component)]
pub struct InfoPanel;

#[derive(Component)]
pub struct CoordinateDisplay;

#[derive(Component)]
pub struct LineInfoDisplay;

/// Настройка основного UI для Bevy 0.15+ с глобальным шрифтом
pub fn setup_ui_panels(
    mut commands: Commands,
    global_font: Res<GlobalFont>, // ИСПОЛЬЗУЕМ глобальный шрифт
) {
    // Создаем главный контейнер UI
    let main_container = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::NONE),
            Name::new("MainUIContainer"),
        ))
        .id();

    // === ВЕРХНЯЯ ПАНЕЛЬ ИНСТРУМЕНТОВ ===
    let tool_panel = create_tool_panel(&mut commands, &global_font);
    commands
        .entity(main_container)
        .insert_children(0, &[tool_panel]);

    // === СРЕДНЯЯ ОБЛАСТЬ (с боковыми панелями и рабочим пространством) ===
    let middle_area = create_middle_area(&mut commands, &global_font);
    commands
        .entity(main_container)
        .insert_children(0, &[middle_area]);

    // === НИЖНЯЯ СТАТУС-ПАНЕЛЬ ===
    let status_panel = create_status_panel(&mut commands, &global_font);
    commands
        .entity(main_container)
        .insert_children(0, &[status_panel]);
}

/// Создание верхней панели инструментов
fn create_tool_panel(commands: &mut Commands, global_font: &Res<GlobalFont>) -> Entity {
    let tool_panel = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(8.0),
                padding: UiRect::all(Val::Px(5.0)),
                border: UiRect::bottom(Val::Px(2.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            ToolPanel,
            Name::new("ToolPanel"),
        ))
        .id();

    // Создаем кнопки с РУССКИМ ТЕКСТОМ
    let line_button = create_button(commands, global_font, "Линия", UIAction::CreateLine);
    let delete_button = create_button(commands, global_font, "Удалить всё", UIAction::DeleteAll);
    let separator1 = create_separator(commands);
    let coord_button = create_button(
        commands,
        global_font,
        "Координаты",
        UIAction::ToggleCoordinateSystem,
    );
    let angle_button = create_button(commands, global_font, "Углы", UIAction::ToggleAngleUnit);

    // Добавляем кнопки как дочерние к панели
    commands.entity(tool_panel).insert_children(
        0,
        &[
            line_button,
            delete_button,
            separator1,
            coord_button,
            angle_button,
        ],
    );

    tool_panel
}

/// Создание средней области с боковыми панелями
fn create_middle_area(commands: &mut Commands, global_font: &Res<GlobalFont>) -> Entity {
    let middle_area = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(85.0),
                flex_direction: FlexDirection::Row,
                ..default()
            },
            Name::new("MiddleArea"),
        ))
        .id();

    // Левая панель настроек
    let settings_panel = create_settings_panel(commands, global_font);
    commands
        .entity(middle_area)
        .insert_children(0, &[settings_panel]);

    // Рабочая область
    let work_area = commands
        .spawn((
            Node {
                width: Val::Percent(70.0),
                height: Val::Percent(100.0),
                ..default()
            },
            Name::new("WorkArea"),
        ))
        .id();
    commands
        .entity(middle_area)
        .insert_children(0, &[work_area]);

    // Правая панель информации
    let info_panel = create_info_panel(commands, global_font);
    commands
        .entity(middle_area)
        .insert_children(0, &[info_panel]);

    middle_area
}

/// Создание левой панели настроек
fn create_settings_panel(commands: &mut Commands, global_font: &Res<GlobalFont>) -> Entity {
    let settings_panel = commands
        .spawn((
            Node {
                width: Val::Percent(15.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border: UiRect::right(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            BorderColor(Color::srgb(0.2, 0.2, 0.2)),
            SettingsPanel,
            Name::new("SettingsPanel"),
        ))
        .id();

    // Заголовок панели с РУССКИМ ТЕКСТОМ
    let title = commands
        .spawn((
            Text::new("Настройки"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
            Name::new("SettingsTitle"),
        ))
        .id();
    commands.entity(settings_panel).insert_children(0, &[title]);

    // Секция координатных систем
    let coord_section = create_coordinate_section(commands, global_font);
    commands
        .entity(settings_panel)
        .insert_children(0, &[coord_section]);

    // Секция настроек сетки
    let grid_section = create_grid_section(commands, global_font);
    commands
        .entity(settings_panel)
        .insert_children(0, &[grid_section]);

    settings_panel
}

/// Создание секции координатных систем
fn create_coordinate_section(commands: &mut Commands, global_font: &Res<GlobalFont>) -> Entity {
    let section = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            Name::new("CoordinateSection"),
        ))
        .id();

    // Заголовок секции с РУССКИМ ТЕКСТОМ
    let title = commands
        .spawn((
            Text::new("Система координат:"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ))
        .id();

    // Кнопки координатных систем с РУССКИМ ТЕКСТОМ
    let cartesian_button =
        create_small_button(commands, global_font, "Декартовы", UIAction::SetCartesian);
    let polar_button = create_small_button(commands, global_font, "Полярные", UIAction::SetPolar);

    // Заголовок углов с РУССКИМ ТЕКСТОМ
    let angle_title = commands
        .spawn((
            Text::new("Единицы углов:"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ))
        .id();

    // Кнопки единиц углов с РУССКИМ ТЕКСТОМ
    let degrees_button =
        create_small_button(commands, global_font, "Градусы", UIAction::SetDegrees);
    let radians_button =
        create_small_button(commands, global_font, "Радианы", UIAction::SetRadians);

    // Добавляем все элементы в секцию
    commands.entity(section).insert_children(
        0,
        &[
            title,
            cartesian_button,
            polar_button,
            angle_title,
            degrees_button,
            radians_button,
        ],
    );

    section
}

/// Создание секции настроек сетки
fn create_grid_section(commands: &mut Commands, global_font: &Res<GlobalFont>) -> Entity {
    let section = commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(5.0),
                ..default()
            },
            Name::new("GridSection"),
        ))
        .id();

    // Заголовок секции с РУССКИМ ТЕКСТОМ
    let title = commands
        .spawn((
            Text::new("Настройки сетки:"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 12.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ))
        .id();
    commands.entity(section).insert_children(0, &[title]);

    // Кнопки шага сетки с РУССКИМ ТЕКСТОМ
    let grid_steps = [0.5, 1.0, 2.0, 5.0];
    let mut grid_buttons = Vec::new();
    for step in grid_steps {
        let button = create_small_button(
            commands,
            global_font,
            &format!("Шаг: {}", step),
            UIAction::SetGridStep(step),
        );
        grid_buttons.push(button);
    }

    // Добавляем кнопки в секцию
    commands.entity(section).insert_children(0, &grid_buttons);

    section
}

/// Создание правой панели информации
fn create_info_panel(commands: &mut Commands, global_font: &Res<GlobalFont>) -> Entity {
    let info_panel = commands
        .spawn((
            Node {
                width: Val::Percent(15.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(10.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                border: UiRect::left(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.05, 0.05, 0.05)),
            BorderColor(Color::srgb(0.2, 0.2, 0.2)),
            InfoPanel,
            Name::new("InfoPanel"),
        ))
        .id();

    // Заголовок панели с РУССКИМ ТЕКСТОМ
    let title = commands
        .spawn((
            Text::new("Информация"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 16.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ))
        .id();

    // Координаты курсора с РУССКИМ ТЕКСТОМ
    let cursor_coords = commands
        .spawn((
            Text::new("X: -, Y: -"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Name::new("CursorCoords"),
            CoordinateDisplay,
        ))
        .id();

    // Информация о линиях с РУССКИМ ТЕКСТОМ
    let line_info = commands
        .spawn((
            Text::new("Линий: 0"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
            Name::new("LineInfo"),
            LineInfoDisplay,
        ))
        .id();

    // Добавляем элементы в панель
    commands
        .entity(info_panel)
        .insert_children(0, &[title, cursor_coords, line_info]);

    info_panel
}

/// Создание нижней статус-панели
fn create_status_panel(commands: &mut Commands, global_font: &Res<GlobalFont>) -> Entity {
    let status_panel = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(7.0),
                padding: UiRect::all(Val::Px(10.0)),
                border: UiRect::top(Val::Px(1.0)),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.08, 0.08, 0.08)),
            BorderColor(Color::srgb(0.2, 0.2, 0.2)),
            Name::new("StatusPanel"),
        ))
        .id();

    // Статус текст с РУССКИМ ТЕКСТОМ
    let status_text = commands
        .spawn((
            Text::new("Готов к работе"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
        ))
        .id();

    // Подсказки горячих клавиш с РУССКИМ ТЕКСТОМ
    let hotkeys_text = commands
        .spawn((
            Text::new("F1-F4: Рендер | L: Линии | X: Координаты | U: Углы"),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgb(0.6, 0.6, 0.6)),
        ))
        .id();

    // Добавляем тексты в панель
    commands
        .entity(status_panel)
        .insert_children(0, &[status_text, hotkeys_text]);

    status_panel
}

/// Создание кнопки - Bevy 0.15+ синтаксис с глобальным шрифтом
fn create_button(
    commands: &mut Commands,
    global_font: &Res<GlobalFont>,
    text: &str,
    action: UIAction,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            Node {
                width: Val::Px(120.0),
                height: Val::Px(30.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
            BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            BorderRadius::all(Val::Px(3.0)),
            action,
            Name::new(format!("Button_{}", text)),
        ))
        .id();

    // Создаем текст как дочерний элемент с ГЛОБАЛЬНЫМ ШРИФТОМ
    let text_entity = commands
        .spawn((
            Text::new(text),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 11.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ))
        .id();

    // Добавляем текст к кнопке
    commands.entity(button).insert_children(0, &[text_entity]);

    button
}

/// Создание маленькой кнопки
fn create_small_button(
    commands: &mut Commands,
    global_font: &Res<GlobalFont>,
    text: &str,
    action: UIAction,
) -> Entity {
    let button = commands
        .spawn((
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(25.0),
                border: UiRect::all(Val::Px(1.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            BorderRadius::all(Val::Px(2.0)),
            action,
            Name::new(format!("SmallButton_{}", text)),
        ))
        .id();

    // Создаем текст как дочерний элемент с ГЛОБАЛЬНЫМ ШРИФТОМ
    let text_entity = commands
        .spawn((
            Text::new(text),
            TextFont {
                font: global_font.handle.clone(), // ИСПОЛЬЗУЕМ глобальный шрифт
                font_size: 10.0,
                ..default()
            },
            TextColor(Color::srgb(0.9, 0.9, 0.9)),
        ))
        .id();

    // Добавляем текст к кнопке
    commands.entity(button).insert_children(0, &[text_entity]);

    button
}

/// Создание разделителя
fn create_separator(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Node {
                width: Val::Px(2.0),
                height: Val::Percent(80.0),
                margin: UiRect::horizontal(Val::Px(5.0)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.3, 0.3, 0.3)),
            Name::new("Separator"),
        ))
        .id()
}

/// Унифицированная система обработки UI взаимодействий
pub fn handle_ui_interactions(
    interaction_query: Query<(&Interaction, &UIAction), (Changed<Interaction>, With<Button>)>,
    mut coordinate_events: EventWriter<CoordinateSystemChangeEvent>,
    mut angle_events: EventWriter<AngleUnitChangeEvent>,
    mut line_events: EventWriter<CreateLineEvent>,
    mut delete_events: EventWriter<DeleteAllLinesEvent>,
    mut grid_events: EventWriter<GridStepChangeEvent>,
    mut camera_toggle_events: EventWriter<CameraToggleEvent>,
    mut camera_reset_events: EventWriter<CameraResetEvent>,
    coordinate_settings: Res<CoordinateSettings>,
) {
    for (interaction, action) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            match action {
                UIAction::ToggleCoordinateSystem => {
                    let new_system = match coordinate_settings.coordinate_system {
                        CoordinateSystem::Cartesian => CoordinateSystem::Polar,
                        CoordinateSystem::Polar => CoordinateSystem::Cartesian,
                    };
                    coordinate_events.write(CoordinateSystemChangeEvent { new_system });
                }

                UIAction::ToggleAngleUnit => {
                    let new_unit = match coordinate_settings.angle_unit {
                        AngleUnit::Degrees => AngleUnit::Radians,
                        AngleUnit::Radians => AngleUnit::Degrees,
                    };
                    angle_events.write(AngleUnitChangeEvent { new_unit });
                }

                _ => {
                    // Для остальных действий используем централизованную логику
                    action.emit_events(
                        &mut coordinate_events,
                        &mut angle_events,
                        &mut line_events,
                        &mut delete_events,
                        &mut grid_events,
                        &mut camera_toggle_events,
                        &mut camera_reset_events,
                    );
                }
            }
        }
    }
}

/// Система обновления информации о координатах курсора
pub fn update_cursor_coordinates(
    cursor_info: Res<CursorInfo>,
    coordinate_settings: Res<CoordinateSettings>,
    mut text_query: Query<&mut Text, With<CoordinateDisplay>>,
) {
    if !cursor_info.is_changed() {
        return;
    }

    for mut text in text_query.iter_mut() {
        if let Some(world_pos) = cursor_info.world_position {
            let coord_point = CoordinatePoint::from_world(world_pos);
            let coords_text = match coordinate_settings.coordinate_system {
                CoordinateSystem::Cartesian => formatting::format_cartesian(coord_point.cartesian),
                CoordinateSystem::Polar => {
                    formatting::format_polar(coord_point.polar, coordinate_settings.angle_unit)
                }
            };
            **text = format!("Курсор: {}", coords_text);
        } else {
            **text = "Курсор: X: -, Y: -".to_string();
        }
    }
}

/// Система обновления информации о линиях
pub fn update_line_info(
    line_query: Query<&DrawableLine>,
    mut text_query: Query<&mut Text, With<LineInfoDisplay>>,
) {
    for mut text in text_query.iter_mut() {
        let line_count = line_query.iter().count();
        **text = format!("Линий: {}", line_count);
    }
}

/// Система отслеживания позиции курсора
pub fn track_cursor_position(
    camera_query: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    window_query: Query<&Window>,
    mut cursor_info: ResMut<CursorInfo>,
) {
    let Ok((camera, camera_transform)) = camera_query.single() else {
        return;
    };
    let Ok(window) = window_query.single() else {
        return;
    };

    if let Some(cursor_position) = window.cursor_position() {
        if let Some(world_position) = crate::line_drawing::screen_to_world_plane(
            cursor_position,
            camera,
            camera_transform,
            Vec3::Y,
            0.0,
        ) {
            cursor_info.world_position = Some(world_position);
            cursor_info.is_valid = true;
        } else {
            cursor_info.is_valid = false;
        }
    } else {
        cursor_info.world_position = None;
        cursor_info.is_valid = false;
    }
}
