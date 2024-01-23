use num_traits::Float;
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

#[derive(Serialize)]
pub struct Snake<T> {
    pub bodies: VecDeque<Coordinate<T>>, // head, ..., tail
    pub acceleration_time_left: u32,
    pub speed: T,
    pub color: String,
    pub velocity: Coordinate<T>,
    pub size: usize,
    pub frame_count_offset: u32,
    pub is_visible_head: bool, // for rendering
}

impl<T> Snake<T>
where
    T: Float,
    f32: Into<T>,
{
    pub fn new(initial_position: Coordinate<T>, initial_speed: T) -> Snake<T> {
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
            velocity: Coordinate {
                x: T::one(),
                y: T::zero(),
            },
            frame_count_offset: 0,
            is_visible_head: true,
        }
    }

    pub fn clone(&self) -> Snake<T> {
        let mut bodies = VecDeque::new();
        for body in self.bodies.iter() {
            bodies.push_back(body.clone());
        }
        Snake {
            bodies,
            acceleration_time_left: self.acceleration_time_left,
            speed: self.speed,
            size: self.size,
            color: self.color.clone(),
            velocity: self.velocity.clone(),
            frame_count_offset: self.frame_count_offset,
            is_visible_head: self.is_visible_head,
        }
    }

    pub fn get_head(&self) -> &Coordinate<T> {
        &self.bodies[0]
    }

    pub fn get_tail(&self) -> &Coordinate<T> {
        &self.bodies[self.bodies.len() - 1]
    }

    pub fn accelerate(&mut self) {
        if self.bodies.len() < 20 {
            return;
        }
        self.acceleration_time_left = 60;
    }
}
