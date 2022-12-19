use ash::vk;
use track::Context;

use self::buffer::Buffer;

mod buffer;
mod image;

#[derive(Default)]
pub struct Resources {
    allocator: vma::Allocator,
    allocated_buffers: buffer::AllocatedBuffers,
    allocated_images: Vec<image::Image>,
}

impl Resources {
    #[inline(always)]
    pub fn new(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        device: &ash::Device,
    ) -> track::Result<Self> {
        let allocator =
            unsafe { vma::create_allocator(instance, physical_device, device, None).track()? };

        Ok(Self {
            allocator,
            allocated_buffers: Default::default(),
            allocated_images: Default::default(),
        })
    }

    #[inline(always)]
    pub fn uplaod_mesh(
        &mut self,
        mesh: &crate::engine::asset_system::mesh::Mesh,
    ) -> track::Result<()> {
        self.allocated_buffers
            .upload_mesh(self.allocator, mesh)
            .track()?;

        Ok(())
    }

    #[inline(always)]
    pub fn allocate_depth_image(
        &mut self,
        image_info: &vk::ImageCreateInfo,
        allocation_info: &vma::AllocationCreateInfo,
    ) -> track::Result<vk::Image> {
        let image_buffer =
            image::Image::new(self.allocator, image_info, allocation_info).track()?;
        let image = image_buffer.image;

        self.allocated_images.push(image_buffer);

        Ok(image)
    }

    #[inline(always)]
    pub unsafe fn bind_buffers_per_draw(
        &self,
        device: &ash::Device,
        command_buffer: vk::CommandBuffer,
    ) {
        self.allocated_buffers
            .vertex_buffers
            .iter()
            .zip(self.allocated_buffers.index_buffers.iter())
            .for_each(|(vertex_buffers, index_buffer)| {
                vertex_buffers.bind_buffer(device, command_buffer);
                index_buffer.bind_buffer(device, command_buffer);
            });
    }

    #[inline(always)]
    pub fn allocated_buffers_len(&self) -> usize {
        self.allocated_buffers.vertex_buffers.len()
    }
}

impl Drop for Resources {
    fn drop(&mut self) {
        unsafe {
            self.allocated_buffers
                .vertex_buffers
                .iter()
                .for_each(|vertex_buffers| {
                    vertex_buffers
                        .buffers
                        .iter()
                        .zip(vertex_buffers.allocations.iter())
                        .for_each(|(&buffer, &allocation)| {
                            vma::destroy_buffer(self.allocator, buffer, allocation);
                        })
                });

            self.allocated_buffers
                .index_buffers
                .iter()
                .for_each(|index_buffer| {
                    vma::destroy_buffer(
                        self.allocator,
                        index_buffer.buffer,
                        index_buffer.allocation,
                    )
                });

            self.allocated_images.iter().for_each(|allocated_image| {
                vma::destroy_image(
                    self.allocator,
                    allocated_image.image,
                    allocated_image.allocation,
                )
            });

            vma::destroy_allocator(self.allocator);
        }
    }
}
