use bevy::prelude::*;

use crate::{camera_system::CameraToggleEvent, CameraState, Id};

const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);

// Обновленная система кнопок с событиями
pub fn button_system(
    // Используем Change Detection для оптимизации - обрабатываем только изменившиеся кнопки
    interaction_query: Query<(&Interaction, &Id), (Changed<Interaction>, With<Button>)>,

    // EventWriter'ы для отправки событий
    mut camera_toggle_events: EventWriter<CameraToggleEvent>,

    // Spawn cube resources (остались без изменений)
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ранний выход если нет изменений
    if interaction_query.is_empty() {
        return;
    }

    // Используем for_each для лучшей производительности компилятора
    interaction_query.iter().for_each(|(interaction, id)| {
        if let Interaction::Pressed = *interaction {
            match id.0 {
                1 => {
                    // Отправляем событие вместо прямого вызова системы
                    camera_toggle_events.write(CameraToggleEvent);
                    info!("Camera toggle event sent");
                }
                2 => {
                    // Оптимизированное создание куба (без изменений)
                    let cube_material = materials.add(StandardMaterial {
                        base_color: Color::srgb_u8(124, 144, 255),
                        ..default()
                    });

                    commands.spawn((
                        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
                        MeshMaterial3d(cube_material),
                        Transform::from_xyz(0.0, 0.5, 0.0),
                        Name::new("SpawnedCube"),
                    ));
                }
                _ => {}
            }
        }
    });
}

// Улучшенная функция создания кнопки с лучшими типами
fn spawn_button(name: &str, id: u32, x: f32, y: f32) -> impl Bundle + use<'_> {
    (
        Node {
            width: Val::Percent(x),
            height: Val::Percent(y),
            align_items: AlignItems::Baseline,
            justify_content: JustifyContent::Center,
            ..default()
        },
        children![(
            Button,
            Node {
                width: Val::Px(75.0),
                height: Val::Px(32.5),
                border: UiRect::all(Val::Px(5.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BorderColor(Color::BLACK),
            BorderRadius::MAX,
            BackgroundColor(NORMAL_BUTTON),
            children![(
                Text::new(name),
                TextFont {
                    font_size: 8.0,
                    ..default()
                },
                TextColor(Color::srgb(0.9, 0.9, 0.9)),
                TextShadow::default(),
            )],
            Id(id),
            Name::new(format!("Button_{}", name)), // Добавляем имя для отладки
        )],
    )
}

// Оптимизированная система настройки кнопок
pub fn setup_buttons(mut commands: Commands) {
    // Создаем кнопки более эффективно
    let buttons = [("Camera", 1, 100.0, 100.0), ("Spawn Cube", 2, 50.0, 50.0)];

    for (name, id, x, y) in buttons {
        commands.spawn(spawn_button(name, id, x, y));
    }
}

// Система для очистки интерфейса кнопок при изменении состояния
pub fn update_button_visuals(
    mut button_query: Query<&mut BackgroundColor, (With<Button>, Changed<Interaction>)>,
) {
    for mut color in &mut button_query {
        // Можно добавить визуальную обратную связь для кнопок
        *color = NORMAL_BUTTON.into();
    }
}
