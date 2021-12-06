use crate::renderer::Mesh;
use crate::{math::types::Matrix4, renderer::MeshHandle};
use ash::{prelude::VkResult, vk, Instance};

mod buffer;
mod command;
mod layout;
mod pipeline;
mod render_pass;
mod swapchain;

use buffer::MeshData;
use command::CommandType;
use layout::Layout;
use pipeline::Pipeline;
pub use swapchain::Frame;
use swapchain::Swapchain;

use std::{
    collections::HashSet, ffi::CStr, iter::FromIterator, mem::size_of, os::raw::c_char, slice,
};

use super::Surface;
use crate::utils::StaticResult;

const PREFERRED_SURFACE_FORMATS: &'static [vk::Format] =
    &[vk::Format::R8G8B8A8_UNORM, vk::Format::B8G8R8A8_UNORM];
const PREFERRED_DEPTH_FORMATS: &'static [vk::Format] = &[
    vk::Format::D32_SFLOAT,
    vk::Format::D24_UNORM_S8_UINT,
    vk::Format::D16_UNORM,
    vk::Format::D16_UNORM_S8_UINT,
];

pub struct Queues {
    pub graphics: vk::Queue,
    pub compute: vk::Queue,
    pub transfer: vk::Queue,
    pub present: vk::Queue,
}

struct CommandPools {
    graphics: vk::CommandPool,
    compute: vk::CommandPool,
    transfer: vk::CommandPool,
}

pub struct QueueFamilies {
    pub graphics: u32,
    pub compute: u32,
    pub transfer: u32,
    pub present: u32,
}

pub struct PhysicalDeviceConfig {
    pub device: vk::PhysicalDevice,
    pub queue_families: QueueFamilies,
    pub depth_format: vk::Format,
    pub present_mode: vk::PresentModeKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub enabled_features: vk::PhysicalDeviceFeatures,
    pub properties: vk::PhysicalDeviceProperties,
    pub memory_properties: vk::PhysicalDeviceMemoryProperties,
}

pub struct Device {
    device: ash::Device,
    queues: Queues,
    command_pools: CommandPools,
    render_pass: vk::RenderPass,
    swapchain: Swapchain,
    layout: Layout,
    pipeline: Pipeline,
    config: PhysicalDeviceConfig,
    mesh_data: MeshData,
}

impl Device {
    pub(super) fn new(
        instance: &Instance,
        surface: &Surface,
        meshes: &[Mesh],
    ) -> StaticResult<Self> {
        let devices = unsafe { instance.enumerate_physical_devices()? };
        let config = devices
            .into_iter()
            .find_map(|device| Device::is_suitable(device, instance, surface))
            .ok_or(format!("Failed to pick suitable physical device"))?;

        println!("Chosen Vulkan physical device name: [{}]", unsafe {
            CStr::from_ptr(&config.properties.device_name as *const c_char)
                .to_str()
                .unwrap_or("UTF8 PARSE ERROR")
        });

        let queue_familites = &config.queue_families;
        let queue_infos: Vec<_> = HashSet::<u32>::from_iter([
            queue_familites.graphics,
            queue_familites.compute,
            queue_familites.transfer,
            queue_familites.present,
        ])
        .into_iter()
        .map(|queue_family_index| vk::DeviceQueueCreateInfo {
            queue_family_index,
            queue_count: 1,
            p_queue_priorities: &1.0f32 as *const f32,
            ..Default::default()
        })
        .collect();

        let required_extensions: Vec<_> = Device::required_extensions()
            .iter()
            .map(|ext| ext.as_ptr())
            .collect();

        let device = unsafe {
            instance.create_device(
                config.device,
                &vk::DeviceCreateInfo::builder()
                    .queue_create_infos(&queue_infos)
                    .enabled_extension_names(&required_extensions)
                    .enabled_features(&config.enabled_features),
                None,
            )?
        };

        let queues = unsafe {
            Queues {
                graphics: device.get_device_queue(config.queue_families.graphics, 0),
                compute: device.get_device_queue(config.queue_families.compute, 0),
                transfer: device.get_device_queue(config.queue_families.transfer, 0),
                present: device.get_device_queue(config.queue_families.present, 0),
            }
        };

        let command_pools = unsafe {
            CommandPools {
                graphics: device.create_command_pool(
                    &vk::CommandPoolCreateInfo::builder()
                        .queue_family_index(config.queue_families.graphics)
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                    None,
                )?,
                compute: device.create_command_pool(
                    &vk::CommandPoolCreateInfo::builder()
                        .queue_family_index(config.queue_families.compute)
                        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                    None,
                )?,
                transfer: device.create_command_pool(
                    &vk::CommandPoolCreateInfo::builder()
                        .queue_family_index(config.queue_families.transfer)
                        .flags(vk::CommandPoolCreateFlags::TRANSIENT),
                    None,
                )?,
            }
        };

        let render_pass = Device::create_render_pass(&device, &config)?;
        let swapchain =
            Device::create_swapchain(instance, &device, &config, surface.handle, render_pass)?;
        let layout = Device::create_layout(&device)?;
        let pipeline = Device::create_pipeline(&device, &layout, &swapchain, render_pass)?;
        let mesh_data = Device::load_mesh_data(&device, &config, &command_pools, &queues, meshes)?;

        Ok(Self {
            device,
            queues,
            command_pools,
            render_pass,
            swapchain,
            layout,
            pipeline,
            config,
            mesh_data,
        })
    }

    fn is_suitable(
        device: vk::PhysicalDevice,
        instance: &Instance,
        surface: &Surface,
    ) -> Option<PhysicalDeviceConfig> {
        let properties = unsafe { instance.get_physical_device_properties(device) };
        match properties.device_type {
            vk::PhysicalDeviceType::INTEGRATED_GPU => {}
            vk::PhysicalDeviceType::DISCRETE_GPU => {}
            _ => return None,
        };
        Device::extension_supported(device, instance)?;
        let enabled_features = Device::features_supported(device, instance)?;
        let present_mode = surface
            .device_present_modes(device)
            .ok()?
            .into_iter()
            .find(|&mode| mode == vk::PresentModeKHR::MAILBOX)
            .unwrap_or(vk::PresentModeKHR::FIFO);
        let queue_families = Device::queue_families(device, instance, surface)?;
        let surface_formats = surface.device_surface_formats(device).ok()?;
        let &surface_format = surface_formats
            .iter()
            .find(|format| PREFERRED_SURFACE_FORMATS.contains(&format.format))
            .unwrap_or(surface_formats.first().unwrap());
        let surface_capabilities = surface.device_surface_capabilities(device).ok()?;
        let depth_format = Device::supported_image_format(
            instance,
            device,
            PREFERRED_DEPTH_FORMATS,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )?;
        let memory_properties = unsafe { instance.get_physical_device_memory_properties(device) };

        Some(PhysicalDeviceConfig {
            device,
            queue_families,
            surface_format,
            present_mode,
            surface_capabilities,
            depth_format,
            memory_properties,
            enabled_features,
            properties,
        })
    }

    fn supported_image_format(
        instance: &Instance,
        device: vk::PhysicalDevice,
        formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> Option<vk::Format> {
        for format in formats {
            let properties =
                unsafe { instance.get_physical_device_format_properties(device, *format) };
            match tiling {
                vk::ImageTiling::OPTIMAL => {
                    if properties.optimal_tiling_features.contains(features) {
                        return Some(*format);
                    }
                }
                vk::ImageTiling::LINEAR => {
                    if properties.linear_tiling_features.contains(features) {
                        return Some(*format);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn queue_families(
        device: vk::PhysicalDevice,
        instance: &Instance,
        surface: &Surface,
    ) -> Option<QueueFamilies> {
        let mut graphics = None;
        let mut compute = None;
        let mut transfer = None;
        let mut present = None;
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(device) };
        for (family, properties) in queue_families.iter().enumerate() {
            if graphics.is_none() && properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                graphics = Some(family as u32);
            }
            if properties.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                if graphics == compute || compute.is_none() {
                    compute = Some(family as u32);
                }
            }
            if properties.queue_flags.contains(vk::QueueFlags::TRANSFER) {
                if transfer == graphics || transfer == compute || transfer.is_none() {
                    transfer = Some(family as u32);
                }
            }
            if let Ok(supported) = surface.device_surface_support(device, family as u32) {
                if supported {
                    present = Some(family as u32)
                }
            }
        }
        Some(QueueFamilies {
            graphics: graphics?,
            compute: compute?,
            transfer: transfer?,
            present: present?,
        })
    }

    fn required_extensions() -> Vec<&'static CStr> {
        vec![ash::extensions::khr::Swapchain::name()]
    }

    fn extension_supported(device: vk::PhysicalDevice, instance: &Instance) -> Option<()> {
        let supported_extensions = unsafe {
            instance
                .enumerate_device_extension_properties(device)
                .ok()?
        };
        for req in Device::required_extensions() {
            supported_extensions.iter().find(
                |ext| unsafe { CStr::from_ptr(&ext.extension_name as *const c_char) } == req,
            )?;
        }
        Some(())
    }

    fn required_features() -> vk::PhysicalDeviceFeatures {
        vk::PhysicalDeviceFeatures {
            sampler_anisotropy: vk::TRUE,
            ..Default::default()
        }
    }

    fn features_supported(
        device: vk::PhysicalDevice,
        instance: &Instance,
    ) -> Option<vk::PhysicalDeviceFeatures> {
        let required = Device::required_features();
        let supported = unsafe { instance.get_physical_device_features(device) };
        let num_features = size_of::<vk::PhysicalDeviceFeatures>() / size_of::<vk::Bool32>();
        let required_slice = unsafe {
            slice::from_raw_parts::<vk::Bool32>(
                std::mem::transmute::<*const vk::PhysicalDeviceFeatures, *const vk::Bool32>(
                    &required,
                ),
                num_features,
            )
        };
        let supported_slice = unsafe {
            slice::from_raw_parts::<vk::Bool32>(
                std::mem::transmute::<*const vk::PhysicalDeviceFeatures, *const vk::Bool32>(
                    &supported,
                ),
                num_features,
            )
        };
        for (&sup, &req) in supported_slice.iter().zip(required_slice) {
            if req == vk::TRUE && sup != req {
                return None;
            }
        }
        Some(required)
    }

    fn memory_type_index(
        config: &PhysicalDeviceConfig,
        types: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> Option<u32> {
        for i in 0..config.memory_properties.memory_type_count {
            let mem = config.memory_properties.memory_types[i as usize];
            if 1 << i & types != 0 && mem.property_flags.contains(properties) {
                return Some(i as u32);
            }
        }
        None
    }

    pub fn begin_frame(&mut self, camera_matrix: &Matrix4) -> VkResult<Frame> {
        let frame = self.swapchain.acquire_image(&self.device)?;

        unsafe {
            self.device.cmd_begin_render_pass(
                frame.command,
                &vk::RenderPassBeginInfo::builder()
                    .render_pass(self.render_pass)
                    .framebuffer(frame.framebuffer)
                    .clear_values(&[
                        vk::ClearValue {
                            depth_stencil: vk::ClearDepthStencilValue {
                                depth: 1.0f32,
                                stencil: 0u32,
                            },
                        },
                        vk::ClearValue {
                            color: vk::ClearColorValue {
                                float32: [0.1f32, 0.1f32, 0.1f32, 1.0f32],
                            },
                        },
                    ])
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: self.swapchain.extent,
                    }),
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_bind_pipeline(
                frame.command,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );

            self.device.cmd_push_constants(
                frame.command,
                self.layout.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                layout::CAMERA_PUSH_OFFSET,
                bytemuck::bytes_of(camera_matrix),
            )
        }
        Device::bind_buffers(&self.device, frame.command, &self.mesh_data);
        Ok(frame)
    }

    pub fn draw(&mut self, frame: &Frame, mesh: MeshHandle, world: &Matrix4) {
        let offsets = &self.mesh_data.mesh_offsets[mesh.0];
        unsafe {
            self.device.cmd_push_constants(
                frame.command,
                self.layout.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                layout::WORLD_PUSH_OFFSET,
                bytemuck::bytes_of(world),
            );
            self.device.cmd_draw_indexed(
                frame.command,
                offsets.index_count as u32,
                1,
                offsets.index_offset as u32,
                offsets.vertex_offset as i32,
                0,
            );
        }
    }

    pub fn end_frame(&mut self, frame: Frame) -> VkResult<()> {
        unsafe {
            self.device.cmd_end_render_pass(frame.command);
            self.device.end_command_buffer(frame.command)?;
            self.device.queue_submit(
                self.queues.graphics,
                &[vk::SubmitInfo::builder()
                    .command_buffers(&[frame.command])
                    .signal_semaphores(&[frame.draw_finished])
                    .wait_semaphores(&[frame.draw_ready])
                    .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT])
                    .build()],
                frame.available,
            )?;
        }
        let _suboptimal = self.swapchain.present_image(frame, self.queues.present)?;
        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().unwrap();
        }
        Device::destory_pipeline(&self.device, &mut self.pipeline);
        Device::destory_layout(&self.device, &mut self.layout);
        Device::destory_mesh_data(&self.device, &mut self.mesh_data);
        Device::destroy_swapchain(&self.device, &mut self.swapchain);
        unsafe {
            self.device
                .destroy_command_pool(self.command_pools.graphics, None);
            self.device
                .destroy_command_pool(self.command_pools.compute, None);
            self.device
                .destroy_command_pool(self.command_pools.transfer, None);
            self.device.destroy_render_pass(self.render_pass, None);
            self.device.destroy_device(None)
        };
    }
}
