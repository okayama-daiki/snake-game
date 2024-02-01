use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub map: Vec<Vec<u32>>,
    pub self_coordinate: (usize, usize),
}
