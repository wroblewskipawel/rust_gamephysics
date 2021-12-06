use super::Device;
use crate::{math::types::Matrix4, renderer::mesh::Vertex};
use ash::{self, prelude::VkResult, vk};
use bytemuck;
use std::mem::size_of;

pub const CAMERA_PUSH_OFFSET: u32 = 0 * size_of::<Matrix4>() as u32;
pub const WORLD_PUSH_OFFSET: u32 = 1 * size_of::<Matrix4>() as u32;

pub(super) struct Layout {
    pub pipeline_layout: vk::PipelineLayout,
    pub vertex_bindings: [vk::VertexInputBindingDescription; 1],
    pub vertex_attribs: [vk::VertexInputAttributeDescription; 5],
}

impl Device {
    pub(super) fn create_layout(device: &ash::Device) -> VkResult<Layout> {
        let vertex = Vertex::default();

        let vertex_bindings = [vk::VertexInputBindingDescription {
            input_rate: vk::VertexInputRate::VERTEX,
            stride: size_of::<Vertex>() as u32,
            binding: 0,
        }];

        let vertex_attribs = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                offset: bytemuck::offset_of!(vertex, Vertex, pos) as u32,
                format: vk::Format::R32G32B32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                offset: bytemuck::offset_of!(vertex, Vertex, norm) as u32,
                format: vk::Format::R32G32B32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                offset: bytemuck::offset_of!(vertex, Vertex, tang) as u32,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                offset: bytemuck::offset_of!(vertex, Vertex, color) as u32,
                format: vk::Format::R32G32B32A32_SFLOAT,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 4,
                offset: bytemuck::offset_of!(vertex, Vertex, tex) as u32,
                format: vk::Format::R32G32_SFLOAT,
            },
        ];

        let push_ranges = [vk::PushConstantRange {
            stage_flags: vk::ShaderStageFlags::VERTEX,
            size: 2 * size_of::<Matrix4>() as u32,
            offset: 0,
        }];

        let pipeline_layout = unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::builder().push_constant_ranges(&push_ranges),
                None,
            )?
        };

        Ok(Layout {
            pipeline_layout,
            vertex_attribs,
            vertex_bindings,
        })
    }

    pub(super) fn destory_layout(device: &ash::Device, layout: &mut Layout) {
        unsafe {
            device.destroy_pipeline_layout(layout.pipeline_layout, None);
        }
    }
}
