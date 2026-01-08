use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameState {
    #[default]
    Setup,
    Battle,
    Evolution,
}

/// Компонент танка
#[derive(Component)]
#[allow(dead_code)]
pub struct Tank {
    pub health: f32,
    pub max_health: f32,
    pub speed: f32,
    pub rotation_speed: f32,
    pub generation: u32,
    pub team: u32,
}

impl Default for Tank {
    fn default() -> Self {
        Self {
            health: 100.0,
            max_health: 100.0,
            speed: 5.0,
            rotation_speed: 2.0,
            generation: 0,
            team: 0,
        }
    }
}

/// Компонент снаряда
#[derive(Component)]
pub struct Projectile {
    pub damage: f32,
    pub speed: f32,
    pub lifetime: Timer,
    pub owner: Entity,
}

/// Перезарядка орудия танка
#[derive(Component)]
pub struct FireCooldown {
    pub timer: Timer,
}

impl Default for FireCooldown {
    fn default() -> Self {
        let mut timer = Timer::from_seconds(0.7, TimerMode::Once);
        timer.set_elapsed(timer.duration()); // сразу готов к выстрелу
        Self { timer }
    }
}

/// Компонент управления ИИ
#[derive(Component, Clone, Serialize, Deserialize)]
pub struct AIController {
    pub genome: Vec<f32>, // Веса нейронной сети
    pub fitness: f32,
    pub kills: u32,
    pub survival_time: f32,
}

impl AIController {
    pub fn new_random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        // Простая нейронная сеть: 8 входов -> 8 скрытых -> 4 выхода
        // Входы: расстояние до ближайшего врага, угол, здоровье свое/врага, позиция и т.д.
        // Выходы: движение вперед/назад, поворот влево/вправо
        let genome_size = (8 * 8) + (8 * 4); // веса связей
        let genome: Vec<f32> = (0..genome_size)
            .map(|_| rng.gen_range(-1.0..1.0))
            .collect();
        
        Self {
            genome,
            fitness: 0.0,
            kills: 0,
            survival_time: 0.0,
        }
    }
}

/// Компонент игрока
#[derive(Component)]
pub struct PlayerControlled;

/// Маркер башни танка
#[derive(Component)]
#[allow(dead_code)]
pub struct TankTurret {
    pub parent_tank: Entity,
}

/// Компонент выбора танка для просмотра от 3-го лица
#[derive(Component)]
pub struct Selected;

/// Цвет команды
#[derive(Component)]
#[allow(dead_code)]
pub struct TeamColor(pub Color);
