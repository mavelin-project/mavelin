use crate::Point2D;

#[derive(Debug, Clone, Copy, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Box2D {
    pub min: Point2D,
    pub max: Point2D,
}

impl Box2D {
    pub const fn to_array(self) -> [Point2D; 2] {
        [self.min, self.max]
    }
}
