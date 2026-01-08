use bevy::prelude::*;
use crate::components::*;
use crate::battle::BattleState;
use crate::genetics::Population;
use crate::camera::CameraState;

/// Ресурс с хэндлами UI (шрифты и т.п.)
#[derive(Resource, Clone)]
#[allow(dead_code)]
pub struct UiAssets {
    pub font: Handle<Font>,
}

/// Система отображения UI
pub fn ui_system(
    _tank_query: Query<(&Tank, &Transform, Option<&AIController>)>,
    _battle_state: Res<BattleState>,
    _population: Res<Population>,
    _camera_state: Res<CameraState>,
) {
    // Здесь можно добавить UI используя bevy_egui или встроенный UI Bevy
    // Пока оставим заглушку
}

/// Компонент для отображения статистики
#[derive(Component)]
pub struct StatsText;

/// Система для создания UI элементов
pub fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Загружаем шрифт (положите файл в assets/fonts/FiraSans-Bold.ttf)
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(UiAssets { font: font.clone() });

    // Создаем текст для статистики
    commands.spawn((
        TextBundle::from_section(
            "Статистика",
            TextStyle {
                font,
                font_size: 20.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        StatsText,
    ));
}

/// Обновление UI со статистикой
pub fn update_stats_ui(
    mut query: Query<&mut Text, With<StatsText>>,
    tank_query: Query<&Tank>,
    population: Res<Population>,
    battle_state: Res<BattleState>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let alive_tanks = tank_query.iter().count();
        let team0_count = tank_query.iter().filter(|t| t.team == 0).count();
        let team1_count = tank_query.iter().filter(|t| t.team == 1).count();
        
        text.sections[0].value = format!(
            "Поколение: {}\n\
             Танков живых: {}\n\
             Команда 0 (синие): {}\n\
             Команда 1 (красные): {}\n\
             Время боя: {:.1}s\n\
             \n\
             Управление:\n\
             WASD - движение танка (если управляете)\n\
             Пробел - выстрел\n\
             Tab - переключение камеры\n\
             Стрелки - движение камеры\n\
             +/- - zoom\n\
             ЛКМ на танк - выбрать для вида от 3-го лица",
            population.generation,
            alive_tanks,
            team0_count,
            team1_count,
            battle_state.battle_time,
        );
    }
}
