mod components;
mod systems;
mod ai;
mod genetics;
mod battle;
mod map;
mod camera;
mod ui;

use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use systems::*;
use camera::*;
use ui::*;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    #[default]
    Setup,
    Battle,
    Evolution,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tank Battle Simulator".to_string(),
                resolution: (1280.0, 720.0).into(),
                ..default()
            }),
            ..default()
        })).add_plugins(EguiPlugin)
        .insert_resource(TimeMultiplier::default())
        .insert_resource(TimeMultiplierUiState::default())
        .add_state::<GameState>()
        .add_systems(Startup, (setup, ui::setup_ui))
        .add_systems(Update, (
            tank_movement_system,
            tank_shooting_system,
            projectile_movement_system,
            collision_system,
            ai_control_system,
            player_control_system,
            health_display_system,
            battle::check_battle_end,
        ).run_if(in_state(GameState::Battle)))
        .add_systems(Update, (
            time_multiplier_input_system,
            time_multiplier_ui_system,
            camera_control_system,
            tank_selection_system.after(time_multiplier_ui_system),
            ui_system,
            update_stats_ui,
        ))
        .add_systems(OnEnter(GameState::Battle), battle::start_battle)
        .add_systems(OnExit(GameState::Battle), battle::end_battle)
        .add_systems(OnEnter(GameState::Evolution), genetics::evolve_population)
        .run();
    }
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    // Свет
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 2000.0,
            shadows_enabled: true,
            range: 200.0,
            ..default()
        },
        transform: Transform::from_xyz(20.0, 40.0, 20.0),
        ..default()
    });
    
    // Дополнительное освещение
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 5000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ,
            -std::f32::consts::FRAC_PI_4,
            std::f32::consts::FRAC_PI_4,
            0.0,
        )),
        ..default()
    });

    // Главная камера (вид сверху)
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 50.0, 0.1)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        camera::MainCamera,
    ));

    // Инициализация ресурсов
    commands.insert_resource(battle::BattleState::default());
    commands.insert_resource(genetics::Population::new(20));
    commands.insert_resource(camera::CameraState::default());
    
    // Генерируем карту
    let game_map = map::GameMap::new(100.0);
    game_map.spawn(&mut commands, &mut meshes, &mut materials);
    
    // Создаем начальную популяцию танков
    battle::spawn_initial_tanks(&mut commands, &mut meshes, &mut materials);
    
    // Начинаем битву
    next_state.set(GameState::Battle);
}
