use super::{Device, Layout, Swapchain};
use ash::{self, prelude::VkResult, vk};
use std::{ffi::CStr, fs::File, io::Read, path::Path};

const VERTEX_SHADER_PATH: &'static str = "shaders/spv/vert.spv";
const FRAGMENT_SHADER_PATH: &'static str = "shaders/spv/frag.spv";
pub(super) struct Pipeline {
    pub pipeline: vk::Pipeline,
}

impl Device {
    pub(super) fn create_pipeline(
        device: &ash::Device,
        layout: &Layout,
        swapchain: &Swapchain,
        render_pass: vk::RenderPass,
    ) -> VkResult<Pipeline> {
        let shaders = Device::load_shaders(device)?;

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[vk::GraphicsPipelineCreateInfo::builder()
                        .color_blend_state(
                            &vk::PipelineColorBlendStateCreateInfo::builder().attachments(&[
                                vk::PipelineColorBlendAttachmentState::builder()
                                    .blend_enable(false)
                                    .color_blend_op(vk::BlendOp::ADD)
                                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                                    .alpha_blend_op(vk::BlendOp::ADD)
                                    .src_alpha_blend_factor(vk::BlendFactor::ONE)
                                    .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
                                    .color_write_mask(vk::ColorComponentFlags::all())
                                    .build(),
                            ]),
                        )
                        .depth_stencil_state(
                            &vk::PipelineDepthStencilStateCreateInfo::builder()
                                .depth_write_enable(true) //TODO: ENABLE
                                .depth_test_enable(true) //TODO: ENABLE
                                .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL),
                        )
                        .input_assembly_state(
                            &vk::PipelineInputAssemblyStateCreateInfo::builder()
                                .topology(vk::PrimitiveTopology::TRIANGLE_LIST),
                        )
                        .multisample_state(
                            &vk::PipelineMultisampleStateCreateInfo::builder()
                                .rasterization_samples(vk::SampleCountFlags::TYPE_1),
                        )
                        .rasterization_state(
                            &vk::PipelineRasterizationStateCreateInfo::builder()
                                .rasterizer_discard_enable(false)
                                .polygon_mode(vk::PolygonMode::FILL)
                                .line_width(1.0f32)
                                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                                .cull_mode(vk::CullModeFlags::BACK), //TODO: ENABLE
                        )
                        .render_pass(render_pass)
                        .stages(&shaders)
                        .subpass(0)
                        .vertex_input_state(
                            &vk::PipelineVertexInputStateCreateInfo::builder()
                                .vertex_binding_descriptions(&layout.vertex_bindings)
                                .vertex_attribute_descriptions(&layout.vertex_attribs),
                        )
                        .viewport_state(
                            &vk::PipelineViewportStateCreateInfo::builder()
                                .viewports(&[vk::Viewport {
                                    width: swapchain.extent.width as f32,
                                    height: -(swapchain.extent.height as f32),
                                    x: 0.0 as f32,
                                    y: swapchain.extent.height as f32,
                                    min_depth: 0.0f32,
                                    max_depth: 1.0f32,
                                }])
                                .scissors(&[vk::Rect2D {
                                    offset: vk::Offset2D { x: 0, y: 0 },
                                    extent: swapchain.extent,
                                }]),
                        )
                        .layout(layout.pipeline_layout)
                        .build()],
                    None,
                )
                .map_err(|(_, err)| err)?[0]
        };

        for shader in shaders {
            unsafe {
                device.destroy_shader_module(shader.module, None);
            }
        }

        Ok(Pipeline { pipeline })
    }

    fn shader_entry_point() -> &'static CStr {
        unsafe { CStr::from_bytes_with_nul_unchecked(b"main\0") }
    }

    fn load_shader_module(
        device: &ash::Device,
        stage: vk::ShaderStageFlags,
        path: &Path,
    ) -> VkResult<vk::PipelineShaderStageCreateInfo> {
        let reader = File::open(path).unwrap();
        let bytes: Vec<_> = reader.bytes().filter_map(|b| b.ok()).collect();
        let module = unsafe {
            device.create_shader_module(
                &vk::ShaderModuleCreateInfo {
                    p_code: bytes.as_ptr() as *const u32,
                    code_size: bytes.len(),
                    ..Default::default()
                },
                None,
            )?
        };
        Ok(vk::PipelineShaderStageCreateInfo::builder()
            .module(module)
            .stage(stage)
            .name(Device::shader_entry_point())
            .build())
    }

    fn load_shaders(device: &ash::Device) -> VkResult<Vec<vk::PipelineShaderStageCreateInfo>> {
        let vertex = Device::load_shader_module(
            device,
            vk::ShaderStageFlags::VERTEX,
            Path::new(VERTEX_SHADER_PATH),
        )?;
        let framgnet = Device::load_shader_module(
            device,
            vk::ShaderStageFlags::FRAGMENT,
            Path::new(FRAGMENT_SHADER_PATH),
        )?;
        Ok(vec![vertex, framgnet])
    }

    pub(super) fn destory_pipeline(device: &ash::Device, pipeline: &mut Pipeline) {
        unsafe {
            device.destroy_pipeline(pipeline.pipeline, None);
        }
    }
}
