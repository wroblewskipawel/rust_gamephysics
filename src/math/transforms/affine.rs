use crate::math::types::{Matrix4, Vector3, Vector4};

#[inline]
pub fn translate(point: Vector3) -> Matrix4 {
    Matrix4 {
        i: Vector4::new(1.0, 0.0, 0.0, 0.0),
        j: Vector4::new(0.0, 1.0, 0.0, 0.0),
        k: Vector4::new(0.0, 0.0, 1.0, 0.0),
        l: Vector4::new(point.x, point.y, point.z, 1.0),
    }
}

#[inline]
pub fn rot_x(rad: f32) -> Matrix4 {
    Matrix4 {
        i: Vector4::new(1.0, 0.0, 0.0, 0.0),
        j: Vector4::new(0.0, f32::cos(rad), f32::sin(rad), 0.0),
        k: Vector4::new(0.0, -f32::sin(rad), f32::cos(rad), 0.0),
        l: Vector4::new(0.0, 0.0, 0.0, 1.0),
    }
}

#[inline]
pub fn rot_y(rad: f32) -> Matrix4 {
    Matrix4 {
        i: Vector4::new(f32::cos(rad), 0.0, -f32::sin(rad), 0.0),
        j: Vector4::new(0.0, 1.0, 0.0, 0.0),
        k: Vector4::new(f32::sin(rad), 0.0, f32::cos(rad), 0.0),
        l: Vector4::new(0.0, 0.0, 0.0, 1.0),
    }
}

#[inline]
pub fn rot_z(rad: f32) -> Matrix4 {
    Matrix4 {
        i: Vector4::new(f32::cos(rad), f32::sin(rad), 0.0, 0.0),
        j: Vector4::new(-f32::sin(rad), f32::cos(rad), 0.0, 0.0),
        k: Vector4::new(0.0, 0.0, 1.0, 0.0),
        l: Vector4::new(0.0, 0.0, 0.0, 1.0),
    }
}

#[inline]
pub fn rot_axis(rad: f32, axis: Vector3) -> Matrix4 {
    let (i, j, k) = axis.ortho();
    let align_inv = Matrix4 {
        i: Vector4::hom_vec(i),
        j: Vector4::hom_vec(j),
        k: Vector4::hom_vec(k),
        l: Vector4::new(0.0, 0.0, 0.0, 1.0),
    };
    let align = align_inv.transpose();
    align * rot_x(rad) * align_inv
}

#[inline]
pub fn align_x_axis(axis: Vector3) -> Matrix4 {
    let (i, j, k) = axis.ortho();
    Matrix4 {
        i: Vector4::hom_vec(i),
        j: Vector4::hom_vec(j),
        k: Vector4::hom_vec(k),
        l: Vector4::new(0.0, 0.0, 0.0, 1.0),
    }
    .transpose()
}

#[inline]
pub fn look_at(eye: Vector3, center: Vector3, up: Vector3) -> Matrix4 {
    let front = (center - eye).normalized();
    let right = front.cross(up).normalized();
    let up = right.cross(front).normalized();
    Matrix4 {
        i: Vector4::new(right.x, up.x, front.x, 0.0),
        j: Vector4::new(right.y, up.y, front.y, 0.0),
        k: Vector4::new(right.z, up.z, front.z, 0.0),
        l: Vector4::new(-(eye * right), -(eye * up), -(eye * front), 1.0),
    }
}

#[inline]
pub fn scale(s: f32) -> Matrix4 {
    Matrix4 {
        i: Vector4::new(s, 0.0, 0.0, 0.0),
        j: Vector4::new(0.0, s, 0.0, 0.0),
        k: Vector4::new(0.0, 0.0, s, 0.0),
        l: Vector4::new(0.0, 0.0, 0.0, 1.0),
    }
}

#[inline]
pub fn scale_nonuniform(x: f32, y: f32, z: f32) -> Matrix4 {
    Matrix4 {
        i: Vector4::new(x, 0.0, 0.0, 0.0),
        j: Vector4::new(0.0, y, 0.0, 0.0),
        k: Vector4::new(0.0, 0.0, z, 0.0),
        l: Vector4::new(0.0, 0.0, 0.0, 1.0),
    }
}
