use serde::Serialize;

use super::pellet::Pellet;
use super::snake::Snake;

#[derive(Serialize)]
pub struct View<T> {
    pub is_alive: bool,
    pub snakes: Vec<Snake<T>>,
    pub pellets: Vec<Pellet<T>>,
    pub map: Vec<Vec<u32>>,
    pub self_coordinate: (usize, usize),
}
