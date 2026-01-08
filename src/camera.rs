use bevy::prelude::*;
use bevy_egui::EguiContexts;
use crate::components::*;

#[derive(Resource, Default)]
pub struct CameraState {
    pub mode: CameraMode,
    pub selected_tank: Option<Entity>,
}

#[derive(Default, PartialEq)]
pub enum CameraMode {
    #[default]
    TopDown,      // Вид сверху (по умолчанию)
    ThirdPerson,  // Вид от третьего лица
}

#[derive(Component)]
pub struct MainCamera;

#[derive(Component)]
#[allow(dead_code)]
pub struct ThirdPersonCamera;

/// Система управления камерой
pub fn camera_control_system(
    keyboard: Res<Input<KeyCode>>,
    mut camera_state: ResMut<CameraState>,
    mut camera_query: Query<&mut Transform, With<MainCamera>>,
    tank_query: Query<&Transform, (With<Tank>, Without<MainCamera>)>,
) {
    if let Ok(mut camera_transform) = camera_query.get_single_mut() {
        match camera_state.mode {
            CameraMode::TopDown => {
                // Управление камерой в режиме вида сверху
                let speed = 30.0;
                
                if keyboard.pressed(KeyCode::Up) {
                    camera_transform.translation.z -= speed * 0.016;
                }
                if keyboard.pressed(KeyCode::Down) {
                    camera_transform.translation.z += speed * 0.016;
                }
                if keyboard.pressed(KeyCode::Left) {
                    camera_transform.translation.x -= speed * 0.016;
                }
                if keyboard.pressed(KeyCode::Right) {
                    camera_transform.translation.x += speed * 0.016;
                }
                
                // Zoom
                if keyboard.pressed(KeyCode::Equals) || keyboard.pressed(KeyCode::NumpadAdd) {
                    camera_transform.translation.y -= speed * 0.016;
                }
                if keyboard.pressed(KeyCode::Minus) || keyboard.pressed(KeyCode::NumpadSubtract) {
                    camera_transform.translation.y += speed * 0.016;
                }
                
                camera_transform.translation.y = camera_transform.translation.y.clamp(10.0, 100.0);
            }
            CameraMode::ThirdPerson => {
                // В режиме третьего лица следуем за выбранным танком
                if let Some(tank_entity) = camera_state.selected_tank {
                    if let Ok(tank_transform) = tank_query.get(tank_entity) {
                        let offset = Vec3::new(0.0, 5.0, -10.0);
                        let target_pos = tank_transform.translation + 
                                       tank_transform.rotation * offset;
                        
                        camera_transform.translation = target_pos;
                        camera_transform.look_at(
                            tank_transform.translation + Vec3::Y * 2.0,
                            Vec3::Y
                        );
                    }
                }
            }
        }
        
        // Переключение режима камеры на Tab
        if keyboard.just_pressed(KeyCode::Tab) {
            camera_state.mode = match camera_state.mode {
                CameraMode::TopDown => CameraMode::ThirdPerson,
                CameraMode::ThirdPerson => CameraMode::TopDown,
            };
            
            // При переключении на вид сверху сбрасываем камеру
            if camera_state.mode == CameraMode::TopDown {
                camera_transform.translation = Vec3::new(0.0, 50.0, 0.1);
                camera_transform.look_at(Vec3::ZERO, Vec3::Y);
            }
        }
    }
}

/// Система выбора танка мышью
pub fn tank_selection_system(
    mut contexts: EguiContexts,
    mouse_button: Res<Input<MouseButton>>,
    windows: Query<&Window>,
    camera_query: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    tank_query: Query<(Entity, &Transform), With<Tank>>,
    mut camera_state: ResMut<CameraState>,
    mut commands: Commands,
) {
    let ctx = contexts.ctx_mut();
    if ctx.wants_pointer_input() || ctx.is_pointer_over_area() || ctx.wants_keyboard_input() {
        return;
    }

    // Клик мышью для выбора танка
    if mouse_button.just_pressed(MouseButton::Left) {
        let window = windows.single();
        
        if let Some(_cursor_position) = window.cursor_position() {
            if let Ok((_camera, _camera_transform)) = camera_query.get_single() {
                // Простая проверка выбора танка
                // В реальном приложении нужен ray casting
                
                // Временно выбираем ближайший танк
                if let Some((entity, _)) = tank_query.iter().next() {
                    camera_state.selected_tank = Some(entity);
                    camera_state.mode = CameraMode::ThirdPerson;
                    
                    // Добавляем маркер выбора
                    commands.entity(entity).insert(Selected);
                }
            }
        }
    }
}
