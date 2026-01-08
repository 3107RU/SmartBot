use bevy::prelude::*;
use bevy::prelude::shape;
use crate::components::*;
use crate::genetics::Population;
use crate::systems::{BASE_SIM_DT, ProgressLog};
use std::time::{Duration, Instant};
use crate::Headless;
use rand::Rng;

#[derive(Resource, Default)]
#[allow(dead_code)]
pub struct BattleState {
    pub tick_count: u32,
    pub max_ticks: u32,
    pub real_time: f32,
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
    battle_state.tick_count = 0;
    battle_state.real_time = 0.0;
    battle_state.max_ticks = (120.0 / BASE_SIM_DT) as u32; // 120 секунд симуляции
    info!("Битва началась!");
}

/// Завершение битвы
pub fn end_battle(
    mut population: ResMut<Population>,
    query: Query<(Entity, &AIController)>,
    mut next_state: ResMut<NextState<crate::GameState>>,
    mut progress: ResMut<ProgressLog>,
) {
    // Обновляем фитнес всех танков
    for (entity, ai) in query.iter() {
        population.calculate_fitness(entity, ai);
    }

    // Выводим итоги боя в консоль: лучшее за матч и глобальный максимум
    let best_current = population
        .genomes
        .iter()
        .map(|g| g.fitness)
        .fold(0.0_f32, f32::max);
    let best_max = population
        .best_genome
        .as_ref()
        .map(|g| g.fitness)
        .unwrap_or(best_current);
    let now = Instant::now();
    if now.duration_since(progress.last) >= Duration::from_secs(30) {
        progress.last = now;
        println!(
            "Бой завершён. Поколение: {}, лучший фитнес в матче: {:.1}, максимальный фитнес: {:.1}",
            population.generation,
            best_current,
            best_max
        );
    }
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
            Vec3::new(x, 0.5, z), // Центр танка на высоте 0.5 для контакта с землей
            (i % 2) as u32, // Команда 0 или 1
            Some(ai),
        );
    }
    
    // Начинаем новый бой
    next_state.set(crate::GameState::Battle);
}

/// Создание танков без рендера (headless)
pub fn spawn_tanks_headless(
    mut commands: Commands,
    population: Res<Population>,
    tank_query: Query<Entity, With<Tank>>,
    mut next_state: ResMut<NextState<crate::GameState>>,
    headless: Res<Headless>,
) {
    if !headless.0 {
        return;
    }

    for entity in tank_query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    info!("[Headless] Спавн танков для поколения {}", population.generation);

    let mut rng = rand::thread_rng();
    for i in 0..10 {
        let x = rng.gen_range(-40.0..40.0);
        let z = rng.gen_range(-40.0..40.0);
        let ai = population.genomes[i % population.genomes.len()].clone();

        let color = if i % 2 == 0 {
            Color::rgb(0.2, 0.5, 0.8)
        } else {
            Color::rgb(0.8, 0.2, 0.2)
        };

        commands.spawn((
            Transform::from_translation(Vec3::new(x, 0.5, z)),
            GlobalTransform::default(),
            Tank { team: (i % 2) as u32, ..default() },
            TeamColor(color),
            FireCooldown::default(),
            ai,
        ));
    }

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
    let turret_entity = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(1.0, 0.5, 1.5))),
            material: materials.add(StandardMaterial {
                base_color: color,
                ..default()
            }),
            transform: Transform::from_translation(Vec3::Y * 0.75),
            ..default()
        },
        TankTurret {
            parent_tank: tank_entity,
        },
    )).id();
    
    commands.entity(tank_entity).add_child(turret_entity);
    
    // Ствол как дочерний объект башни
    commands.entity(turret_entity).with_children(|turret_parent| {
        turret_parent.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(0.2, 0.2, 1.0))),
                material: materials.add(StandardMaterial {
                    base_color: color,
                    ..default()
                }),
                transform: Transform::from_translation(Vec3::new(0.0, 0.0, 0.75)),
                ..default()
            },
        ));
    });
    
    tank_entity
}

/// Система проверки окончания боя
pub fn check_battle_end(
    mut battle_state: ResMut<BattleState>,
    tank_query: Query<&Tank>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    battle_state.tick_count += 1;
    battle_state.real_time = battle_state.tick_count as f32 * BASE_SIM_DT as f32;
    
    // Проверяем, есть ли танки разных команд
    let mut teams_alive = std::collections::HashSet::new();
    let total_tanks = tank_query.iter().count();
    
    for tank in tank_query.iter() {
        teams_alive.insert(tank.team);
    }
    
    // Бой закончен если:
    // 1. Осталась только одна команда
    // 2. Такты закончились
    // 3. Все танки уничтожены
    if teams_alive.len() <= 1 || 
       battle_state.tick_count >= battle_state.max_ticks ||
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
