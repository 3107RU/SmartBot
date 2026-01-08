use bevy::prelude::*;

/// Нейронная сеть для управления танком
#[allow(dead_code)]
pub struct NeuralNetwork {
    weights_input_hidden: Vec<Vec<f32>>,
    weights_hidden_output: Vec<Vec<f32>>,
}

#[allow(dead_code)]
impl NeuralNetwork {
    pub fn from_genome(genome: &[f32]) -> Self {
        let input_size = 8;
        let hidden_size = 8;
        let output_size = 4;
        
        let mut idx = 0;
        let mut weights_input_hidden = vec![vec![0.0; hidden_size]; input_size];
        let mut weights_hidden_output = vec![vec![0.0; output_size]; hidden_size];
        
        // Заполняем веса из генома
        for i in 0..input_size {
            for j in 0..hidden_size {
                weights_input_hidden[i][j] = genome[idx];
                idx += 1;
            }
        }
        
        for i in 0..hidden_size {
            for j in 0..output_size {
                weights_hidden_output[i][j] = genome[idx];
                idx += 1;
            }
        }
        
        Self {
            weights_input_hidden,
            weights_hidden_output,
        }
    }
    
    pub fn forward(&self, inputs: &[f32; 8]) -> [f32; 4] {
        let hidden_size = self.weights_input_hidden[0].len();
        let mut hidden = vec![0.0; hidden_size];
        
        // Входной слой -> скрытый слой
        for i in 0..hidden_size {
            let mut sum = 0.0;
            for j in 0..inputs.len() {
                sum += inputs[j] * self.weights_input_hidden[j][i];
            }
            hidden[i] = Self::activation(sum);
        }
        
        // Скрытый слой -> выходной слой
        let mut outputs = [0.0; 4];
        for i in 0..4 {
            let mut sum = 0.0;
            for j in 0..hidden_size {
                sum += hidden[j] * self.weights_hidden_output[j][i];
            }
            outputs[i] = Self::activation(sum);
        }
        
        outputs
    }
    
    fn activation(x: f32) -> f32 {
        // Tanh activation
        x.tanh()
    }
    
    /// Получает входные данные для нейронной сети из окружения
    pub fn get_inputs(
        tank_pos: Vec3,
        tank_rotation: f32,
        tank_health: f32,
        nearest_enemy_pos: Option<Vec3>,
        nearest_enemy_health: Option<f32>,
    ) -> [f32; 8] {
        let mut inputs = [0.0; 8];
        
        inputs[0] = tank_health / 100.0; // Нормализованное здоровье
        
        if let Some(enemy_pos) = nearest_enemy_pos {
            let direction = enemy_pos - tank_pos;
            let distance = direction.length();
            
            inputs[1] = (distance / 50.0).min(1.0); // Нормализованная дистанция
            inputs[2] = direction.x / distance; // Направление X
            inputs[3] = direction.z / distance; // Направление Z
            
            let angle_to_enemy = direction.x.atan2(direction.z);
            let angle_diff = angle_to_enemy - tank_rotation;
            inputs[4] = angle_diff.sin();
            inputs[5] = angle_diff.cos();
        }
        
        if let Some(enemy_health) = nearest_enemy_health {
            inputs[6] = enemy_health / 100.0;
        }
        
        inputs[7] = tank_rotation / std::f32::consts::PI; // Нормализованный угол
        
        inputs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_neural_network() {
        let genome = vec![0.5; 96]; // 8*8 + 8*4
        let nn = NeuralNetwork::from_genome(&genome);
        let inputs = [0.0, 0.5, 0.3, -0.2, 0.1, 0.9, 0.7, -0.1];
        let outputs = nn.forward(&inputs);
        
        assert_eq!(outputs.len(), 4);
    }
}
