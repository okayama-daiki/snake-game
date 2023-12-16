use serde::Serialize;

#[derive(Serialize, Default)]
pub struct Coordinate {
    pub x: f64,
    pub y: f64,
}

impl Coordinate {
    pub fn clone(&self) -> Coordinate {
        Coordinate {
            x: self.x,
            y: self.y,
        }
    }

    pub fn distance2(&self, other: &Coordinate) -> f64 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    pub fn is_in_rectangle(&self, x: f64, y: f64, width: f64, height: f64) -> bool {
        self.x > x && self.x < x + width && self.y > y && self.y < y + height
    }
}
