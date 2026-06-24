use crate::{Box2D, Point2D, Size2D};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point2D,
    pub size: Size2D,
}

impl Rect {
    pub const ZERO: Self = Self::new(Point2D::ZERO, Size2D::ZERO);

    pub const fn new(origin: Point2D, size: Size2D) -> Self {
        Self { origin, size }
    }

    pub fn center(&self) -> Point2D {
        self.origin + self.size / 2.0
    }

    pub fn to_box2d(self) -> Box2D {
        Box2D {
            min: self.origin,
            max: self.origin + self.size,
        }
    }
}
