# Примеры использования и расширения симулятора

## Пример 1: Добавление управляемого игроком танка

Откройте `src/battle.rs` и измените функцию `spawn_initial_tanks`:

```rust
pub fn spawn_initial_tanks(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
) {
    let mut rng = rand::thread_rng();
    
    // Создаем 1 управляемый танк
    let player_tank = spawn_tank(
        commands,
        meshes,
        materials,
        Vec3::new(0.0, 1.0, 0.0),
        0,
        None,
    );
    commands.entity(player_tank).insert(PlayerControlled);
    
    // Создаем 9 AI танков
    for i in 0..9 {
        let x = rng.gen_range(-40.0..40.0);
        let z = rng.gen_range(-40.0..40.0);
        
        let ai = AIController::new_random();
        
        spawn_tank(
            commands,
            meshes,
            materials,
            Vec3::new(x, 1.0, z),
            1, // Все враги в команде 1
            Some(ai),
        );
    }
}
```

## Пример 2: Бой между поколениями

Создайте новую систему для запуска боя между разными поколениями:

```rust
// В src/battle.rs

pub fn setup_cross_generation_battle(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Загружаем сохраненные геномы
    if let Some(old_population) = genetics::load_genomes() {
        let mut rng = rand::thread_rng();
        
        // Берем лучших из старого поколения (команда 0)
        for i in 0..5 {
            if let Some(ai) = old_population.genomes.get(i) {
                let x = rng.gen_range(-40.0..40.0);
                let z = rng.gen_range(-40.0..40.0);
                
                spawn_tank(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    Vec3::new(x, 1.0, z),
                    0,
                    Some(ai.clone()),
                );
            }
        }
        
        // Создаем новых случайных (команда 1)
        for _ in 0..5 {
            let x = rng.gen_range(-40.0..40.0);
            let z = rng.gen_range(-40.0..40.0);
            
            spawn_tank(
                &mut commands,
                &mut meshes,
                &mut materials,
                Vec3::new(x, 1.0, z),
                1,
                Some(AIController::new_random()),
            );
        }
    }
}
```

## Пример 3: Настройка параметров эволюции

В `src/genetics.rs` измените параметры:

```rust
// Более агрессивная мутация
let mutation_rate = 0.2;        // было 0.1
let mutation_strength = 0.5;    // было 0.3

// Больше элиты
let elite_count = self.population_size / 3;  // было 1/5

// Больший размер турнира (более сильная селекция)
let tournament_size = 5;  // было 3
```

## Пример 4: Добавление новых типов танков

Создайте enum для типов танков в `src/components.rs`:

```rust
#[derive(Component)]
pub enum TankType {
    Light,   // Быстрый, слабая броня
    Medium,  // Средний
    Heavy,   // Медленный, сильная броня
}

impl TankType {
    pub fn get_stats(&self) -> (f32, f32, f32) {
        // (скорость, здоровье, урон)
        match self {
            TankType::Light => (8.0, 60.0, 15.0),
            TankType::Medium => (5.0, 100.0, 20.0),
            TankType::Heavy => (3.0, 150.0, 25.0),
        }
    }
}
```

## Пример 5: Визуализация нейронной сети

Добавьте функцию отрисовки нейронной сети в `src/ui.rs`:

```rust
pub fn draw_neural_network(
    mut gizmos: Gizmos,
    selected_tank: Query<&AIController, With<Selected>>,
) {
    if let Ok(ai) = selected_tank.get_single() {
        // Рисуем структуру нейронной сети
        // Входной слой
        for i in 0..8 {
            let pos = Vec3::new(-10.0, i as f32 * 2.0 - 7.0, 0.0);
            gizmos.circle(pos, Vec3::Z, 0.5, Color::GREEN);
        }
        
        // Скрытый слой
        for i in 0..8 {
            let pos = Vec3::new(0.0, i as f32 * 2.0 - 7.0, 0.0);
            gizmos.circle(pos, Vec3::Z, 0.5, Color::BLUE);
        }
        
        // Выходной слой
        for i in 0..4 {
            let pos = Vec3::new(10.0, i as f32 * 3.0 - 4.5, 0.0);
            gizmos.circle(pos, Vec3::Z, 0.5, Color::RED);
        }
    }
}
```

## Пример 6: Ускоренный режим эволюции

Добавьте систему для ускорения времени:

```rust
// В src/main.rs добавьте ресурс
#[derive(Resource)]
struct TimeScale(f32);

// В системах используйте
fn update_with_time_scale(
    time: Res<Time>,
    time_scale: Res<TimeScale>,
) {
    let delta = time.delta_seconds() * time_scale.0;
    // Используйте delta вместо time.delta_seconds()
}

// Управление клавишами
if keyboard.just_pressed(KeyCode::Key1) {
    time_scale.0 = 1.0;  // Обычная скорость
}
if keyboard.just_pressed(KeyCode::Key2) {
    time_scale.0 = 2.0;  // 2x
}
if keyboard.just_pressed(KeyCode::Key3) {
    time_scale.0 = 5.0;  // 5x
}
```

## Пример 7: Сохранение истории эволюции

Добавьте в `src/genetics.rs`:

```rust
#[derive(Serialize, Deserialize)]
struct EvolutionHistory {
    generations: Vec<GenerationStats>,
}

#[derive(Serialize, Deserialize)]
struct GenerationStats {
    generation: u32,
    best_fitness: f32,
    average_fitness: f32,
    worst_fitness: f32,
}

impl Population {
    pub fn save_history(&self, history: &mut EvolutionHistory) {
        let best = self.genomes[0].fitness;
        let worst = self.genomes.last().unwrap().fitness;
        let avg = self.genomes.iter()
            .map(|g| g.fitness)
            .sum::<f32>() / self.genomes.len() as f32;
        
        history.generations.push(GenerationStats {
            generation: self.generation,
            best_fitness: best,
            average_fitness: avg,
            worst_fitness: worst,
        });
        
        // Сохраняем в файл
        if let Ok(json) = serde_json::to_string_pretty(history) {
            std::fs::write("evolution_history.json", json).ok();
        }
    }
}
```

## Пример 8: Различные стратегии поведения

Добавьте предопределенные стратегии в `src/ai.rs`:

```rust
impl AIController {
    pub fn new_aggressive() -> Self {
        // ИИ который всегда атакует
        let mut controller = Self::new_random();
        // Настраиваем веса для агрессивного поведения
        controller
    }
    
    pub fn new_defensive() -> Self {
        // ИИ который держит дистанцию
        let mut controller = Self::new_random();
        // Настраиваем веса для оборонительного поведения
        controller
    }
    
    pub fn new_sniper() -> Self {
        // ИИ снайпер - держится далеко
        let mut controller = Self::new_random();
        controller
    }
}
```

## Запуск различных режимов

### Обычный режим эволюции
```bash
cargo run --release
```

### Режим с вашим участием
Измените код как в Примере 1 и запустите

### Режим наблюдения
Не изменяйте код, просто наблюдайте за эволюцией

## Полезные команды

```bash
# Компиляция с оптимизацией
cargo build --release

# Запуск с логами
RUST_LOG=info cargo run --release

# Проверка кода
cargo check

# Тесты
cargo test

# Форматирование
cargo fmt

# Lint
cargo clippy
```
