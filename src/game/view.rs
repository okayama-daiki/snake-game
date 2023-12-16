use serde::Serialize;

use super::pellet::Pellet;
use super::snake::Snake;

#[derive(Serialize)]
pub struct View {
    pub is_alive: bool,
    pub snakes: Vec<Snake>,
    pub pellets: Vec<Pellet>,
}
