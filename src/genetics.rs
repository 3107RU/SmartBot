use bevy::prelude::*;
use crate::components::*;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Resource, Serialize, Deserialize)]
pub struct Population {
    pub generation: u32,
    pub population_size: usize,
    pub genomes: Vec<AIController>,
    pub best_genome: Option<AIController>,
}

impl Population {
    pub fn new(size: usize) -> Self {
        if let Some(loaded) = load_genomes() {
            println!("Загружены лучшие геномы из предыдущего запуска, поколение {}", loaded.generation);
            loaded
        } else {
            println!("Создание новой популяции случайных геномов");
            Self::new_fresh(size)
        }
    }
    
    pub fn new_fresh(size: usize) -> Self {
        let genomes = (0..size)
            .map(|_| AIController::new_random())
            .collect();
        
        Self {
            generation: 0,
            population_size: size,
            genomes,
            best_genome: None,
        }
    }
    
    /// Сортирует популяцию по приспособленности
    pub fn sort_by_fitness(&mut self) {
        self.genomes.sort_by(|a, b| {
            b.fitness.partial_cmp(&a.fitness).unwrap()
        });
    }
    
    /// Выполняет селекцию, скрещивание и мутацию
    pub fn evolve(&mut self) {
        self.sort_by_fitness();
        self.best_genome = self.genomes.first().cloned();
        
        info!("Поколение {}: Лучший фитнес = {:.2}", 
              self.generation, 
              self.genomes[0].fitness);
        
        // Оставляем лучшие 20% (элитизм)
        let elite_count = self.population_size / 5;
        let mut new_genomes = Vec::new();
        
        // Копируем элиту
        for i in 0..elite_count {
            let mut elite = self.genomes[i].clone();
            elite.fitness = 0.0;
            elite.kills = 0;
            elite.survival_time = 0.0;
            new_genomes.push(elite);
        }
        
        // Восстанавливаем лучший геном с правильным фитнесом
        if let Some(best) = &self.best_genome {
            new_genomes[0] = best.clone();
        }
        
        // Создаем остальных через скрещивание и мутацию
        while new_genomes.len() < self.population_size {
            let parent1 = self.select_parent();
            let parent2 = self.select_parent();
            
            let mut child = self.crossover(&parent1, &parent2);
            self.mutate(&mut child);
            
            new_genomes.push(child);
        }
        
        self.genomes = new_genomes;
        self.generation += 1;
    }
    
    /// Выбор родителя с помощью турнирной селекции
    fn select_parent(&self) -> AIController {
        let mut rng = rand::thread_rng();
        let tournament_size = 3;
        
        let mut best = self.genomes[rng.gen_range(0..self.genomes.len())].clone();
        
        for _ in 1..tournament_size {
            let competitor = &self.genomes[rng.gen_range(0..self.genomes.len())];
            if competitor.fitness > best.fitness {
                best = competitor.clone();
            }
        }
        
        best
    }
    
    /// Одноточечное скрещивание
    fn crossover(&self, parent1: &AIController, parent2: &AIController) -> AIController {
        let mut rng = rand::thread_rng();
        let crossover_point = rng.gen_range(0..parent1.genome.len());
        
        let mut child_genome = Vec::new();
        
        for i in 0..parent1.genome.len() {
            if i < crossover_point {
                child_genome.push(parent1.genome[i]);
            } else {
                child_genome.push(parent2.genome[i]);
            }
        }
        
        AIController {
            genome: child_genome,
            fitness: 0.0,
            kills: 0,
            survival_time: 0.0,
        }
    }
    
    /// Мутация генома
    fn mutate(&self, individual: &mut AIController) {
        let mut rng = rand::thread_rng();
        let mutation_rate = 0.1;
        let mutation_strength = 0.3;
        
        for gene in individual.genome.iter_mut() {
            if rng.gen::<f32>() < mutation_rate {
                *gene += rng.gen_range(-mutation_strength..mutation_strength);
                *gene = gene.clamp(-1.0, 1.0);
            }
        }
    }
    
    /// Вычисляет фитнес для индивида
    #[allow(dead_code)]
    pub fn calculate_fitness(&mut self, _entity: Entity, ai: &AIController) {
        // Фитнес = kills * 100 + survival_time * 2 - deaths * 50
        let fitness = (ai.kills as f32 * 100.0) + (ai.survival_time * 2.0);
        
        // Обновляем фитнес в популяции
        if let Some(genome) = self.genomes.iter_mut()
            .find(|g| g.genome == ai.genome) {
            genome.fitness = fitness;
            genome.kills = ai.kills;
            genome.survival_time = ai.survival_time;
        }
    }
}

/// Система эволюции популяции
pub fn evolve_population(
    mut population: ResMut<Population>,
    mut next_state: ResMut<NextState<crate::GameState>>,
) {
    population.evolve();
    
    // Сохраняем лучшие геномы
    save_best_genomes(&population);
    
    info!("Эволюция завершена, поколение {}", population.generation);
    
    // Начинаем следующий матч
    next_state.set(crate::GameState::Battle);
}

/// Сохранение лучших геномов в файл
fn save_best_genomes(population: &Population) {
    if let Ok(json) = serde_json::to_string_pretty(population) {
        std::fs::write("best_genomes.json", json).ok();
    }
}

/// Загрузка геномов из файла
pub fn load_genomes() -> Option<Population> {
    match std::fs::read_to_string("best_genomes.json") {
        Ok(json) => {
            match serde_json::from_str::<Population>(&json) {
                Ok(pop) => {
                    println!("Successfully loaded population with generation {}", pop.generation);
                    Some(pop)
                },
                Err(e) => {
                    println!("Failed to parse JSON: {}", e);
                    None
                }
            }
        },
        Err(e) => {
            println!("Failed to read best_genomes.json: {}", e);
            None
        }
    }
}
