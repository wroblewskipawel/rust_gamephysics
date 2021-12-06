use ash::{self, vk};
use winit::window::Window;

use std::ffi::CStr;
use std::os::raw::c_char;

mod debug;
mod device;
mod surface;

use device::{Device, Frame};
use surface::Surface;

use crate::math::types::Matrix4;
use crate::renderer::{MeshHandle, Renderer};
use crate::utils::StaticResult;

use super::{Camera, Mesh};

struct Instance {
    instance: ash::Instance,
    entry: ash::Entry,
}

pub struct Backend {
    current_frame: Option<Frame>,
    device: Device,
    surface: Surface,
    messenger: debug::Messenger,
    instance: Instance,
}

impl Instance {
    fn new(window: &Window) -> StaticResult<Self> {
        let entry = unsafe { ash::Entry::new()? };
        let mut required_extensions: Vec<_> = ash_window::enumerate_required_extensions(window)?;
        required_extensions.append(&mut debug::required_extensions());

        let supported_extensions = entry.enumerate_instance_extension_properties()?;
        for &req in &required_extensions {
            supported_extensions
                .iter()
                .find(|ext| unsafe { CStr::from_ptr(&ext.extension_name as *const c_char) } == req)
                .ok_or(format!(
                    "Required Vulkan extension [{}] not supported",
                    req.to_str().unwrap_or("UTF8 PARSE ERROR")
                ))?;
        }

        let required_layers = debug::required_layers();
        let supported_layers = entry.enumerate_instance_layer_properties()?;
        for &req in &required_layers {
            supported_layers
                .iter()
                .find(|layer| unsafe { CStr::from_ptr(&layer.layer_name as *const c_char) } == req)
                .ok_or(format!(
                    "Required Vulkan layer [{}] not supported",
                    req.to_str().unwrap_or("UTF8 PARSE ERROR")
                ))?;
        }

        let required_extensions: Vec<_> =
            required_extensions.iter().map(|ext| ext.as_ptr()).collect();

        let required_layers: Vec<_> = required_layers.iter().map(|layer| layer.as_ptr()).collect();

        let app_info = vk::ApplicationInfo {
            api_version: vk::API_VERSION_1_2,
            ..Default::default()
        };

        let instance = unsafe {
            entry.create_instance(
                &vk::InstanceCreateInfo::builder()
                    .application_info(&app_info)
                    .enabled_extension_names(&required_extensions)
                    .enabled_layer_names(&required_layers)
                    .push_next(debug::MessengerBuilder::new().as_mut()),
                None,
            )?
        };

        Ok(Self { instance, entry })
    }
}

impl AsRef<ash::Instance> for Instance {
    fn as_ref(&self) -> &ash::Instance {
        &self.instance
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { self.instance.destroy_instance(None) }
    }
}

impl Backend {
    pub fn new(window: &Window, meshes: &[Mesh]) -> StaticResult<Self> {
        let instance = Instance::new(window)?;
        let messenger = debug::MessengerBuilder::new().build(&instance.entry, instance.as_ref())?;
        let surface = Surface::new(&instance.entry, instance.as_ref(), window)?;
        let device = Device::new(instance.as_ref(), &surface, meshes)?;

        Ok(Self {
            device,
            surface,
            messenger,
            instance,
            current_frame: None,
        })
    }
}

impl Renderer for Backend {
    fn begin_frame(&mut self, camera: &Camera) -> StaticResult<()> {
        if self.current_frame.is_none() {
            self.current_frame = Some(self.device.begin_frame(&camera.matrix())?);
        }
        Ok(())
    }
    fn draw(&mut self, mesh: MeshHandle, world: &Matrix4) {
        if self.current_frame.is_some() {
            let frame = self.current_frame.as_ref().unwrap();
            self.device.draw(frame, mesh, world)
        }
    }
    fn end_frame(&mut self) -> StaticResult<()> {
        if self.current_frame.is_some() {
            self.device.end_frame(self.current_frame.take().unwrap())?;
        }
        Ok(())
    }
}
