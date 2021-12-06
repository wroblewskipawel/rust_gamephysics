use ash::{extensions::khr, prelude::VkResult, vk};
use ash_window;
use winit::window::Window;

use crate::utils::StaticResult;

pub(super) struct Surface {
    loader: khr::Surface,
    pub(super) handle: vk::SurfaceKHR,
}

impl Surface {
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &Window,
    ) -> StaticResult<Self> {
        let loader = khr::Surface::new(entry, instance);
        let handle = unsafe { ash_window::create_surface(entry, instance, window, None)? };
        Ok(Self { loader, handle })
    }

    pub fn device_surface_support(
        &self,
        device: vk::PhysicalDevice,
        family: u32,
    ) -> VkResult<bool> {
        unsafe {
            self.loader
                .get_physical_device_surface_support(device, family, self.handle)
        }
    }

    pub fn device_surface_capabilities(
        &self,
        device: vk::PhysicalDevice,
    ) -> VkResult<vk::SurfaceCapabilitiesKHR> {
        unsafe {
            self.loader
                .get_physical_device_surface_capabilities(device, self.handle)
        }
    }

    pub fn device_present_modes(
        &self,
        device: vk::PhysicalDevice,
    ) -> VkResult<Vec<vk::PresentModeKHR>> {
        unsafe {
            self.loader
                .get_physical_device_surface_present_modes(device, self.handle)
        }
    }

    pub fn device_surface_formats(
        &self,
        device: vk::PhysicalDevice,
    ) -> VkResult<Vec<vk::SurfaceFormatKHR>> {
        unsafe {
            self.loader
                .get_physical_device_surface_formats(device, self.handle)
        }
    }
}

impl Drop for Surface {
    fn drop(&mut self) {
        unsafe {
            self.loader.destroy_surface(self.handle, None);
        }
    }
}
