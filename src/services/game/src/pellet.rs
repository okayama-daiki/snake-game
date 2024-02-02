use rand::Rng;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Clone)]
pub struct Pellet {
    pub center: Coordinate,
    pub radius: f32,
    pub position: Coordinate,
    pub size: u8,
    pub color: String,
    pub frame_count_offset: u32,
}

impl Pellet {
    pub fn new(initial_position: Coordinate) -> Pellet {
        Pellet {
            center: initial_position,
            radius: rand::thread_rng().gen_range(0.5, 5.),
            position: initial_position,
            size: rand::thread_rng().gen_range(1, 4),
            color: COLORS[rand::thread_rng().gen_range(0, COLORS.len())].to_string(),
            frame_count_offset: 0,
        }
    }

    pub fn new_with_color_and_size(
        initial_position: Coordinate,
        color: String,
        size: u8,
    ) -> Pellet {
        Pellet {
            center: initial_position,
            radius: Rng::gen_range(&mut rand::thread_rng(), 0.5, 5.0),
            position: initial_position,
            size,
            color,
            frame_count_offset: 0,
        }
    }

    pub fn update(&mut self) {
        let theta = self.frame_count_offset % 72 * 5;
        let rad = theta as f32 * std::f32::consts::PI / 180.0;
        self.position.x = self.center.x + self.radius * rad.cos();
        self.position.y = self.center.y + self.radius * rad.sin();
    }
}
