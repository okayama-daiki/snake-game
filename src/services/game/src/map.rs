use ciborium::{
    de::{from_reader, Error as CiboriumError},
    ser::into_writer,
};
use serde::{Deserialize, Serialize};
use std::io::Error;

#[derive(Serialize, Deserialize)]
pub struct Map {
    pub map: Vec<Vec<u32>>,
    pub self_coordinate: (usize, usize),
}

impl Map {
    pub fn from_bytes(bytes: &[u8]) -> Result<Map, CiboriumError<Error>> {
        from_reader(bytes)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![];
        into_writer(&self, &mut bytes).unwrap();
        bytes
    }
}
