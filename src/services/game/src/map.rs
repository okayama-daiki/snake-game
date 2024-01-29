use serde::Serialize;

#[derive(Serialize)]
pub struct Map {
    pub map: Vec<Vec<u32>>,
    pub self_coordinate: (usize, usize),
}
