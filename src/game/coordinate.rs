use num_traits::Float;
use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Coordinate<T> {
    pub x: T,
    pub y: T,
}

impl<T: Float> Coordinate<T> {
    pub fn clone(&self) -> Coordinate<T> {
        Coordinate {
            x: self.x,
            y: self.y,
        }
    }

    pub fn distance2(&self, other: &Coordinate<T>) -> T {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    pub fn is_in_rectangle(&self, x: T, y: T, width: T, height: T) -> bool {
        self.x > x && self.x < x + width && self.y > y && self.y < y + height
    }
}
