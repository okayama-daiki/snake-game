use num_traits::Float;
use serde::Serialize;

static FIELD_SIZE: f32 = 10000.0;

#[derive(Serialize, Default, Clone)]
pub struct Coordinate<T> {
    pub x: T,
    pub y: T,
}

impl<T> Coordinate<T>
where
    T: Float,
    f32: Into<T>,
{
    pub fn distance2(&self, other: &Coordinate<T>) -> T {
        (self.x - other.x).powi(2) + (self.y - other.y).powi(2)
    }

    pub fn is_in_rectangle(&self, x0: T, y0: T, width: T, height: T) -> bool {
        //! Check if the coordinate is in the rectangle.
        //! Left-top corner is (x0, y0) and the size is (width, height).
        //! Note that the rectangle is on the torus.

        let b = FIELD_SIZE.into();

        let y_condition = if T::zero() < y0 && y0 + height < b {
            y0 <= self.y && self.y <= y0 + height
        } else {
            (y0 + b) % b <= self.y || self.y <= (y0 + height) % b
        };
        let x_condition = if T::zero() < x0 && x0 + width < b {
            x0 <= self.x && self.x <= x0 + width
        } else {
            (x0 + b) % b <= self.x || self.x <= (x0 + width) % b
        };

        x_condition && y_condition
    }
}
