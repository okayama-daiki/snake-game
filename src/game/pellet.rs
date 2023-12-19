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
pub struct Pellet {
    pub position: Coordinate,
    pub size: u8,
    pub color: String,
    pub frame_count_offset: u32,
}

impl Pellet {
    pub fn new(initial_position: Coordinate) -> Pellet {
        Pellet {
            position: initial_position,
            size: rand::thread_rng().gen_range(1..=3),
            color: COLORS[rand::thread_rng().gen_range(0..COLORS.len())].to_string(),
            frame_count_offset: 0,
        }
    }

    pub fn new_with_color_and_size(
        initial_position: Coordinate,
        color: String,
        size: u8,
    ) -> Pellet {
        Pellet {
            position: initial_position,
            size,
            color,
            frame_count_offset: 0,
        }
    }

    pub fn clone(&self) -> Pellet {
        Pellet {
            position: self.position.clone(),
            size: self.size,
            color: self.color.clone(),
            frame_count_offset: self.frame_count_offset,
        }
    }
}
