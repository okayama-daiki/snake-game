use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Pellet {
    pub position: Coordinate,
    pub size: f64,
    pub color: String,
    pub frame_count_offset: u32,
}

impl Pellet {
    pub fn hsl(&self) -> String {
        format!(
            "hsl({}, 100%, {}%)",
            self.color,
            (30. * (self.frame_count_offset as f64 / 7.).sin()).abs() + 50.
        )
    }
    pub fn radius(&self) -> f64 {
        (self.size * 2.).min(self.frame_count_offset as f64)
    }
}

#[derive(Serialize, Deserialize)]
pub struct Snake {
    pub bodies: Vec<Coordinate>,
    pub acceleration_time_left: u32,
    pub speed: f32,
    pub color: String,
    pub velocity: Coordinate,
    pub size: u32,
    pub frame_count_offset: u32,
}

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub map: Vec<Vec<u32>>,
    pub self_coordinate: [usize; 2],
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub is_alive: bool,
    pub snakes: Vec<Snake>,
    pub pellets: Vec<Pellet>,
    pub background_dots: Vec<Coordinate>,
    pub map: Map,
}
