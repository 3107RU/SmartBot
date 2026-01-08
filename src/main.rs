mod components;
mod systems;
mod ai;
mod genetics;
mod battle;
mod map;
mod camera;
mod ui;

use bevy::prelude::*;
use bevy::time::{Fixed, TimeUpdateStrategy};
use bevy::window::PresentMode;
use bevy::app::ScheduleRunnerPlugin;
use bevy_egui::EguiPlugin;
use systems::*;
use camera::*;
use ui::*;
use crate::components::GameState;
use std::time::Duration;
use battle::BattleState;
use camera::CameraState;

const BASE_MANUAL_DT: f64 = BASE_SIM_DT;

#[derive(Resource, Clone, Copy, Debug, Default)]
pub struct Headless(pub bool);

fn is_headless(headless: Res<Headless>) -> bool {
    headless.0
}

fn main() {
    let fresh_start = std::env::args().any(|arg| arg == "--fresh");
    let headless = std::env::args().any(|arg| arg == "--headless");
    let population = if fresh_start {
        genetics::Population::new_fresh(20)
    } else {
        genetics::Population::new(20)
    };
    
    println!("Population initialized: generation {}, fresh_start: {}", population.generation, fresh_start);

    let mut app = App::new();
    app.insert_resource(Headless(headless));

    if headless {
        app.add_plugins(MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(Duration::ZERO)));
    } else {
        app.add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tank Battle Simulator".to_string(),
                resolution: (1280.0, 720.0).into(),
                present_mode: PresentMode::Immediate,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EguiPlugin);
    }

    app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f64(BASE_MANUAL_DT)))
        .insert_resource(population)
        .insert_resource(TimeMultiplier::default())
        .insert_resource(TimeMultiplierUiState::default())
        .insert_resource(crate::ui::StartupChoiceMade::default())
        .insert_resource(crate::ui::StartupTimer(Timer::from_seconds(5.0, TimerMode::Once)))
        .insert_resource(ProgressLog::default())
        .add_state::<GameState>()
        .add_systems(Startup, (setup, load_ui_assets).run_if(not_headless))
        .add_systems(Startup, headless_setup.run_if(is_headless))
        .add_systems(Startup, init_fixed_timestep)
        .add_systems(First, refresh_time_update_strategy)
        .add_systems(FixedUpdate, (
            tank_movement_system,
            tank_shooting_system.run_if(not_headless),
            projectile_movement_system,
            collision_system,
            ai_control_system,
            player_control_system.run_if(not_headless),
            health_display_system,
            battle::check_battle_end,
        ).run_if(in_state(GameState::Battle)))
        .add_systems(Update, (
            time_multiplier_input_system.run_if(not_headless),
            time_multiplier_ui_system.run_if(not_headless),
            camera_control_system.run_if(not_headless),
            tank_selection_system.after(time_multiplier_ui_system).run_if(not_headless),
            ui_system.run_if(not_headless),
            update_stats_ui.run_if(not_headless),
        ))
        .add_systems(Update, log_progress)
        .add_systems(Update, setup_ui_system.run_if(in_state(GameState::Setup)).run_if(not_headless))
        .add_systems(OnEnter(GameState::Battle), (battle::start_battle, battle::spawn_tanks_from_population).run_if(not_headless))
        .add_systems(OnEnter(GameState::Battle), battle::spawn_tanks_headless.run_if(is_headless))
        .add_systems(OnEnter(GameState::Battle), create_stats_ui.run_if(not_headless))
        .add_systems(OnExit(GameState::Battle), battle::end_battle)
        .add_systems(OnExit(GameState::Battle), despawn_stats_ui.run_if(not_headless))
        .add_systems(OnEnter(GameState::Evolution), genetics::evolve_population);

    {
        // Настраиваем редкую отрисовку: RenderApp обновляет фазы только на выбранных кадрах
        // Рендеринг оставлен без пропуска кадров, чтобы избежать падений в Bevy 0.12
    }

    app.run();
    }
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    _next_state: ResMut<NextState<GameState>>,
) {
    info!("Setup started");
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
            transform: Transform::from_xyz(0.0, 150.0, 0.1)
                .looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        camera::MainCamera,
    ));

    // Инициализация ресурсов
    commands.insert_resource(battle::BattleState::default());
    commands.insert_resource(camera::CameraState::default());
    
    // Генерируем карту
    let game_map = map::GameMap::new(100.0);
    game_map.spawn(&mut commands, &mut meshes, &mut materials);
    
    // Создаем начальную популяцию танков
    // battle::spawn_initial_tanks(&mut commands, &mut meshes, &mut materials);
    
    // Начинаем битву
    // next_state.set(GameState::Battle);
}

fn headless_setup(
    mut commands: Commands,
    mut next_state: ResMut<NextState<GameState>>,
) {
    commands.insert_resource(BattleState::default());
    commands.insert_resource(CameraState::default());
    next_state.set(GameState::Battle);
}

fn init_fixed_timestep(
    mut time_fixed: ResMut<Time<Fixed>>,
) {
    time_fixed.set_timestep_hz(BASE_SIM_HZ);
}
