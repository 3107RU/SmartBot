use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use crate::components::*;
use crate::battle::BattleState;
use crate::genetics::Population;
use crate::camera::CameraState;
use crate::systems::TimeMultiplier;

/// Ресурс для отслеживания, был ли показан стартовый выбор
#[derive(Resource, Default)]
pub struct StartupChoiceMade(pub bool);

/// Таймер для автоматического продолжения
#[derive(Resource)]
pub struct StartupTimer(pub Timer);

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

/// Система UI для состояния Setup
pub fn setup_ui_system(
    mut contexts: EguiContexts,
    mut population: ResMut<Population>,
    mut next_state: ResMut<NextState<GameState>>,
    mut choice_made: ResMut<StartupChoiceMade>,
    mut timer: ResMut<StartupTimer>,
    time: Res<Time>,
) {
    if choice_made.0 {
        next_state.set(GameState::Battle);
        return;
    }
    
    timer.0.tick(time.delta());
    
    if timer.0.finished() {
        info!("Автоматическое продолжение");
        choice_made.0 = true;
        next_state.set(GameState::Battle);
        return;
    }
    
    egui::CentralPanel::default().show(contexts.ctx_mut(), |ui| {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.heading("Tank Battle Simulator");
            ui.add_space(50.0);
            
            ui.label(format!("Текущее поколение: {}", population.generation));
            ui.add_space(20.0);
            
            if ui.button("Начать сначала").clicked() {
                *population = crate::genetics::Population::new_fresh(population.genomes.len());
                choice_made.0 = true;
                next_state.set(GameState::Battle);
            }
            
            ui.add_space(10.0);
            
            if ui.button("Продолжить").clicked() {
                choice_made.0 = true;
                next_state.set(GameState::Battle);
            }
            
            ui.add_space(20.0);
            ui.label(format!("Автопродолжение через: {:.1}s", timer.0.remaining_secs()));
        });
    });
}

/// Компонент для отображения статистики
#[derive(Component)]
pub struct StatsText;

/// Система для загрузки UI ассетов
pub fn load_ui_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Загружаем шрифт (положите файл в assets/fonts/FiraSans-Bold.ttf)
    let font: Handle<Font> = asset_server.load("fonts/FiraSans-Bold.ttf");
    commands.insert_resource(UiAssets { font });
}

/// Система для создания UI элементов статистики
pub fn create_stats_ui(mut commands: Commands, ui_assets: Res<UiAssets>) {
    // Создаем текст для статистики
    commands.spawn((
        TextBundle::from_section(
            "Статистика",
            TextStyle {
                font: ui_assets.font.clone(),
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

/// Система для удаления UI статистики
pub fn despawn_stats_ui(mut commands: Commands, query: Query<Entity, With<StatsText>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

/// Обновление UI со статистикой
pub fn update_stats_ui(
    mut query: Query<&mut Text, With<StatsText>>,
    tank_query: Query<&Tank>,
    population: Res<Population>,
    battle_state: Res<BattleState>,
    time_multiplier: Res<TimeMultiplier>,
) {
    if let Ok(mut text) = query.get_single_mut() {
        let alive_tanks = tank_query.iter().count();
        let team0_count = tank_query.iter().filter(|t| t.team == 0).count();
        let team1_count = tank_query.iter().filter(|t| t.team == 1).count();
        
        let current_best = population.genomes.first().map(|g| g.fitness).unwrap_or(0.0);
        let max_fitness = population.best_genome.as_ref().map(|b| b.fitness).unwrap_or(0.0);
        
        text.sections[0].value = format!(
            "Поколение: {}\n\
             Танков живых: {}\n\
             Команда 0 (синие): {}\n\
             Команда 1 (красные): {}\n\
             Время боя: {:.1}s ({:.0} тактов)\n\
             Скорость времени: {:.2}x\n\
             Лучший фитнес: {:.0}\n\
             Максимальный фитнес: {:.0}\n\
             \n\
             F1 — показать/скрыть помощь и слайдер скорости",
            population.generation,
            alive_tanks,
            team0_count,
            team1_count,
            battle_state.real_time,
            battle_state.tick_count,
            time_multiplier.scale,
            current_best,
            max_fitness,
        );
    }
}
