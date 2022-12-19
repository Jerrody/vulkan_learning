use ash::vk;
use smallvec::SmallVec;
use tracing::info;
use track::Context;

use crate::engine::{asset_system::mesh::VertexDescription, renderer::context::depth};

pub struct PipelineHandle {
    pub pipeline: vk::Pipeline,
    pub pipeline_layout: vk::PipelineLayout,
}

impl PipelineHandle {
    pub fn new(
        device: &ash::Device,
        shader_handle: &super::shader::ShaderHandle,
        format: vk::Format,
        image_extent: vk::Extent2D,
    ) -> track::Result<Self> {
        info!("Preparing Graphics Pipeline");

        let shader_stages = Self::create_shader_stages(&shader_handle.shader_modules);

        let assembly_info = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);

        let viewports = [vk::Viewport::default()
            .height(image_extent.height as f32)
            .width(image_extent.width as f32)
            .max_depth(1.0)];
        let scissors = [vk::Rect2D {
            offset: vk::Offset2D { x: 0, y: 0 },
            extent: image_extent,
        }];

        let viewport_state = vk::PipelineViewportStateCreateInfo::default()
            .viewports(&viewports)
            .scissors(&scissors);

        let vertex_description = VertexDescription::new();
        let bindings = [vertex_description.binding];

        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&bindings)
            .vertex_attribute_descriptions(&vertex_description.attributes);

        let rasterization_info = vk::PipelineRasterizationStateCreateInfo::default()
            .cull_mode(vk::CullModeFlags::FRONT)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .line_width(1.0)
            .polygon_mode(vk::PolygonMode::FILL);

        let multisample_state = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let color_blending_state = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)];

        let color_blend_attachment =
            vk::PipelineColorBlendStateCreateInfo::default().attachments(&color_blending_state);

        let depth_stencil_state_info = vk::PipelineDepthStencilStateCreateInfo::default()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .max_depth_bounds(1.0);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::default();
        let pipeline_layout =
            unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }.track()?;

        let color_attachment_infos = [format];
        let mut pipeline_rendering_info = vk::PipelineRenderingCreateInfo::default()
            .color_attachment_formats(&color_attachment_infos)
            .depth_attachment_format(depth::DepthBuffer::DEPTH_BUFFER_FORMAT);

        let pipeline_infos = [vk::GraphicsPipelineCreateInfo::default()
            .vertex_input_state(&vertex_input_state)
            .depth_stencil_state(&depth_stencil_state_info)
            .stages(&shader_stages)
            .input_assembly_state(&assembly_info)
            .viewport_state(&viewport_state)
            .rasterization_state(&rasterization_info)
            .multisample_state(&multisample_state)
            .color_blend_state(&color_blend_attachment)
            .layout(pipeline_layout)
            .push_next(&mut pipeline_rendering_info)];

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(vk::PipelineCache::null(), &pipeline_infos, None)
                .unwrap()
                .remove(Default::default())
        };

        Ok(Self {
            pipeline,
            pipeline_layout,
        })
    }

    #[inline]
    fn create_shader_stages<'a>(
        shader_modules: &SmallVec<[(vk::ShaderModule, vk::ShaderStageFlags); 2]>,
    ) -> SmallVec<[vk::PipelineShaderStageCreateInfo<'a>; 2]> {
        shader_modules
            .iter()
            .map(|(shader_module, shader_stage)| {
                vk::PipelineShaderStageCreateInfo::default()
                    .name(super::shader::SHADER_ENTRY_NAME)
                    .stage(*shader_stage)
                    .module(*shader_module)
            })
            .collect()
    }
}
