use serde::{Deserialize, Serialize};

use super::coordinate::Coordinate;
use super::map::Map;
use super::pellet::Pellet;
use super::snake::Snake;

#[derive(Serialize, Deserialize)]
pub struct View {
    pub is_alive: bool,
    pub snakes: Vec<Snake>,
    pub pellets: Vec<Pellet>,
    pub background_dots: Vec<Coordinate>,
    pub map: Map,
}
