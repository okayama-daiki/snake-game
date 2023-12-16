use serde::Serialize;
use std::collections::VecDeque;

use super::coordinate::Coordinate;

#[derive(Serialize)]
pub struct Snake {
    pub bodies: VecDeque<Coordinate>, // head, ..., tail
    pub speed: f64,
    pub color: String,
    pub velocity: Coordinate,
    pub frame_count_offset: u32,
}

impl Snake {
    pub fn new(initial_position: Coordinate, initial_speed: f64) -> Snake {
        Snake {
            bodies: VecDeque::from([
                initial_position.clone(),
                initial_position.clone(),
                initial_position.clone(),
                initial_position.clone(),
                initial_position.clone(),
            ]),
            speed: initial_speed,
            color: "green".to_string(),
            velocity: Coordinate { x: 1., y: 0. },
            frame_count_offset: 0,
        }
    }

    pub fn clone(&self) -> Snake {
        let mut bodies = VecDeque::new();
        for body in self.bodies.iter() {
            bodies.push_back(body.clone());
        }
        Snake {
            bodies,
            speed: self.speed,
            color: self.color.clone(),
            velocity: self.velocity.clone(),
            frame_count_offset: self.frame_count_offset,
        }
    }

    pub fn get_head(&self) -> &Coordinate {
        &self.bodies[0]
    }
}
