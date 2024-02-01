use rand::Rng;
use serde::Serialize;
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

#[derive(Serialize, Clone)]
pub struct Snake {
    pub bodies: VecDeque<Coordinate>, // head, ..., tail
    pub acceleration_time_left: u32,
    pub speed: f32,
    pub color: String,
    pub velocity: Coordinate,
    pub size: usize,
    pub frame_count_offset: u32,
    pub is_visible_head: bool, // for rendering
}

impl Snake {
    pub fn new(initial_position: Coordinate, initial_speed: f32) -> Snake {
        let mut bodies = VecDeque::new();
        for _ in 0..10 {
            bodies.push_back(initial_position.clone());
        }
        Snake {
            bodies,
            acceleration_time_left: 0,
            speed: initial_speed,
            size: 15,
            color: COLORS[rand::thread_rng().gen_range(0..COLORS.len())].to_string(),
            velocity: Coordinate { x: 0., y: 0. },
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
}
