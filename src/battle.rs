use bevy::prelude::*;
use bevy::prelude::shape;
use crate::components::*;
use crate::genetics::Population;
use crate::systems::TimeMultiplier;
use rand::Rng;

#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct BattleState {
    pub battle_time: f32,
    pub max_battle_time: f32,
    pub teams: Vec<TeamStats>,
}

#[allow(dead_code)]
pub struct TeamStats {
    pub team_id: u32,
    pub alive_count: u32,
    pub total_kills: u32,
}

impl Default for TeamStats {
    fn default() -> Self {
        Self {
            team_id: 0,
            alive_count: 0,
            total_kills: 0,
        }
    }
}

/// Начало битвы
pub fn start_battle(
    mut battle_state: ResMut<BattleState>,
) {
    battle_state.battle_time = 0.0;
    battle_state.max_battle_time = 120.0; // 2 минуты на бой
    info!("Битва началась!");
}

/// Завершение битвы
pub fn end_battle(
    mut population: ResMut<Population>,
    query: Query<(Entity, &AIController)>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    // Обновляем фитнес всех танков
    for (entity, ai) in query.iter() {
        population.calculate_fitness(entity, ai);
    }
    
    info!("Битва завершена!");
    next_state.set(crate::GameState::Evolution);
}

/// Создание танков из текущей популяции
pub fn spawn_tanks_from_population(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    population: Res<Population>,
    tank_query: Query<Entity, With<Tank>>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    // Удаляем всех существующих танков
    for entity in tank_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    
    info!("Спавн танков для поколения {}", population.generation);
    
    let mut rng = rand::thread_rng();
    
    // Создаем 10 танков из лучших геномов популяции
    for i in 0..10 {
        let x = rng.gen_range(-40.0..40.0);
        let z = rng.gen_range(-40.0..40.0);
        
        // Используем геном из популяции (предполагаем, что они отсортированы)
        let ai = population.genomes[i % population.genomes.len()].clone();
        
        spawn_tank(
            &mut commands,
            &mut meshes,
            &mut materials,
            Vec3::new(x, 1.0, z),
            (i % 2) as u32, // Команда 0 или 1
            Some(ai),
        );
    }
    
    // Начинаем новый бой
    next_state.set(crate::GameState::Battle);
}

/// Создание одного танка
pub fn spawn_tank(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    team: u32,
    ai_controller: Option<AIController>,
) -> Entity {
    let color = if team == 0 {
        Color::rgb(0.2, 0.5, 0.8) // Синий
    } else {
        Color::rgb(0.8, 0.2, 0.2) // Красный
    };
    
    // Корпус танка
    let tank_entity = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(2.0, 1.0, 3.0))),
            material: materials.add(StandardMaterial {
                base_color: color,
                ..default()
            }),
            transform: Transform::from_translation(position),
            ..default()
        },
        Tank {
            team,
            ..default()
        },
        TeamColor(color),
    )).id();
    
    // Добавляем AI контроллер или метку игрока
    if let Some(ai) = ai_controller {
        commands.entity(tank_entity).insert(ai);
    }
    // Перезарядка
    commands.entity(tank_entity).insert(FireCooldown::default());
    
    // Башня танка
    let turret = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 0.5, 1.5))),
            material: materials.add(StandardMaterial {
                base_color: color,
                ..default()
            }),
            transform: Transform::from_translation(position + Vec3::Y * 0.75),
            ..default()
        },
        TankTurret {
            parent_tank: tank_entity,
        },
    )).id();
    
    commands.entity(tank_entity).add_child(turret);
    
    tank_entity
}

/// Система проверки окончания боя
pub fn check_battle_end(
    mut battle_state: ResMut<BattleState>,
    time: Res<Time>,
    time_multiplier: Res<TimeMultiplier>,
    tank_query: Query<&Tank>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    battle_state.battle_time += time_multiplier.scaled_seconds(&time);
    
    // Проверяем, есть ли танки разных команд
    let mut teams_alive = std::collections::HashSet::new();
    let total_tanks = tank_query.iter().count();
    
    for tank in tank_query.iter() {
        teams_alive.insert(tank.team);
    }
    
    // Бой закончен если:
    // 1. Осталась только одна команда
    // 2. Время вышло
    // 3. Все танки уничтожены
    if teams_alive.len() <= 1 || 
       battle_state.battle_time >= battle_state.max_battle_time ||
       total_tanks == 0 {
        next_state.set(crate::GameState::Evolution);
    }
}

/// Система для устройства боя между разными поколениями
#[allow(dead_code)]
pub fn setup_generation_battle(
    _commands: &mut Commands,
    _meshes: &mut ResMut<Assets<Mesh>>,
    _materials: &mut ResMut<Assets<StandardMaterial>>,
    _population: &Population,
    _generation_a: u32,
    _generation_b: u32,
) {
    // Загружаем геномы разных поколений и устраиваем бой
    // Это можно вызвать вручную через UI
}
