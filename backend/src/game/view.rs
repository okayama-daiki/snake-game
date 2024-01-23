use serde::Serialize;

use super::coordinate::Coordinate;
use super::map::Map;
use super::pellet::Pellet;
use super::snake::Snake;

#[derive(Serialize)]
pub struct View<T> {
    pub is_alive: bool,
    pub snakes: Vec<Snake<T>>,
    pub pellets: Vec<Pellet<T>>,
    pub background_dots: Vec<Coordinate<T>>,
    pub map: Map,
}
