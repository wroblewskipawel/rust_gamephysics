use std::convert::AsRef;
use std::ffi::{c_void, CStr};
use std::ops::{Deref, DerefMut};

use crate::utils::StaticResult;
use ash::{extensions::ext, vk, Entry, Instance};

const REQUIRED_VALIDATION_LAYERS: &'static [&'static [u8]] = &[b"VK_LAYER_KHRONOS_validation\0"];

pub struct Messenger {
    loader: ext::DebugUtils,
    messenger: vk::DebugUtilsMessengerEXT,
}

pub struct MessengerBuilder(vk::DebugUtilsMessengerCreateInfoEXT);

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut c_void,
) -> vk::Bool32 {
    let message_severity = match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => "[ERROR]",
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => "[WARNING]",
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => "[INFO]",
        vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => "[VERBOSE]",
        _ => "[UNKNOWN]",
    };
    let message_type = match message_type {
        vk::DebugUtilsMessageTypeFlagsEXT::GENERAL => "[GENERAL]",
        vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE => "[PERFORMACE]",
        vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION => "[VALIDATION]",
        _ => "[UNKNOWN]",
    };
    let message = CStr::from_ptr((*data).p_message);
    println!("[Debug]{}{}{:?}", message_severity, message_type, message);
    vk::FALSE
}

pub fn required_layers() -> Vec<&'static CStr> {
    REQUIRED_VALIDATION_LAYERS
        .iter()
        .map(|&layer| CStr::from_bytes_with_nul(layer).unwrap())
        .collect()
}

pub fn required_extensions() -> Vec<&'static CStr> {
    vec![ext::DebugUtils::name()]
}

impl MessengerBuilder {
    pub fn new() -> Self {
        let info = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE | {
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION
            },
            pfn_user_callback: Some(vulkan_debug_callback),
            ..Default::default()
        };
        Self(info)
    }

    pub fn build(&self, entry: &Entry, instance: &Instance) -> StaticResult<Messenger> {
        let loader = ext::DebugUtils::new(entry, instance);
        let messenger = unsafe { loader.create_debug_utils_messenger(&self.0, None)? };
        Ok(Messenger { loader, messenger })
    }
}

impl AsMut<vk::DebugUtilsMessengerCreateInfoEXT> for MessengerBuilder {
    fn as_mut(&mut self) -> &mut vk::DebugUtilsMessengerCreateInfoEXT {
        &mut self.0
    }
}

impl Drop for Messenger {
    fn drop(&mut self) {
        unsafe {
            self.loader
                .destroy_debug_utils_messenger(self.messenger, None)
        };
    }
}
