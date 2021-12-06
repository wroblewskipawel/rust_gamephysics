use super::{Matrix3, Vector3};
use bytemuck::{Pod, Zeroable};
use std::ops::Mul;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub r: f32,
    pub i: f32,
    pub j: f32,
    pub k: f32,
}

unsafe impl Zeroable for Quaternion {}
unsafe impl Pod for Quaternion {}

impl Quaternion {
    #[inline]
    pub const fn new(r: f32, i: f32, j: f32, k: f32) -> Self {
        Self { r, i, j, k }
    }

    #[inline]
    pub fn vec_angle(n: Vector3, rad: f32) -> Self {
        let (s, r) = f32::sin_cos(0.5 * rad);
        let Vector3 { x, y, z } = n.normalized() * s;
        Self {
            i: x,
            j: y,
            k: z,
            r,
        }
    }

    #[inline]
    pub fn normalized(self) -> Self {
        let mag_inv = 1.0 / self.mag();
        if mag_inv.is_finite() {
            self * mag_inv
        } else {
            self
        }
    }

    #[inline]
    pub fn inverse(self) -> Self {
        let mag_sqr_inv = 1.0 / self.mag_squared();
        Self {
            r: self.r * mag_sqr_inv,
            i: -self.i * mag_sqr_inv,
            j: -self.j * mag_sqr_inv,
            k: -self.k * mag_sqr_inv,
        }
    }

    #[inline]
    pub fn mag_squared(self) -> f32 {
        self.r * self.r + self.i * self.i + self.j * self.j + self.k * self.k
    }

    #[inline]
    pub fn mag(self) -> f32 {
        self.mag_squared().sqrt()
    }

    #[inline]
    pub fn rotate_point(self, point: Vector3) -> Vector3 {
        let point = self * Self::new(0.0, point.x, point.y, point.z) * self.inverse();
        Vector3 {
            x: point.i,
            y: point.j,
            z: point.k,
        }
    }

    #[inline]
    pub fn rotate_matrix(self, mat: Matrix3) -> Matrix3 {
        let Matrix3 { i, j, k } = mat.transpose();
        Matrix3 {
            i: self.rotate_point(i),
            j: self.rotate_point(j),
            k: self.rotate_point(k),
        }
        .transpose()
    }

    #[inline]
    pub fn xyz(self) -> Vector3 {
        Vector3 {
            x: self.i,
            y: self.j,
            z: self.k,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        self.r.is_finite() && self.i.is_finite() & self.j.is_finite() && self.k.is_finite()
    }
}

impl Default for Quaternion {
    #[inline]
    fn default() -> Self {
        Self {
            r: 1.0,
            i: 0.0,
            j: 0.0,
            k: 0.0,
        }
    }
}

impl Mul<f32> for Quaternion {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            i: self.i * rhs,
            j: self.j * rhs,
            k: self.k * rhs,
        }
    }
}

impl Mul for Quaternion {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k,
            i: self.i * rhs.r + self.r * rhs.i + self.j * rhs.k - self.k * rhs.j,
            j: self.j * rhs.r + self.r * rhs.j + self.k * rhs.i - self.i * rhs.k,
            k: self.k * rhs.r + self.r * rhs.i + self.i * rhs.j - self.j * rhs.i,
        }
    }
}
