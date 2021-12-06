use crate::math::types::{Matrix4, Vector4};

#[inline]
pub fn perspective(fovy_deg: f32, aspect: f32, near: f32, far: f32) -> Matrix4 {
    let fovy = f32::to_radians(fovy_deg);
    let xscale = 1.0 / f32::tan(fovy * 0.5);
    let yscale = xscale / aspect;
    let zscale = 0.5 * (far + near) / (near - far) - 0.5;
    let zpos = (far * near) / (near - far);
    Matrix4 {
        i: Vector4::new(xscale, 0.0, 0.0, 0.0),
        j: Vector4::new(0.0, yscale, 0.0, 0.0),
        k: Vector4::new(0.0, 0.0, -zscale, 1.0),
        l: Vector4::new(0.0, 0.0, zpos, 0.0),
    }
}

#[inline]
pub fn ortho(xmin: f32, xmax: f32, ymin: f32, ymax: f32, znear: f32, zfar: f32) -> Matrix4 {
    let width = xmax - xmin;
    let height = ymax - ymin;
    let depth = zfar - znear;

    let tx = -(xmax + xmin) / width;
    let ty = -(ymax + ymin) / height;
    let tz = -(zfar + znear) / depth;

    Matrix4 {
        i: Vector4::new(2.0 / width, 0.0, 0.0, 0.0),
        j: Vector4::new(0.0, -2.0 / height, 0.0, 0.0),
        k: Vector4::new(0.0, 0.0, -depth, 0.0),
        l: Vector4::new(tx, -ty, 0.5 * tz + 0.5, 1.0),
    }
}
