use bevy::prelude::*;
use bevy::prelude::shape;
use bevy_egui::{egui, EguiContexts};
use crate::components::*;
use crate::ai::*;
use crate::map::ObstacleComponent;
use std::time::Duration;

/// Глобальный множитель скорости симуляции
#[derive(Resource, Debug, Clone, Copy)]
pub struct TimeMultiplier {
    pub scale: f32,
}

impl Default for TimeMultiplier {
    fn default() -> Self {
        Self { scale: 1.0 }
    }
}

impl TimeMultiplier {
    pub fn scaled_seconds(&self, time: &Time) -> f32 {
        time.delta_seconds() * self.scale
    }

    pub fn scaled_delta(&self, time: &Time) -> Duration {
        time.delta().mul_f32(self.scale)
    }
}

/// Состояние видимости UI множителя времени
#[derive(Resource, Debug, Clone, Copy)]
pub struct TimeMultiplierUiState {
    pub visible: bool,
}

impl Default for TimeMultiplierUiState {
    fn default() -> Self {
        Self { visible: false }
    }
}

/// Управление множителем времени: [ и ] — замедление/ускорение, \ — сброс, F1 — показать/скрыть UI
pub fn time_multiplier_input_system(
    keys: Res<Input<KeyCode>>,
    mut multiplier: ResMut<TimeMultiplier>,
    mut ui_state: ResMut<TimeMultiplierUiState>,
) {
    let mut new_scale = multiplier.scale;

    if keys.just_pressed(KeyCode::F1) {
        ui_state.visible = !ui_state.visible;
    }

    if keys.just_pressed(KeyCode::BracketLeft) {
        new_scale -= 0.25;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        new_scale += 0.25;
    }
    if keys.just_pressed(KeyCode::Backslash) {
        new_scale = 1.0;
    }

    new_scale = new_scale.clamp(0.1, 1000.0);

    if (new_scale - multiplier.scale).abs() > f32::EPSILON {
        multiplier.scale = new_scale;
        info!("Множитель времени: {:.2}x", multiplier.scale);
    }
}

/// UI слайдер для управления скоростью времени
pub fn time_multiplier_ui_system(
    mut contexts: EguiContexts,
    mut multiplier: ResMut<TimeMultiplier>,
    ui_state: Res<TimeMultiplierUiState>,
) {
    if !ui_state.visible {
        return;
    }

    let ctx = contexts.ctx_mut();
    egui::Window::new("Скорость времени")
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-8.0, 8.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Скорость симуляции (0.1x - 1000x)");
        let mut scale = multiplier.scale;
        if ui.add(
                egui::Slider::new(&mut scale, 0.1..=1000.0)
                .logarithmic(true)
                .text("x"),
        ).changed() {
            multiplier.scale = scale;
        }
        if ui.button("Сбросить до 1x").clicked() {
            multiplier.scale = 1.0;
        }
    });

    egui::Window::new("Справка по клавишам")
        .anchor(egui::Align2::RIGHT_TOP, egui::Vec2::new(-8.0, 170.0))
        .resizable(false)
        .show(ctx, |ui| {
            ui.label("Управление:");
            ui.label("WASD — движение танка");
            ui.label("Space — выстрел");
            ui.label("Tab — смена вида камеры");
            ui.label("Стрелки — движение камеры");
            ui.label("+/- — zoom камеры");
            ui.label("ЛКМ по танку — выбрать для вида 3-го лица");
            ui.separator();
            ui.label("[ / ] — замедлить / ускорить время");
            ui.label("\\ — сбросить скорость времени");
            ui.label("F1 — показать/скрыть этот UI");
        });
}

/// Система движения танков
pub fn tank_movement_system(
    _time: Res<Time>,
    mut query: Query<(&Tank, &mut Transform), Without<Projectile>>,
) {
    for (_tank, _transform) in query.iter_mut() {
        // Движение обрабатывается в системах AI или игрока
    }
}

/// Система стрельбы танков
pub fn tank_shooting_system(
    _commands: Commands,
    _meshes: ResMut<Assets<Mesh>>,
    _materials: ResMut<Assets<StandardMaterial>>,
    _query: Query<(Entity, &Transform, &Tank)>,
) {
    // Логика стрельбы добавляется через события
}

/// Система движения снарядов
pub fn projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    time_multiplier: Res<TimeMultiplier>,
    mut query: Query<(Entity, &mut Transform, &mut Projectile)>,
) {
    for (entity, mut transform, mut projectile) in query.iter_mut() {
        let dt = time_multiplier.scaled_seconds(&time);
        let scaled_delta = time_multiplier.scaled_delta(&time);
        // Двигаем снаряд вперед
        let forward = transform.back();
        transform.translation += forward * projectile.speed * dt;
        
        // Обновляем таймер жизни
        projectile.lifetime.tick(scaled_delta);
        if projectile.lifetime.finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Система обнаружения столкновений
pub fn collision_system(
    mut commands: Commands,
    projectile_query: Query<(Entity, &Transform, &Projectile)>,
    mut tank_query: Query<(Entity, &Transform, &mut Tank), Without<Projectile>>,
    obstacle_query: Query<&Transform, With<ObstacleComponent>>,
    mut ai_query: Query<&mut AIController>,
) {
    for (proj_entity, proj_transform, projectile) in projectile_query.iter() {
        for (tank_entity, tank_transform, mut tank) in tank_query.iter_mut() {
            // Не попадаем в себя
            if tank_entity == projectile.owner {
                continue;
            }
            
            let distance = proj_transform.translation.distance(tank_transform.translation);
            if distance < 1.5 { // Радиус попадания
                tank.health -= projectile.damage;
                commands.entity(proj_entity).despawn();
                
                if tank.health <= 0.0 {
                    // Увеличиваем счётчик убийств у владельца снаряда
                    if let Ok(mut ai) = ai_query.get_mut(projectile.owner) {
                        ai.kills += 1;
                    }
                    commands.entity(tank_entity).despawn_recursive();
                }
                break;
            }
        }
        
        // Проверяем столкновение с препятствиями
        for obstacle_transform in obstacle_query.iter() {
            let distance = proj_transform.translation.distance(obstacle_transform.translation);
            if distance < 5.0 { // Увеличенный радиус для стен и препятствий
                commands.entity(proj_entity).despawn();
                break;
            }
        }
    }
}

/// Система управления танком игроком
pub fn player_control_system(
    keyboard: Res<Input<KeyCode>>,
    time: Res<Time>,
    time_multiplier: Res<TimeMultiplier>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut query: Query<(Entity, &mut Transform, &Tank, &mut FireCooldown, &TeamColor), With<PlayerControlled>>,
    all_tanks: Query<(&Transform, &Tank), Without<PlayerControlled>>,
    obstacle_query: Query<&Transform, (With<ObstacleComponent>, Without<Tank>)>,
) {
    for (entity, mut transform, tank, mut cooldown, team_color) in query.iter_mut() {
        let mut direction = Vec3::ZERO;
        let mut rotation = 0.0;
        let dt = time_multiplier.scaled_seconds(&time);
        let scaled_delta = time_multiplier.scaled_delta(&time);
        cooldown.timer.tick(scaled_delta);
        
        if keyboard.pressed(KeyCode::W) {
            direction += transform.forward();
        }
        if keyboard.pressed(KeyCode::S) {
            direction -= transform.forward();
        }
        if keyboard.pressed(KeyCode::A) {
            rotation += tank.rotation_speed;
        }
        if keyboard.pressed(KeyCode::D) {
            rotation -= tank.rotation_speed;
        }
        
        transform.translation += direction * tank.speed * dt;
        
        // Проверяем столкновение с препятствиями
        let mut collided = false;
        for obstacle_transform in obstacle_query.iter() {
            let distance = transform.translation.distance(obstacle_transform.translation);
            if distance < 6.0 {
                collided = true;
                break;
            }
        }
        
        // Проверяем столкновение с другими танками
        if !collided {
            for (other_transform, _other_tank) in all_tanks.iter() {
                let distance = transform.translation.distance(other_transform.translation);
                if distance < 3.0 {
                    collided = true;
                    break;
                }
            }
        }
        
        // Если столкновение, откатываем движение
        if collided {
            transform.translation -= direction * tank.speed * dt;
        }
        
        transform.rotate_y(rotation * dt);
        
        // Фиксируем высоту танка на поверхности
        transform.translation.y = 0.5;
        
        // Стрельба на пробел с кулдауном
        if keyboard.just_pressed(KeyCode::Space) && cooldown.timer.finished() {
            spawn_projectile(&mut commands, &mut meshes, &mut materials, entity, &transform, team_color.0);
            cooldown.timer.reset();
        }
    }
}

/// Система управления танком через ИИ
pub fn ai_control_system(
    time: Res<Time>,
    time_multiplier: Res<TimeMultiplier>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ai_tanks: Query<(Entity, &mut Transform, &Tank, &mut AIController, &mut FireCooldown, &TeamColor), Without<PlayerControlled>>,
    all_tanks: Query<(&Transform, &Tank), Without<AIController>>,
    obstacle_query: Query<&Transform, (With<ObstacleComponent>, Without<Tank>)>,
) {
    info!("AI control running, tanks: {}", ai_tanks.iter().count());
    for (entity, mut transform, tank, mut ai, mut cooldown, team_color) in ai_tanks.iter_mut() {
        let dt = time_multiplier.scaled_seconds(&time);
        let scaled_delta = time_multiplier.scaled_delta(&time);

        ai.survival_time += dt;
        cooldown.timer.tick(scaled_delta);
        
        // Находим ближайшего врага
        let mut nearest_enemy_pos = None;
        let mut nearest_enemy_health = None;
        let mut min_distance = f32::MAX;
        
        for (other_transform, other_tank) in all_tanks.iter() {
            if other_tank.team != tank.team {
                let distance = transform.translation.distance(other_transform.translation);
                if distance < min_distance {
                    min_distance = distance;
                    nearest_enemy_pos = Some(other_transform.translation);
                    nearest_enemy_health = Some(other_tank.health);
                }
            }
        }
        
        // Получаем входные данные для нейронной сети
        let current_angle = transform.rotation.to_euler(EulerRot::YXZ).0;
        let inputs = NeuralNetwork::get_inputs(
            transform.translation,
            current_angle,
            tank.health,
            nearest_enemy_pos,
            nearest_enemy_health,
        );
        
        // Создаем нейронную сеть и получаем выходы
        let nn = NeuralNetwork::from_genome(&ai.genome);
        let outputs = nn.forward(&inputs);
        
        // Интерпретируем выходы: [0] - движение вперед/назад (0..1 -> -1..1), [1] - поворот (0..1 -> -1..1)
        let move_input = (outputs[0] - 0.5) * 2.0;
        let turn_input = (outputs[1] - 0.5) * 2.0;
        
        // Применяем движение
        let forward = transform.forward();
        let old_pos = transform.translation;
        transform.translation += forward * move_input * tank.speed * dt;
        
        // Проверяем столкновение с препятствиями
        let mut collided = false;
        for obstacle_transform in obstacle_query.iter() {
            let distance = transform.translation.distance(obstacle_transform.translation);
            if distance < 6.0 { // Увеличенный радиус
                collided = true;
                break;
            }
        }
        
        // Проверяем столкновение с другими танками
        if !collided {
            for (other_transform, _other_tank) in all_tanks.iter() {
                let distance = transform.translation.distance(other_transform.translation);
                if distance < 3.0 { // Радиус двух танков
                    collided = true;
                    break;
                }
            }
        }
        
        // Если столкновение, откатываем движение
        if collided {
            transform.translation = old_pos;
        }
        
        // Применяем поворот
        transform.rotate_y(turn_input * tank.rotation_speed * dt);
        
        // Фиксируем высоту танка на поверхности
        transform.translation.y = 0.5;
        
        // Логируем высоту для отладки
        if entity.index() == 0 {
            info!("AI Tank 0 y: {}, obstacles: {}", transform.translation.y, obstacle_query.iter().count());
        }
        
        // Стрельба: если outputs[2] > 0.5 и cooldown готов
        if outputs[2] > 0.5 && cooldown.timer.finished() {
            spawn_projectile(&mut commands, &mut meshes, &mut materials, entity, &transform, team_color.0);
            cooldown.timer.reset();
        }
    }
}

/// Вспомогательная функция для создания снаряда
fn spawn_projectile(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    owner: Entity,
    transform: &Transform,
    color: Color,
) {
    // Позиция конца ствола: turret center + barrel offset
    let turret_offset = Vec3::Y * 0.75;
    let barrel_end_offset = Vec3::new(0.0, 0.0, 1.25); // конец ствола от turret center
    let barrel_end_pos = transform.translation + transform.rotation * (turret_offset + barrel_end_offset);
    
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::UVSphere { radius: 0.3, ..default() })),
            material: materials.add(StandardMaterial {
                base_color: color,
                ..default()
            }),
            transform: Transform::from_translation(barrel_end_pos).with_rotation(transform.rotation),
            ..default()
        },
        Projectile {
            damage: 20.0,
            speed: 30.0,
            lifetime: Timer::from_seconds(3.0, TimerMode::Once),
            owner,
        },
    ));
}

/// Система отображения здоровья
pub fn health_display_system(
    _query: Query<(&Tank, &Transform)>,
) {
    // Позже добавим UI для отображения здоровья
}
