use bytemuck::{Pod, Zeroable};
use std::ops::{Add, Index, IndexMut, Mul, Sub};

use super::{Quaternion, Vector2, Vector3, Vector4};

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Matrix2 {
    pub i: Vector2,
    pub j: Vector2,
}

unsafe impl Zeroable for Matrix2 {}
unsafe impl Pod for Matrix2 {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Matrix3 {
    pub i: Vector3,
    pub j: Vector3,
    pub k: Vector3,
}

unsafe impl Zeroable for Matrix3 {}
unsafe impl Pod for Matrix3 {}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Matrix4 {
    pub i: Vector4,
    pub j: Vector4,
    pub k: Vector4,
    pub l: Vector4,
}

unsafe impl Zeroable for Matrix4 {}
unsafe impl Pod for Matrix4 {}

impl Matrix2 {
    #[inline]
    pub const fn new(i: Vector2, j: Vector2) -> Self {
        Self { i, j }
    }

    #[inline]
    pub fn iden() -> Self {
        Self {
            i: Vector2::new(1.0, 0.0),
            j: Vector2::new(0.0, 1.0),
        }
    }

    #[inline]
    pub fn det(&self) -> f32 {
        self.i.x * self.j.y - self.i.y * self.j.x
    }
}

impl Add for Matrix2 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i + rhs.i,
            j: self.j + rhs.j,
        }
    }
}

impl Sub for Matrix2 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i - rhs.i,
            j: self.j - rhs.j,
        }
    }
}

impl Mul for Matrix2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let i = self * rhs.i;
        let j = self * rhs.j;
        Self { i, j }
    }
}

impl Mul<Vector2> for Matrix2 {
    type Output = Vector2;
    #[inline]
    fn mul(self, rhs: Vector2) -> Self::Output {
        let x = self.i * rhs.x;
        let y = self.j * rhs.y;
        Vector2 {
            x: x.x + y.x,
            y: x.y + y.y,
        }
    }
}

impl Mul<f32> for Matrix2 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            i: self.i * rhs,
            j: self.j * rhs,
        }
    }
}

impl Index<usize> for Matrix2 {
    type Output = Vector2;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 2);
        unsafe { &*(&self.i as *const Vector2).offset(index as isize) }
    }
}

impl IndexMut<usize> for Matrix2 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < 2);
        unsafe { &mut *(&mut self.i as *mut Vector2).offset(index as isize) }
    }
}

impl Matrix3 {
    #[inline]
    pub const fn new(i: Vector3, j: Vector3, k: Vector3) -> Self {
        Self { i, j, k }
    }

    #[inline]
    pub fn iden() -> Self {
        Self {
            i: Vector3::new(1.0, 0.0, 0.0),
            j: Vector3::new(0.0, 1.0, 0.0),
            k: Vector3::new(0.0, 0.0, 1.0),
        }
    }

    #[inline]
    pub fn det(&self) -> f32 {
        let i = self.i.x * (self.j.y * self.k.z - self.j.z * self.k.y);
        let j = self.i.y * (self.j.x * self.k.z - self.j.z * self.k.x);
        let k = self.i.z * (self.j.x * self.k.y - self.j.y * self.k.x);
        i - j + k
    }

    #[inline]
    pub fn minor(&self, i: usize, j: usize) -> f32 {
        let mut l = 0;
        let mut x = [0; 2];
        for n in 0..3 {
            if n == i {
                continue;
            }
            x[l] = n;
            l += 1;
        }
        l = 0;
        let mut y = [0; 2];
        for n in 0..3 {
            if n == i {
                continue;
            }
            y[l] = n;
            l += 1;
        }
        self[x[0]][y[0]] * self[x[1]][y[1]] - self[x[1]][y[0]] * self[x[0]][y[1]]
    }

    #[inline]
    pub fn cofactor(&self, i: usize, j: usize) -> f32 {
        (-1f32).powi((i + j) as i32) * self.minor(i, j)
    }

    #[inline]
    pub fn trace(&self) -> f32 {
        self.i.x + self.j.y + self.k.z
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Self {
            i: Vector3::new(self.i.x, self.j.x, self.k.x),
            j: Vector3::new(self.i.y, self.j.y, self.k.y),
            k: Vector3::new(self.i.z, self.j.z, self.k.z),
        }
    }

    #[inline]
    pub fn inv(&self) -> Self {
        let mut cof = Self::default();
        for i in 0..3 {
            for j in 0..3 {
                cof[j][i] = self.cofactor(i, j);
            }
        }
        cof * (1f32 / self.det())
    }
}

impl Add for Matrix3 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i + rhs.i,
            j: self.j + rhs.j,
            k: self.k + rhs.k,
        }
    }
}

impl Sub for Matrix3 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i - rhs.i,
            j: self.j - rhs.j,
            k: self.k - rhs.k,
        }
    }
}

impl Mul for Matrix3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let i = self * rhs.i;
        let j = self * rhs.j;
        let k = self * rhs.k;
        Self { i, j, k }
    }
}

impl Mul<Vector3> for Matrix3 {
    type Output = Vector3;
    #[inline]
    fn mul(self, rhs: Vector3) -> Self::Output {
        let x = self.i * rhs.x;
        let y = self.j * rhs.y;
        let z = self.k * rhs.z;
        Vector3 {
            x: x.x + y.x + z.x,
            y: x.y + y.y + z.y,
            z: x.z + y.z + z.z,
        }
    }
}

impl Mul<f32> for Matrix3 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            i: self.i * rhs,
            j: self.j * rhs,
            k: self.k * rhs,
        }
    }
}

impl Index<usize> for Matrix3 {
    type Output = Vector3;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 3);
        unsafe { &*(&self.i as *const Vector3).offset(index as isize) }
    }
}

impl IndexMut<usize> for Matrix3 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < 3);
        unsafe { &mut *(&mut self.i as *mut Vector3).offset(index as isize) }
    }
}

impl From<Quaternion> for Matrix3 {
    #[inline]
    fn from(quat: Quaternion) -> Self {
        let Matrix3 { i, j, k } = Matrix3::iden();
        Matrix3 {
            i: quat.rotate_point(i),
            j: quat.rotate_point(j),
            k: quat.rotate_point(k),
        }
        .transpose()
    }
}

impl Matrix4 {
    #[inline]
    pub const fn new(i: Vector4, j: Vector4, k: Vector4, l: Vector4) -> Self {
        Self { i, j, k, l }
    }

    #[inline]
    pub fn iden() -> Self {
        Self {
            i: Vector4::new(1.0, 0.0, 0.0, 0.0),
            j: Vector4::new(0.0, 1.0, 0.0, 0.0),
            k: Vector4::new(0.0, 0.0, 1.0, 0.0),
            l: Vector4::new(0.0, 0.0, 0.0, 1.0),
        }
    }

    #[inline]
    pub fn det(&self) -> f32 {
        (0..4).fold(0.0, |det, i| det + self.cofactor(i, 0) * self[i][0])
    }

    #[inline]
    pub fn minor(&self, i: usize, j: usize) -> Matrix3 {
        let mut l = 0;
        let mut x = [0; 3];
        for n in 0..4 {
            if n == i {
                continue;
            }
            x[l] = n;
            l += 1;
        }
        l = 0;
        let mut y = [0; 3];
        for n in 0..4 {
            if n == j {
                continue;
            }
            y[l] = n;
            l += 1;
        }
        Matrix3 {
            i: Vector3::new(self[x[0]][y[0]], self[x[1]][x[0]], self[x[2]][x[0]]),
            j: Vector3::new(self[x[0]][y[1]], self[x[1]][x[1]], self[x[2]][x[1]]),
            k: Vector3::new(self[x[0]][y[2]], self[x[1]][x[2]], self[x[2]][x[2]]),
        }
    }

    #[inline]
    pub fn cofactor(&self, i: usize, j: usize) -> f32 {
        (-1.0f32).powi((i + j) as i32) * self.minor(i, j).det()
    }

    #[inline]
    pub fn trace(&self) -> f32 {
        self.i.x + self.j.y + self.k.z + self.l.w
    }

    #[inline]
    pub fn transpose(&self) -> Self {
        Matrix4 {
            i: Vector4::new(self.i.x, self.j.x, self.k.x, self.l.x),
            j: Vector4::new(self.i.y, self.j.y, self.k.y, self.l.y),
            k: Vector4::new(self.i.z, self.j.z, self.k.z, self.l.z),
            l: Vector4::new(self.i.w, self.j.w, self.k.w, self.l.w),
        }
    }

    #[inline]
    pub fn inv(&self) -> Self {
        let mut cof = Self::default();
        for i in 0..4 {
            for j in 0..4 {
                cof[i][j] = self.cofactor(j, i);
            }
        }
        cof * (1f32 / self.det())
    }
}

impl Add for Matrix4 {
    type Output = Self;
    #[inline]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i + rhs.i,
            j: self.j + rhs.j,
            k: self.k + rhs.k,
            l: self.l + rhs.l,
        }
    }
}

impl Sub for Matrix4 {
    type Output = Self;
    #[inline]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            i: self.i - rhs.i,
            j: self.j - rhs.j,
            k: self.k - rhs.k,
            l: self.l - rhs.l,
        }
    }
}

impl Mul for Matrix4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: Self) -> Self::Output {
        let i = self * rhs.i;
        let j = self * rhs.j;
        let k = self * rhs.k;
        let l = self * rhs.l;
        Self { i, j, k, l }
    }
}

impl Mul<Vector4> for Matrix4 {
    type Output = Vector4;
    #[inline]
    fn mul(self, rhs: Vector4) -> Self::Output {
        let x = self.i * rhs.x;
        let y = self.j * rhs.y;
        let z = self.k * rhs.z;
        let l = self.l * rhs.w;
        Vector4 {
            x: x.x + y.x + z.x + l.w,
            y: x.y + y.y + z.y + l.y,
            z: x.z + y.z + z.z + l.z,
            w: x.w + y.w + z.w + l.w,
        }
    }
}

impl Mul<f32> for Matrix4 {
    type Output = Self;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            i: self.i * rhs,
            j: self.j * rhs,
            k: self.k * rhs,
            l: self.l * rhs,
        }
    }
}

impl Index<usize> for Matrix4 {
    type Output = Vector4;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < 4);
        unsafe { &*(&self.i as *const Vector4).offset(index as isize) }
    }
}

impl IndexMut<usize> for Matrix4 {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        assert!(index < 4);
        unsafe { &mut *(&mut self.i as *mut Vector4).offset(index as isize) }
    }
}
