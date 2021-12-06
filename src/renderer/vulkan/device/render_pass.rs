use std::ops::SubAssign;

use crate::utils::ScopedResult;
use ash::{prelude::VkResult, vk};

use super::{Device, PhysicalDeviceConfig};

impl Device {
    pub(super) fn create_render_pass(
        device: &ash::Device,
        config: &PhysicalDeviceConfig,
    ) -> VkResult<vk::RenderPass> {
        let attachments = [
            vk::AttachmentDescription {
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                initial_layout: vk::ImageLayout::UNDEFINED,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                format: config.depth_format,
                samples: vk::SampleCountFlags::TYPE_1,
                ..Default::default()
            },
            vk::AttachmentDescription {
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                initial_layout: vk::ImageLayout::UNDEFINED,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::STORE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                format: config.surface_format.format,
                samples: vk::SampleCountFlags::TYPE_1,
                ..Default::default()
            },
        ];

        let depth_reference = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        };

        let color_reference = [vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ..Default::default()
        }];

        let subpasses = [vk::SubpassDescription::builder()
            .color_attachments(&color_reference)
            .depth_stencil_attachment(&depth_reference)
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .build()];

        let dependencies = [
            vk::SubpassDependency {
                src_subpass: vk::SUBPASS_EXTERNAL,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ,
                dst_subpass: 0,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
                ..Default::default()
            },
            vk::SubpassDependency {
                src_subpass: 0,
                src_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                src_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                dst_subpass: vk::SUBPASS_EXTERNAL,
                dst_stage_mask: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                dst_access_mask: vk::AccessFlags::COLOR_ATTACHMENT_READ,
                ..Default::default()
            },
        ];

        unsafe {
            device.create_render_pass(
                &vk::RenderPassCreateInfo::builder()
                    .attachments(&attachments)
                    .dependencies(&dependencies)
                    .subpasses(&subpasses),
                None,
            )
        }
    }
}
