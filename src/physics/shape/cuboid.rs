use crate::math::types::Vector3;

#[derive(Debug, Clone, Copy)]
pub struct Cuboid {
    pub bounds_min: Vector3,
    pub bounds_max: Vector3,
}
