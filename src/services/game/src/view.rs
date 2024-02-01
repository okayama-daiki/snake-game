use ciborium::{de::from_reader, ser::into_writer};
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

impl View {
    pub fn from_bytes(bytes: &[u8]) -> View {
        from_reader(bytes).unwrap()
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        into_writer(&self, &mut bytes).unwrap();
        bytes
    }
}
