use crate::{
    math::{
        transforms,
        types::{Matrix4, Vector3},
    },
    physics, renderer,
};

use crate::utils::StaticResult;

pub struct Object {
    shape: physics::Shape,
    pub(super) world: Matrix4,
    pub(super) mesh: renderer::MeshHandle,
}

pub struct SceneBuilder {
    pub(super) shapes: Vec<physics::Shape>,
    pub(super) meshes: Vec<renderer::Mesh>,
    pub(super) camera: Option<renderer::CameraBuilder>,
    pub(super) objects: Vec<Object>,
}

#[derive(Debug, Clone, Copy)]
pub struct ShapeHandle {
    index: usize,
}

pub struct Scene {
    pub(super) objects: Vec<Object>,
    pub(super) camera: renderer::Camera,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            meshes: vec![],
            shapes: vec![],
            objects: vec![],
            camera: None,
        }
    }

    pub fn add_shape(&mut self, shape: physics::Shape) -> ShapeHandle {
        self.meshes.push(renderer::Mesh::from_shape(&shape));
        self.shapes.push(shape);
        ShapeHandle {
            index: self.shapes.len() - 1,
        }
    }

    pub fn add_instance(&mut self, shape: ShapeHandle, location: Vector3) {
        self.objects.push(Object {
            shape: self.shapes[shape.index],
            mesh: renderer::MeshHandle(shape.index),
            world: transforms::translate(location),
        })
    }

    pub fn set_camera(&mut self, eye: Vector3, center: Vector3) {
        self.camera = Some(renderer::CameraBuilder::new(eye, center));
    }

    pub fn build(self, fovy_deg: f32, aspect: f32, near: f32, far: f32) -> StaticResult<Scene> {
        let camera = self
            .camera
            .ok_or(format!("Camera not provided"))?
            .build(fovy_deg, aspect, near, far);
        Ok(Scene {
            camera,
            objects: self.objects,
        })
    }
}
