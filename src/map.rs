use bevy::prelude::*;
use bevy::prelude::shape;
use rand::Rng;

/// Генерация игровой карты
pub struct GameMap {
    pub size: f32,
    pub obstacles: Vec<Obstacle>,
}

pub struct Obstacle {
    pub position: Vec3,
    pub size: Vec3,
}

impl Default for GameMap {
    fn default() -> Self {
        Self::new(100.0)
    }
}

impl GameMap {
    pub fn new(size: f32) -> Self {
        let mut obstacles = Vec::new();
        let mut rng = rand::thread_rng();
        
        // Создаем случайные препятствия
        for _ in 0..15 {
            let x = rng.gen_range(-size/2.0..size/2.0);
            let z = rng.gen_range(-size/2.0..size/2.0);
            let width = rng.gen_range(2.0..6.0);
            let height = rng.gen_range(2.0..8.0);
            let depth = rng.gen_range(2.0..6.0);
            
            obstacles.push(Obstacle {
                position: Vec3::new(x, height / 2.0, z),
                size: Vec3::new(width, height, depth),
            });
        }
        
        Self { size, obstacles }
    }
    
    /// Создает 3D-объекты для карты
    pub fn spawn(
        &self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        materials: &mut ResMut<Assets<StandardMaterial>>,
    ) {
        // Земля
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: self.size,
                subdivisions: 0,
            })),
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(0.3, 0.5, 0.3),
                ..default()
            }),
            ..default()
        });
        
        // Препятствия
        let obstacle_material = materials.add(StandardMaterial {
            base_color: Color::rgb(0.5, 0.5, 0.5),
            ..default()
        });
        
        for obstacle in &self.obstacles {
            commands.spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box::new(
                    obstacle.size.x,
                    obstacle.size.y,
                    obstacle.size.z,
                ))),
                material: obstacle_material.clone(),
                transform: Transform::from_translation(obstacle.position),
                ..default()
            });
        }
        
        // Границы карты
        let wall_material = materials.add(StandardMaterial {
            base_color: Color::rgba(0.8, 0.8, 0.8, 0.3),
            ..default()
        });
        let wall_height = 5.0;
        let wall_thickness = 1.0;
        
        // Северная стена
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(self.size, wall_height, wall_thickness))),
            material: wall_material.clone(),
            transform: Transform::from_xyz(0.0, wall_height / 2.0, self.size / 2.0),
            ..default()
        });
        
        // Южная стена
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(self.size, wall_height, wall_thickness))),
            material: wall_material.clone(),
            transform: Transform::from_xyz(0.0, wall_height / 2.0, -self.size / 2.0),
            ..default()
        });
        
        // Западная стена
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(wall_thickness, wall_height, self.size))),
            material: wall_material.clone(),
            transform: Transform::from_xyz(-self.size / 2.0, wall_height / 2.0, 0.0),
            ..default()
        });
        
        // Восточная стена
        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(wall_thickness, wall_height, self.size))),
            material: wall_material,
            transform: Transform::from_xyz(self.size / 2.0, wall_height / 2.0, 0.0),
            ..default()
        });
    }
}
