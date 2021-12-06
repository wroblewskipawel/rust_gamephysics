use std::{collections::HashSet, iter::FromIterator};

use ash::{extensions::khr, prelude::VkResult, vk};

use crate::renderer::vulkan::device;

use super::{Device, PhysicalDeviceConfig};

struct DepthBuffer {
    memory: vk::DeviceMemory,
    image: vk::Image,
    view: vk::ImageView,
}

pub(super) struct Swapchain {
    pub(super) extent: vk::Extent2D,
    images: Vec<vk::Image>,
    views: Vec<vk::ImageView>,
    depth_buffer: DepthBuffer,
    framebuffers: Vec<vk::Framebuffer>,
    image_available: Vec<vk::Fence>,
    image_draw_ready: Vec<vk::Semaphore>,
    image_draw_finished: Vec<vk::Semaphore>,
    pool: vk::CommandPool,
    command_buffers: Vec<vk::CommandBuffer>,
    loader: khr::Swapchain,
    handle: vk::SwapchainKHR,
    frame: usize,
}

pub struct Frame {
    pub(super) command: vk::CommandBuffer,
    pub(super) framebuffer: vk::Framebuffer,
    pub(super) available: vk::Fence,
    pub(super) draw_ready: vk::Semaphore,
    pub(super) draw_finished: vk::Semaphore,
    pub(super) image_index: u32,
}

impl Swapchain {
    pub(super) fn acquire_image(&mut self, device: &ash::Device) -> VkResult<Frame> {
        let mut state = Frame {
            command: self.command_buffers[self.frame],
            draw_ready: self.image_draw_ready[self.frame],
            draw_finished: self.image_draw_finished[self.frame],
            framebuffer: vk::Framebuffer::null(),
            available: vk::Fence::null(),
            image_index: 0,
        };
        unsafe {
            let (image_index, _suboptimal) = self.loader.acquire_next_image(
                self.handle,
                u64::MAX,
                state.draw_ready,
                vk::Fence::null(),
            )?;
            state.image_index = image_index;
            state.available = self.image_available[image_index as usize];
            state.framebuffer = self.framebuffers[image_index as usize];
            device.wait_for_fences(&[state.available], true, u64::MAX)?;
            device.reset_fences(&[state.available])?;
            device.begin_command_buffer(
                state.command,
                &vk::CommandBufferBeginInfo::builder()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            )?;
        }
        Ok(state)
    }

    pub(super) fn present_image(
        &mut self,
        state: Frame,
        present_queue: vk::Queue,
    ) -> VkResult<bool> {
        let suboptimal = unsafe {
            self.loader.queue_present(
                present_queue,
                &vk::PresentInfoKHR::builder()
                    .image_indices(&[state.image_index])
                    .swapchains(&[self.handle])
                    .wait_semaphores(&[state.draw_finished]),
            )?
        };
        self.frame = (self.frame + 1) % self.images.len();
        Ok(suboptimal)
    }
}

impl Device {
    pub(super) fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        config: &PhysicalDeviceConfig,
        surface: vk::SurfaceKHR,
        render_pass: vk::RenderPass,
    ) -> VkResult<Swapchain> {
        let loader = khr::Swapchain::new(instance, device);
        let capabilities = &config.surface_capabilities;
        let extent = vk::Extent2D {
            width: u32::clamp(
                capabilities.current_extent.width,
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: u32::clamp(
                capabilities.current_extent.height,
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        };
        let min_image_count = if capabilities.max_image_count == 0 {
            capabilities.min_image_count + 1
        } else {
            u32::min(
                capabilities.min_image_count + 1,
                capabilities.max_image_count,
            )
        };

        let queue_indices: Vec<_> = HashSet::<u32>::from_iter([
            config.queue_families.graphics,
            config.queue_families.present,
        ])
        .into_iter()
        .collect();

        let handle = unsafe {
            loader.create_swapchain(
                &vk::SwapchainCreateInfoKHR::builder()
                    .image_format(config.surface_format.format)
                    .image_color_space(config.surface_format.color_space)
                    .surface(surface)
                    .min_image_count(min_image_count)
                    .image_extent(extent)
                    .clipped(true)
                    .image_array_layers(1)
                    .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
                    .pre_transform(capabilities.current_transform)
                    .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                    .present_mode(config.present_mode)
                    .queue_family_indices(&queue_indices)
                    .image_sharing_mode(if queue_indices.len() == 1 {
                        vk::SharingMode::EXCLUSIVE
                    } else {
                        vk::SharingMode::CONCURRENT
                    }),
                None,
            )?
        };

        let images = unsafe { loader.get_swapchain_images(handle)? };

        let views =
            Device::create_swapchain_image_views(device, &images, config.surface_format.format)?;
        let depth_buffer =
            Device::create_swapchain_depth_buffer(device, config, &extent, &queue_indices)?;
        let framebuffers = Device::create_swapchain_framebuffers(
            device,
            &depth_buffer,
            &views,
            &extent,
            render_pass,
        )?;
        let (pool, command_buffers) = Device::create_swapchain_command_buffers(
            device,
            config.queue_families.graphics,
            images.len(),
        )?;
        let (image_available, image_draw_ready, image_draw_finished) =
            Device::create_swapchain_sync_primitives(device, images.len())?;

        Ok(Swapchain {
            loader,
            handle,
            extent,
            images,
            views,
            depth_buffer,
            pool,
            command_buffers,
            framebuffers,
            image_available,
            image_draw_ready,
            image_draw_finished,
            frame: 0,
        })
    }

    fn create_swapchain_image_views(
        device: &ash::Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> VkResult<Vec<vk::ImageView>> {
        images
            .iter()
            .map(|&image| unsafe {
                device.create_image_view(
                    &vk::ImageViewCreateInfo::builder()
                        .image(image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .components(vk::ComponentMapping::default())
                        .subresource_range(vk::ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            base_array_layer: 0,
                            level_count: 1,
                            layer_count: 1,
                        })
                        .format(format),
                    None,
                )
            })
            .collect()
    }

    fn create_swapchain_depth_buffer(
        device: &ash::Device,
        config: &PhysicalDeviceConfig,
        extent: &vk::Extent2D,
        queue_indices: &[u32],
    ) -> VkResult<DepthBuffer> {
        let image = unsafe {
            device.create_image(
                &vk::ImageCreateInfo::builder()
                    .array_layers(1)
                    .mip_levels(1)
                    .extent(vk::Extent3D {
                        width: extent.width,
                        height: extent.height,
                        depth: 1,
                    })
                    .format(config.depth_format)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .queue_family_indices(queue_indices)
                    .sharing_mode(if queue_indices.len() == 1 {
                        vk::SharingMode::EXCLUSIVE
                    } else {
                        vk::SharingMode::CONCURRENT
                    })
                    .usage(vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .image_type(vk::ImageType::TYPE_2D),
                None,
            )?
        };
        let requirements = unsafe { device.get_image_memory_requirements(image) };
        let mem_index = Device::memory_type_index(
            config,
            requirements.memory_type_bits,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        )
        .ok_or(vk::Result::ERROR_UNKNOWN)?;
        let memory = unsafe {
            device.allocate_memory(
                &vk::MemoryAllocateInfo::builder()
                    .allocation_size(requirements.size)
                    .memory_type_index(mem_index),
                None,
            )?
        };
        unsafe { device.bind_image_memory(image, memory, 0)? };
        let view = unsafe {
            device.create_image_view(
                &vk::ImageViewCreateInfo::builder()
                    .image(image)
                    .format(config.depth_format)
                    .components(vk::ComponentMapping::default())
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::DEPTH,
                        base_mip_level: 0,
                        level_count: 1,
                        base_array_layer: 0,
                        layer_count: 1,
                    }),
                None,
            )?
        };
        Ok(DepthBuffer {
            memory,
            image,
            view,
        })
    }

    fn create_swapchain_framebuffers(
        device: &ash::Device,
        depth_buffer: &DepthBuffer,
        views: &[vk::ImageView],
        extent: &vk::Extent2D,
        render_pass: vk::RenderPass,
    ) -> VkResult<Vec<vk::Framebuffer>> {
        views
            .iter()
            .map(|&view| unsafe {
                device.create_framebuffer(
                    &vk::FramebufferCreateInfo::builder()
                        .attachments(&[depth_buffer.view, view])
                        .layers(1)
                        .render_pass(render_pass)
                        .width(extent.width)
                        .height(extent.height),
                    None,
                )
            })
            .collect()
    }

    fn create_swapchain_sync_primitives(
        device: &ash::Device,
        count: usize,
    ) -> VkResult<(Vec<vk::Fence>, Vec<vk::Semaphore>, Vec<vk::Semaphore>)> {
        let image_available: Result<Vec<_>, _> = (0..count)
            .map(|_| unsafe {
                device.create_fence(
                    &vk::FenceCreateInfo::builder().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
            })
            .collect();
        let image_draw_ready: Result<Vec<_>, _> = (0..count)
            .map(|_| unsafe { device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None) })
            .collect();
        let image_draw_finished: Result<Vec<_>, _> = (0..count)
            .map(|_| unsafe {
                {
                    device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)
                }
            })
            .collect();
        Ok((image_available?, image_draw_ready?, image_draw_finished?))
    }

    fn create_swapchain_command_buffers(
        device: &ash::Device,
        queue_index: u32,
        count: usize,
    ) -> VkResult<(vk::CommandPool, Vec<vk::CommandBuffer>)> {
        unsafe {
            let pool = device.create_command_pool(
                &vk::CommandPoolCreateInfo::builder()
                    .queue_family_index(queue_index)
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER),
                None,
            )?;
            let buffers = device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::builder()
                    .command_pool(pool)
                    .command_buffer_count(count as u32),
            )?;
            Ok((pool, buffers))
        }
    }

    pub(super) fn destroy_swapchain(device: &ash::Device, swapchain: &mut Swapchain) {
        unsafe {
            for &framebuffer in &swapchain.framebuffers {
                device.destroy_framebuffer(framebuffer, None);
            }
            for &view in &swapchain.views {
                device.destroy_image_view(view, None);
            }
            device.destroy_image_view(swapchain.depth_buffer.view, None);
            device.destroy_image(swapchain.depth_buffer.image, None);
            device.free_memory(swapchain.depth_buffer.memory, None);
            for &semaphore in &swapchain.image_draw_finished {
                device.destroy_semaphore(semaphore, None);
            }
            for &semaphore in &swapchain.image_draw_ready {
                device.destroy_semaphore(semaphore, None);
            }
            for &fence in &swapchain.image_available {
                device.destroy_fence(fence, None);
            }
            swapchain.loader.destroy_swapchain(swapchain.handle, None);
            device.destroy_command_pool(swapchain.pool, None);
        }
    }
}
