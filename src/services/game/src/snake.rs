use rand::RngExt;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

use super::coordinate::Coordinate;

const COLORS: [&str; 7] = [
    "8",   // red
    "120", // green
    "240", // blue
    "60",  // yellow
    "30",  // orange
    "300", // purple
    "330", // pink
];
const MAX_TURN_ANGLE: f32 = 6.0 * std::f32::consts::PI / 180.0;
const BASE_SNAKE_SIZE: f32 = 15.0;

#[derive(Serialize, Deserialize, Clone)]
pub struct Snake {
    #[serde(rename = "b")]
    pub bodies: VecDeque<Coordinate>, // head, ..., tail
    #[serde(rename = "a")]
    pub acceleration_time_left: u32,
    #[serde(skip)]
    pub speed: f32,
    #[serde(rename = "c")]
    pub color: String,
    #[serde(rename = "v")]
    pub velocity: Coordinate,
    #[serde(skip)]
    pub target_velocity: Coordinate,
    #[serde(rename = "s")]
    pub size: usize,
    #[serde(skip)]
    pub frame_count_offset: u32,
    #[serde(rename = "h")]
    pub is_visible_head: bool, // for rendering
}

impl Snake {
    pub fn new(initial_position: Coordinate, initial_speed: f32) -> Snake {
        let mut bodies = VecDeque::new();
        for _ in 0..10 {
            bodies.push_back(initial_position);
        }
        Snake {
            bodies,
            acceleration_time_left: 0,
            speed: initial_speed,
            size: 15,
            color: COLORS[rand::rng().random_range(0..COLORS.len())].to_string(),
            velocity: Coordinate { x: 0., y: 0. },
            target_velocity: Coordinate { x: 0., y: 0. },
            frame_count_offset: 0,
            is_visible_head: true,
        }
    }

    pub fn get_head(&self) -> &Coordinate {
        &self.bodies[0]
    }

    pub fn get_tail(&self) -> &Coordinate {
        &self.bodies[self.bodies.len() - 1]
    }

    pub fn accelerate(&mut self) {
        if self.bodies.len() < 20 {
            return;
        }
        self.acceleration_time_left = 60;
    }

    pub fn turn_towards_target(&mut self) {
        let target_norm = (self.target_velocity.x.powi(2) + self.target_velocity.y.powi(2)).sqrt();
        if target_norm <= f32::EPSILON || !target_norm.is_finite() {
            return;
        }

        let velocity_norm = (self.velocity.x.powi(2) + self.velocity.y.powi(2)).sqrt();
        if velocity_norm <= f32::EPSILON || !velocity_norm.is_finite() {
            self.velocity = self.target_velocity;
            return;
        }

        let current_angle = self.velocity.y.atan2(self.velocity.x);
        let target_angle = self.target_velocity.y.atan2(self.target_velocity.x);
        let angle_difference = (target_angle - current_angle + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        let size_factor = (BASE_SNAKE_SIZE / self.size as f32).sqrt().clamp(0.6, 1.0);
        let next_angle = current_angle
            + angle_difference.clamp(-MAX_TURN_ANGLE * size_factor, MAX_TURN_ANGLE * size_factor);

        self.velocity = Coordinate {
            x: next_angle.cos(),
            y: next_angle.sin(),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn limits_turning_to_six_degrees_per_frame() {
        let mut snake = Snake::new(Coordinate::default(), 5.0);
        snake.velocity = Coordinate { x: 1.0, y: 0.0 };
        snake.target_velocity = Coordinate { x: 0.0, y: 1.0 };

        snake.turn_towards_target();

        let angle = snake.velocity.y.atan2(snake.velocity.x);
        assert!((angle - MAX_TURN_ANGLE).abs() < 1e-6);
    }

    #[test]
    fn takes_the_short_path_across_the_angle_boundary() {
        let mut snake = Snake::new(Coordinate::default(), 5.0);
        let current_angle = 179.0_f32.to_radians();
        let target_angle = -179.0_f32.to_radians();
        snake.velocity = Coordinate {
            x: current_angle.cos(),
            y: current_angle.sin(),
        };
        snake.target_velocity = Coordinate {
            x: target_angle.cos(),
            y: target_angle.sin(),
        };

        snake.turn_towards_target();

        let angle = snake.velocity.y.atan2(snake.velocity.x);
        let difference = (angle - target_angle + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        assert!(difference.abs() < 1e-5);
    }

    #[test]
    fn larger_snakes_turn_more_slowly() {
        let mut small = Snake::new(Coordinate::default(), 5.0);
        small.velocity = Coordinate { x: 1.0, y: 0.0 };
        small.target_velocity = Coordinate { x: 0.0, y: 1.0 };
        let mut large = small.clone();
        large.size = 40;

        small.turn_towards_target();
        large.turn_towards_target();

        assert!(
            small.velocity.y.atan2(small.velocity.x) > large.velocity.y.atan2(large.velocity.x)
        );
    }
}
