mod command;
mod debug;
mod depth;
mod device;
mod instance;
mod pipeline;
mod shader;
mod surface;
mod swapchain;

use ash::prelude::VkResult;
use ash::vk;
use track::Context as TrackContext; // Renamed the `Context`'s name due to name collision with backend's `Context`.

use self::device::DeviceHandle;
use self::surface::SurfaceHandle;
use self::swapchain::SwapchainHandle;

use super::resources;
pub struct Context {
    #[cfg(feature = "validation")]
    pub debug_handle: debug::DebugHandle,
    pub surface_handle: SurfaceHandle,
    pub device_handle: DeviceHandle,
    pub swapchain_handle: SwapchainHandle,
    pub pipeline_handle: pipeline::PipelineHandle,
    pub command: command::Command,
    pub instance_handle: instance::InstaceHandle,
    pub depth_buffer: depth::DepthBuffer,
}

// TODO: `debug!` message of every stage of Vulkan initialization
impl Context {
    pub fn new(window: &winit::window::Window) -> track::Result<(Self, resources::Resources)> {
        let instance_handle = instance::InstaceHandle::new(window).track()?;

        #[cfg(feature = "validation")]
        let debug_handle =
            debug::DebugHandle::new(&instance_handle.entry, &instance_handle.instance).track()?;

        let surface_handle =
            SurfaceHandle::new(&instance_handle.entry, &instance_handle.instance, window)
                .track()?;

        let device_handle =
            DeviceHandle::new(&instance_handle.instance, &surface_handle).track()?;

        let mut resources = resources::Resources::new(
            &instance_handle.instance,
            device_handle.physical_device,
            &device_handle.device,
        )
        .track()?;

        let swapchain_handle = swapchain::SwapchainHandle::new(
            &instance_handle.instance,
            &device_handle,
            &surface_handle,
            window,
        )
        .track()?;

        let depth_buffer = depth::DepthBuffer::new(
            &device_handle.device,
            &mut resources,
            vk::Extent3D {
                width: swapchain_handle.image_extent.width,
                height: swapchain_handle.image_extent.height,
                depth: 1,
            },
        )
        .track()?;

        let shader_handle = shader::ShaderHandle::new(&device_handle.device);

        let pipeline_handle = pipeline::PipelineHandle::new(
            &device_handle.device,
            &shader_handle,
            device_handle.surface_format.format,
            swapchain_handle.image_extent,
        )
        .track()?;

        shader_handle
            .shader_modules
            .iter()
            .for_each(|(shader_module, _)| {
                unsafe {
                    device_handle
                        .device
                        .destroy_shader_module(*shader_module, None)
                };
            });

        let command = command::Command::new(
            &device_handle.device,
            device_handle.queue_family_index,
            swapchain_handle.images.len() as u32,
        )
        .track()?;

        Ok((
            Self {
                instance_handle,
                #[cfg(feature = "validation")]
                debug_handle,
                surface_handle,
                device_handle,
                swapchain_handle,
                depth_buffer,
                pipeline_handle,
                command,
            },
            resources,
        ))
    }

    #[inline]
    pub unsafe fn create_fence(
        &self,
        fence_info: &vk::FenceCreateInfo,
    ) -> track::Result<vk::Fence> {
        self.device_handle
            .device
            .create_fence(fence_info, None)
            .track()
    }

    #[inline]
    pub unsafe fn create_semaphore(
        &self,
        semaphore_info: &vk::SemaphoreCreateInfo,
    ) -> track::Result<vk::Semaphore> {
        self.device_handle
            .device
            .create_semaphore(semaphore_info, None)
            .track()
    }

    #[inline(always)]
    pub unsafe fn reset_fences(&self, fences: &[vk::Fence]) -> VkResult<()> {
        self.device_handle
            .device
            .wait_for_fences(fences, true, u64::MAX)?;

        self.device_handle.device.reset_fences(fences)
    }

    #[inline(always)]
    pub unsafe fn reset_commmand_buffers(&self) -> VkResult<()> {
        self.device_handle.device.reset_command_pool(
            self.command.command_pool,
            vk::CommandPoolResetFlags::empty(),
        )
    }

    #[inline(always)]
    pub unsafe fn get_image(
        &self,
        semaphore: vk::Semaphore,
        fence: vk::Fence,
    ) -> VkResult<(usize, vk::Image, vk::ImageView)> {
        let next_image_index = self
            .swapchain_handle
            .swapchain_loader
            .acquire_next_image(self.swapchain_handle.swapchain, u64::MAX, semaphore, fence)?
            .0 as usize;

        Ok((
            next_image_index,
            self.swapchain_handle.images[next_image_index],
            self.swapchain_handle.image_views[next_image_index],
        ))
    }

    #[inline(always)]
    pub unsafe fn set_pipeline_barrier(
        &self,
        command_buffer: vk::CommandBuffer,
        dependency_info: &vk::DependencyInfo,
    ) {
        self.device_handle
            .device
            .cmd_pipeline_barrier2(command_buffer, dependency_info)
    }
}
