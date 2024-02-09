use ciborium::{
    de::{from_reader, Error as CiboriumError},
    ser::into_writer,
};
use serde::{Deserialize, Serialize};
use std::io::Error;

use super::coordinate::Coordinate;
use super::pellet::Pellet;
use super::snake::Snake;

#[derive(Serialize, Deserialize)]
pub struct View {
    pub is_alive: bool,
    pub snakes: Vec<Snake>,
    pub pellets: Vec<Pellet>,
    pub background_dots: Vec<Coordinate>,
}

impl View {
    pub fn from_bytes(bytes: &[u8]) -> Result<View, CiboriumError<Error>> {
        from_reader(bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        into_writer(&self, &mut bytes).unwrap();
        bytes
    }
}
