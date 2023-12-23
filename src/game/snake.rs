use num_traits::Float;
use serde::Serialize;
use std::collections::VecDeque;

use super::coordinate::Coordinate;

#[derive(Serialize)]
pub struct Snake<T> {
    pub bodies: VecDeque<Coordinate<T>>, // head, ..., tail
    pub speed: T,
    pub color: String,
    pub velocity: Coordinate<T>,
    pub frame_count_offset: u32,
}

impl<T: Float> Snake<T> {
    pub fn new(initial_position: Coordinate<T>, initial_speed: T) -> Snake<T> {
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
            velocity: Coordinate {
                x: T::one(),
                y: T::zero(),
            },
            frame_count_offset: 0,
        }
    }

    pub fn clone(&self) -> Snake<T> {
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

    pub fn get_head(&self) -> &Coordinate<T> {
        &self.bodies[0]
    }
}
