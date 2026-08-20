use serde::{Deserialize, Serialize};

const FIELD_SIZE: f32 = 10000.0;

#[derive(Serialize, Deserialize, Default, Clone, Copy, Debug, PartialEq)]
#[serde(into = "(f32, f32)", from = "(f32, f32)")]
pub struct Coordinate {
    pub x: f32,
    pub y: f32,
}

impl Coordinate {
    pub fn distance2(&self, other: &Coordinate) -> f32 {
        let dx = (self.x - other.x).abs().rem_euclid(FIELD_SIZE);
        let dy = (self.y - other.y).abs().rem_euclid(FIELD_SIZE);
        let dx = dx.min(FIELD_SIZE - dx);
        let dy = dy.min(FIELD_SIZE - dy);

        dx.powi(2) + dy.powi(2)
    }

    pub fn is_in_rectangle(&self, x0: f32, y0: f32, width: f32, height: f32) -> bool {
        //! Check if the coordinate is in the rectangle.
        //! Left-top corner is (x0, y0) and the size is (width, height).
        //! Note that the rectangle is on the torus.

        axis_contains(self.x, x0, width) && axis_contains(self.y, y0, height)
    }
}

impl From<Coordinate> for (f32, f32) {
    fn from(coordinate: Coordinate) -> Self {
        (coordinate.x, coordinate.y)
    }
}

impl From<(f32, f32)> for Coordinate {
    fn from((x, y): (f32, f32)) -> Self {
        Self { x, y }
    }
}

fn axis_contains(value: f32, start: f32, length: f32) -> bool {
    if !value.is_finite() || !start.is_finite() || !length.is_finite() || length < 0.0 {
        return false;
    }
    if length >= FIELD_SIZE {
        return true;
    }

    let value = value.rem_euclid(FIELD_SIZE);
    let start = start.rem_euclid(FIELD_SIZE);
    let end = start + length;

    if end <= FIELD_SIZE {
        start <= value && value <= end
    } else {
        start <= value || value <= end - FIELD_SIZE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn distance_wraps_around_the_field_edges() {
        let left = Coordinate { x: 5.0, y: 10.0 };
        let right = Coordinate {
            x: FIELD_SIZE - 5.0,
            y: 10.0,
        };

        assert_eq!(left.distance2(&right), 100.0);
    }

    #[test]
    fn rectangle_starting_at_zero_does_not_include_the_whole_axis() {
        let inside = Coordinate { x: 50.0, y: 50.0 };
        let outside = Coordinate { x: 500.0, y: 50.0 };

        assert!(inside.is_in_rectangle(0.0, 0.0, 100.0, 100.0));
        assert!(!outside.is_in_rectangle(0.0, 0.0, 100.0, 100.0));
    }

    #[test]
    fn rectangle_wraps_around_the_field_edges() {
        let wrapped = Coordinate { x: 25.0, y: 50.0 };

        assert!(wrapped.is_in_rectangle(FIELD_SIZE - 50.0, 0.0, 100.0, 100.0));
    }
}
