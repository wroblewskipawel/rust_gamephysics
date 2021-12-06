use bytemuck::{Pod, Zeroable};
use std::ops::{Add, Div, Index, IndexMut, Mul, Neg, Sub};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

unsafe impl Zeroable for Vector2 {}
unsafe impl Pod for Vector2 {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

unsafe impl Zeroable for Vector3 {}
unsafe impl Pod for Vector3 {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Vector4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

unsafe impl Zeroable for Vector4 {}
unsafe impl Pod for Vector4 {}

impl Vector2 {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    #[inline]
    pub fn mag_squared(self) -> f32 {
        self * self
    }

    #[inline]
    pub fn mag(self) -> f32 {
        f32::sqrt(self * self)
    }

    #[inline]
    pub fn normalized(self) -> Self {
        self / self.mag()
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite()
    }
}

impl Add for Vector2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl Sub for Vector2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl Mul for Vector2 {
    type Output = f32;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y
    }
}

impl Mul<f32> for Vector2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl Div<f32> for Vector2 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        let rhs = 1.0 / rhs;
        self * rhs
    }
}

impl Neg for Vector2 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self * -1.0f32
    }
}

impl Index<usize> for Vector2 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 2);
        unsafe { &*(&self.x as *const f32).offset(index as isize) }
    }
}

impl IndexMut<usize> for Vector2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < 2);
        unsafe { &mut *(&mut self.x as *mut f32).offset(index as isize) }
    }
}

impl Vector3 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn mag_squared(self) -> f32 {
        self * self
    }

    #[inline]
    pub fn mag(self) -> f32 {
        f32::sqrt(self * self)
    }

    #[inline]
    pub fn normalized(self) -> Self {
        self / self.mag()
    }

    #[inline]
    pub fn cross(self, rhs: Self) -> Self {
        Self {
            x: self.y * rhs.z - rhs.y * self.z,
            y: rhs.x * self.z - self.x * rhs.z,
            z: self.x * rhs.y - rhs.x * self.y,
        }
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite()
    }

    pub fn ortho(self) -> (Self, Self, Self) {
        let n = self.normalized();
        let w = if n.z * n.z > 0.9f32 * 0.9f32 {
            Self::new(1.0, 0.0, 0.0)
        } else {
            Self::new(0.0, 0.0, 1.0)
        };
        let mut u = w.cross(n).normalized();
        let v = n.cross(u).normalized();
        u = v.cross(n).normalized();

        (n, u, v)
    }
}

impl Add for Vector3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
        }
    }
}

impl Mul for Vector3 {
    type Output = f32;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
        }
    }
}

impl Div<f32> for Vector3 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        let rhs = 1.0 / rhs;
        self * rhs
    }
}

impl Neg for Vector3 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self * -1.0f32
    }
}

impl Index<usize> for Vector3 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 3);
        unsafe { &*(&self.x as *const f32).offset(index as isize) }
    }
}

impl IndexMut<usize> for Vector3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < 3);
        unsafe { &mut *(&mut self.x as *mut f32).offset(index as isize) }
    }
}

impl Vector4 {
    #[inline]
    pub const fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    #[inline]
    pub fn hom_point(point: Vector3) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
            w: 1.0,
        }
    }

    #[inline]
    pub fn hom_vec(point: Vector3) -> Self {
        Self {
            x: point.x,
            y: point.y,
            z: point.z,
            w: 0.0,
        }
    }

    #[inline]
    pub fn mag_squared(self) -> f32 {
        self * self
    }

    #[inline]
    pub fn mag(self) -> f32 {
        f32::sqrt(self * self)
    }

    #[inline]
    pub fn normalized(self) -> Self {
        self / self.mag()
    }

    #[inline]
    pub fn is_valid(self) -> bool {
        self.x.is_finite() && self.y.is_finite() && self.z.is_finite() && self.w.is_finite()
    }
}

impl Add for Vector4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
            z: self.z + rhs.z,
            w: self.w + rhs.w,
        }
    }
}

impl Sub for Vector4 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
            z: self.z - rhs.z,
            w: self.w - rhs.w,
        }
    }
}

impl Mul for Vector4 {
    type Output = f32;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        self.x * rhs.x + self.y * rhs.y + self.z * rhs.z + self.w * rhs.w
    }
}

impl Mul<f32> for Vector4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            x: self.x * rhs,
            y: self.y * rhs,
            z: self.z * rhs,
            w: self.w * rhs,
        }
    }
}

impl Div<f32> for Vector4 {
    type Output = Self;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        let rhs = 1.0 / rhs;
        self * rhs
    }
}

impl Neg for Vector4 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        self * -1.0f32
    }
}

impl Index<usize> for Vector4 {
    type Output = f32;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 4);
        unsafe { &*(&self.x as *const f32).offset(index as isize) }
    }
}

impl IndexMut<usize> for Vector4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < 4);
        unsafe { &mut *(&mut self.x as *mut f32).offset(index as isize) }
    }
}
