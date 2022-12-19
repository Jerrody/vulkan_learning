use std::mem;

use ash::vk;
use smallvec::SmallVec;
use track::Context;

pub trait Buffer {
    unsafe fn bind_buffer(&self, device: &ash::Device, command_buffer: vk::CommandBuffer);
}

pub struct VertexBuffers {
    pub buffers: SmallVec<[vk::Buffer; AllocatedBuffers::VERTEX_BUFFERS_COUNT_PER_MESH]>,
    pub offsets: SmallVec<[u64; AllocatedBuffers::VERTEX_BUFFERS_COUNT_PER_MESH]>,
    pub allocations: SmallVec<[vma::Allocation; AllocatedBuffers::VERTEX_BUFFERS_COUNT_PER_MESH]>,
}

impl Buffer for VertexBuffers {
    #[inline(always)]
    unsafe fn bind_buffer(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        device.cmd_bind_vertex_buffers(
            command_buffer,
            Default::default(),
            &self.buffers,
            &self.offsets,
        );
    }
}

impl VertexBuffers {
    pub fn new(buffers: &[vk::Buffer], offsets: &[u64], allocations: &[vma::Allocation]) -> Self {
        Self {
            buffers: buffers.into(),
            offsets: offsets.into(),
            allocations: allocations.into(),
        }
    }
}

pub struct IndexBuffer {
    pub buffer: vk::Buffer,
    pub index_type: vk::IndexType,
    pub offset: u64,
    pub allocation: vma::Allocation,
}

impl Buffer for IndexBuffer {
    #[inline(always)]
    unsafe fn bind_buffer(&self, device: &ash::Device, command_buffer: vk::CommandBuffer) {
        device.cmd_bind_index_buffer(command_buffer, self.buffer, self.offset, self.index_type);
    }
}

impl IndexBuffer {
    pub fn new(
        buffer: vk::Buffer,
        allocation: vma::Allocation,
        index_type: vk::IndexType,
        offset: u64,
    ) -> Self {
        Self {
            buffer,
            allocation,
            index_type,
            offset,
        }
    }
}

#[derive(Default)]
pub struct AllocatedBuffers {
    pub vertex_buffers: Vec<VertexBuffers>,
    pub index_buffers: Vec<IndexBuffer>,
}

impl AllocatedBuffers {
    pub const VERTEX_BUFFERS_COUNT_PER_MESH: usize = 4;

    pub fn upload_mesh(
        &mut self,
        allocator: vma::Allocator,
        mesh: &crate::engine::asset_system::mesh::Mesh,
    ) -> track::Result<()> {
        let size = mem::size_of_val(&*mesh.vertices);
        let allocation_info = vma::AllocationCreateInfo {
            usage: vma::MemoryUsage::AUTO,
            flags: vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            Self::allocate_buffer(
                allocator,
                size,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                &allocation_info,
                mesh.vertices.as_ptr().cast(),
            )
            .track()?
        };

        let buffers = [buffer];
        let offsets = [0];
        let allocations = [allocation];
        let vertex_buffers = VertexBuffers::new(&buffers, &offsets, &allocations);
        self.vertex_buffers.push(vertex_buffers);

        let size = mem::size_of_val(&*mesh.indices);
        let allocation_info = vma::AllocationCreateInfo {
            usage: vma::MemoryUsage::AUTO,
            flags: vma::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
            ..Default::default()
        };

        let (buffer, allocation) = unsafe {
            Self::allocate_buffer(
                allocator,
                size,
                vk::BufferUsageFlags::INDEX_BUFFER,
                &allocation_info,
                mesh.indices.as_ptr().cast(),
            )
            .track()?
        };

        let index_buffer = IndexBuffer::new(
            buffer,
            allocation,
            vk::IndexType::UINT32,
            Default::default(),
        );
        self.index_buffers.push(index_buffer);

        Ok(())
    }

    unsafe fn allocate_buffer(
        allocator: vma::Allocator,
        size: usize,
        usage: vk::BufferUsageFlags,
        allocation_info: &vma::AllocationCreateInfo,
        ptr_data: *const std::ffi::c_void,
    ) -> track::Result<(vk::Buffer, vma::Allocation)> {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(size as u64)
            .usage(usage);

        let (buffer, allocation, _) =
            vma::create_buffer(allocator, &buffer_info, allocation_info).track()?;

        let ptr_buffer = vma::map_memory(allocator, allocation).track()?;

        std::ptr::copy_nonoverlapping(ptr_data, ptr_buffer, size);

        vma::unmap_memory(allocator, allocation);

        Ok((buffer, allocation))
    }
}
