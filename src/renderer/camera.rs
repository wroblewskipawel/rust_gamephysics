use crate::math::{
    transforms::{look_at, perspective},
    types::{Matrix4, Vector2, Vector3},
};

pub struct CameraBuilder {
    eye: Vector3,
    center: Vector3,
}

pub struct Camera {
    view: Matrix4,
    proj: Matrix4,
}

impl Camera {
    fn new(eye: Vector3, center: Vector3, fovy_deg: f32, aspect: f32, near: f32, far: f32) -> Self {
        Self {
            proj: perspective(fovy_deg, aspect, near, far),
            view: look_at(eye, center, Vector3::new(0.0, 0.0, 1.0)),
        }
    }

    pub(super) fn matrix(&self) -> Matrix4 {
        self.proj * self.view
    }
}

impl CameraBuilder {
    pub fn new(eye: Vector3, center: Vector3) -> Self {
        Self { eye, center }
    }

    pub fn build(self, fovy_deg: f32, aspect: f32, near: f32, far: f32) -> Camera {
        Camera::new(self.eye, self.center, fovy_deg, aspect, near, far)
    }
}
