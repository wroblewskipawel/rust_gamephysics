use crate::math::types::Matrix4;
use winit::window::Window;

use crate::utils::StaticResult;

mod camera;
mod mesh;
mod vulkan;

pub use camera::{Camera, CameraBuilder};
pub(super) use mesh::Mesh;

#[derive(Debug, Clone, Copy)]
pub struct MeshHandle(pub usize);

pub enum Backend {
    Vulkan,
}

pub trait Renderer {
    fn begin_frame(&mut self, camera: &Camera) -> StaticResult<()>;
    fn draw(&mut self, model: MeshHandle, world: &Matrix4);
    fn end_frame(&mut self) -> StaticResult<()>;
}

pub fn create(
    backend: Backend,
    window: &Window,
    meshes: &[Mesh],
) -> StaticResult<Box<dyn Renderer>> {
    match backend {
        Backend::Vulkan => Ok(Box::new(vulkan::Backend::new(window, meshes)?)),
    }
}
