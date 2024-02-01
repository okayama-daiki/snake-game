use serde::{Deserialize, Serialize};

static FIELD_SIZE: f32 = 10000.0;

#[derive(Serialize, Deserialize, Default, Clone, Copy)]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

impl Coordinate {
    pub fn distance2(&self, other: &Coordinate) -> f32 {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    pub fn is_in_rectangle(&self, x0: f32, y0: f32, width: f32, height: f32) -> bool {
        //! Check if the coordinate is in the rectangle.
        //! Left-top corner is (x0, y0) and the size is (width, height).
        //! Note that the rectangle is on the torus.

        let b = FIELD_SIZE;

        let y_condition = if 0. < y0 && y0 + height < b {
            y0 <= self.y && self.y <= y0 + height
        } else {
            (y0 + b) % b <= self.y || self.y <= (y0 + height) % b
        };
        let x_condition = if 0. < x0 && x0 + width < b {
            x0 <= self.x && self.x <= x0 + width
        } else {
            (x0 + b) % b <= self.x || self.x <= (x0 + width) % b
        };

        x_condition && y_condition
    }
}
