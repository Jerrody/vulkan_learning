use std::{mem::ManuallyDrop, path::Path};

use ash::{prelude::VkResult, vk};
use tracing::info;
use track::Context;

use crate::profile;

use super::asset_system::mesh;

mod context;
mod resources;

pub struct Renderer {
    context: context::Context,
    resources: ManuallyDrop<resources::Resources>,
    render_fence: vk::Fence,
    render_semaphore: vk::Semaphore,
    present_semaphore: vk::Semaphore,
}

impl Renderer {
    pub unsafe fn new(window: &winit::window::Window) -> track::Result<Self> {
        info!("Initializing Vulkan");
        let (context, resources) = context::Context::new(window).track()?;

        let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
        let render_fence = context.create_fence(&fence_info).track()?;

        let semaphore_info = vk::SemaphoreCreateInfo::default();
        let render_semaphore = context.create_semaphore(&semaphore_info).track()?;
        let present_semaphore = context.create_semaphore(&semaphore_info).track()?;

        info!("Rensderer prepared");

        Ok(Self {
            context,
            resources: ManuallyDrop::new(resources),
            render_fence,
            render_semaphore,
            present_semaphore,
        })
    }

    #[inline(always)]
    pub unsafe fn draw(
        &self,
        mesh: &mesh::Mesh,
        // meshes: &Vec<[mesh::Mesh; super::Engine::DEFAULT_STACK_BASED_MESHES_SIZE]>,
    ) -> VkResult<()> {
        profile!("Draw Triangle");

        let device = &self.context.device_handle.device;
        let queue_family_index = self.context.device_handle.queue_family_index;

        self.context.reset_fences(&[self.render_fence])?;
        self.context.reset_commmand_buffers()?;

        let (image_index, image, image_view) = self
            .context
            .get_image(self.present_semaphore, vk::Fence::null())?;

        let command_buffer = self.context.command.command_buffers[0];
        let command_buffer_begin_info = vk::CommandBufferBeginInfo::default()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        device.begin_command_buffer(command_buffer, &command_buffer_begin_info)?;

        let memory_barriers = [
            vk::ImageMemoryBarrier2::default()
                .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                .new_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .src_queue_family_index(queue_family_index)
                .dst_queue_family_index(queue_family_index)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                }),
            vk::ImageMemoryBarrier2::default()
                .src_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags2::COLOR_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags2::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags2::NONE)
                .old_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                .new_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .src_queue_family_index(queue_family_index)
                .dst_queue_family_index(queue_family_index)
                .image(image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                }),
            vk::ImageMemoryBarrier2::default()
                .src_stage_mask(
                    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                        | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                )
                .dst_stage_mask(
                    vk::PipelineStageFlags2::EARLY_FRAGMENT_TESTS
                        | vk::PipelineStageFlags2::LATE_FRAGMENT_TESTS,
                )
                .dst_access_mask(vk::AccessFlags2::DEPTH_STENCIL_ATTACHMENT_WRITE)
                .new_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
                .src_queue_family_index(queue_family_index)
                .dst_queue_family_index(queue_family_index)
                .image(self.context.depth_buffer.image)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::DEPTH,
                    level_count: 1,
                    layer_count: 1,
                    ..Default::default()
                }),
        ];

        self.context.set_pipeline_barrier(
            command_buffer,
            &vk::DependencyInfo::default().image_memory_barriers(&memory_barriers),
        );

        let color_attachment_infos = [vk::RenderingAttachmentInfo::default()
            .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
            .image_view(image_view)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0.5, 0.5, 0.5, 1.0],
                },
            })];
        let depth_attachment_info = vk::RenderingAttachmentInfo::default()
            .image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL)
            .image_view(self.context.depth_buffer.image_view)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .clear_value(vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                },
            });

        let rendering_info = vk::RenderingInfo::default()
            .color_attachments(&color_attachment_infos)
            .depth_attachment(&depth_attachment_info)
            .render_area(vk::Rect2D {
                extent: self.context.swapchain_handle.image_extent,
                offset: Default::default(),
            })
            .layer_count(1);

        device.cmd_begin_rendering(command_buffer, &rendering_info);

        device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.context.pipeline_handle.pipeline,
        );

        (0..self.resources.allocated_buffers_len()).for_each(|_| {
            self.resources.bind_buffers_per_draw(device, command_buffer);
            device.cmd_draw_indexed(command_buffer, mesh.indices.len() as u32, 1, 0, 0, 0);
        });

        device.cmd_end_rendering(command_buffer);
        device.end_command_buffer(command_buffer)?;

        let command_buffers = [command_buffer];
        let signal_semaphores = [self.render_semaphore];
        let wait_semaphores = [self.present_semaphore];

        let submit_info = vk::SubmitInfo::default()
            .command_buffers(&command_buffers)
            .signal_semaphores(&signal_semaphores)
            .wait_semaphores(&wait_semaphores)
            .wait_dst_stage_mask(&[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT]);

        let queue_graphics = self.context.device_handle.queue_graphics;
        device.queue_submit(queue_graphics, &[submit_info], self.render_fence)?;

        let swapchains = [self.context.swapchain_handle.swapchain];
        let image_indices = [image_index as u32];
        let present_info = vk::PresentInfoKHR::default()
            .swapchains(&swapchains)
            .wait_semaphores(&signal_semaphores)
            .image_indices(&image_indices);

        self.context
            .swapchain_handle
            .swapchain_loader
            .queue_present(queue_graphics, &present_info)?;

        Ok(())
    }

    #[inline(always)]
    pub fn upload_asset<P: AsRef<Path> + std::fmt::Debug>(
        &mut self,
        path: P,
    ) -> track::Result<mesh::Mesh> {
        let mesh = mesh::Mesh::new(path).track()?;
        self.resources.uplaod_mesh(&mesh).track()?;

        Ok(mesh)
    }
}

// FIXME: Move into the another place.
impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            let context = &self.context;
            let device = &context.device_handle.device;
            device.device_wait_idle().unwrap();

            device.destroy_command_pool(context.command.command_pool, None);
            context
                .swapchain_handle
                .image_views
                .iter()
                .for_each(|image_view| {
                    device.destroy_image_view(*image_view, None);
                });
            device.destroy_image_view(context.depth_buffer.image_view, None);

            context
                .swapchain_handle
                .swapchain_loader
                .destroy_swapchain(context.swapchain_handle.swapchain, None);

            device.destroy_pipeline(context.pipeline_handle.pipeline, None);
            device.destroy_pipeline_layout(context.pipeline_handle.pipeline_layout, None);

            device.destroy_fence(self.render_fence, None);
            device.destroy_semaphore(self.present_semaphore, None);
            device.destroy_semaphore(self.render_semaphore, None);

            ManuallyDrop::drop(&mut self.resources);

            device.destroy_device(None);

            context
                .surface_handle
                .surface_loader
                .destroy_surface(context.surface_handle.surface, None);

            #[cfg(feature = "validation")]
            context
                .debug_handle
                .debug_loader
                .destroy_debug_utils_messenger(context.debug_handle.debug_utils, None);

            context.instance_handle.instance.destroy_instance(None);
        }
    }
}
