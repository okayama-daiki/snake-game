use num_traits::Float;
use rand::Rng;
use serde::Serialize;

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
pub struct Pellet<T> {
    pub position: Coordinate<T>,
    pub size: u8,
    pub color: String,
    pub frame_count_offset: u32,
}

impl<T: Float> Pellet<T> {
    pub fn new(initial_position: Coordinate<T>) -> Pellet<T> {
        Pellet {
            position: initial_position,
            size: rand::thread_rng().gen_range(1..=3),
            color: COLORS[rand::thread_rng().gen_range(0..COLORS.len())].to_string(),
            frame_count_offset: 0,
        }
    }

    pub fn new_with_color_and_size(
        initial_position: Coordinate<T>,
        color: String,
        size: u8,
    ) -> Pellet<T> {
        Pellet {
            position: initial_position,
            size,
            color,
            frame_count_offset: 0,
        }
    }

    pub fn clone(&self) -> Pellet<T> {
        Pellet {
            position: self.position.clone(),
            size: self.size,
            color: self.color.clone(),
            frame_count_offset: self.frame_count_offset,
        }
    }
}
