use ash::vk;
use smallvec::{SmallVec, ToSmallVec};
use track::Context;

pub struct Command {
    pub command_pool: vk::CommandPool,
    pub command_buffers: SmallVec<[vk::CommandBuffer; 3]>,
}

impl Command {
    pub fn new(
        device: &ash::Device,
        queue_family_index: u32,
        image_count: u32,
    ) -> track::Result<Self> {
        let command_pool_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family_index)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

        let command_pool = unsafe {
            device
                .create_command_pool(&command_pool_info, None)
                .track()?
        };

        let command_buffer_info = vk::CommandBufferAllocateInfo::default()
            .command_pool(command_pool)
            .level(vk::CommandBufferLevel::PRIMARY)
            .command_buffer_count(image_count);

        let command_buffers = unsafe {
            device
                .allocate_command_buffers(&command_buffer_info)
                .track()?
                .to_smallvec()
        };

        Ok(Self {
            command_pool,
            command_buffers,
        })
    }
}
