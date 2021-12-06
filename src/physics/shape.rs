use crate::math::types::Vector3;

mod cuboid;
mod sphere;

pub use cuboid::*;
pub use sphere::*;

#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Cuboid(cuboid::Cuboid),
    Sphere(sphere::Sphere),
}

impl Shape {
    pub fn new_cuboid(bounds: Vector3) -> Self {
        Self::Cuboid(Cuboid {
            bounds_min: -bounds / 2.0,
            bounds_max: bounds / 2.0,
        })
    }

    pub fn new_sphere(radius: f32) -> Self {
        Self::Sphere(Sphere { radius })
    }
}
