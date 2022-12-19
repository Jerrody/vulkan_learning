use std::mem;
use std::path::Path;

use ash::vk;
use math::Vec3;
use memoffset::offset_of;
use rayon::prelude::*;
use track::Context;

#[repr(C)]
pub struct Vertex {
    pub position: Vec3,
    pub color: Vec3,
}

pub struct VertexDescription {
    pub binding: vk::VertexInputBindingDescription,
    pub attributes: Vec<vk::VertexInputAttributeDescription>,
}

// TODO: Give a different vertex input description depending on type of resources.
impl VertexDescription {
    const VERTEX_FORMAT: vk::Format = vk::Format::R32G32B32_SFLOAT;
    const MESH_ATTRIBUTES_LENGTH: usize = 3;

    #[inline]
    pub fn new() -> Self {
        let binding = vk::VertexInputBindingDescription::default()
            .stride(mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX);
        let mut attributes = Vec::with_capacity(Self::MESH_ATTRIBUTES_LENGTH);

        let mut location = Default::default();

        attributes.push(Self::create_attribute(
            Default::default(),
            offset_of!(Vertex, position) as u32,
            Self::VERTEX_FORMAT,
            &mut location,
        ));
        attributes.push(Self::create_attribute(
            Default::default(),
            offset_of!(Vertex, color) as u32,
            Self::VERTEX_FORMAT,
            &mut location,
        ));

        Self {
            binding,
            attributes,
        }
    }

    #[inline(always)]
    fn create_attribute(
        binding: u32,
        offset: u32,
        format: vk::Format,
        location: &mut u32,
    ) -> vk::VertexInputAttributeDescription {
        let attribute = vk::VertexInputAttributeDescription::default()
            .binding(binding)
            .format(format)
            .offset(offset)
            .location(*location);

        *location += 1;

        attribute
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    pub const TRIANGLE_VERTEX_COUNT: usize = 3;

    pub fn new<P: AsRef<Path> + std::fmt::Debug>(path: P) -> track::Result<Self> {
        let (mut models, _) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                ignore_points: true,
                ignore_lines: true,
                single_index: true,
            },
        )
        .track()?;

        let mesh = models.remove(Default::default()).mesh;

        let vertices: Vec<Vertex> = mesh
            .positions
            .par_chunks_exact(Self::TRIANGLE_VERTEX_COUNT)
            .zip(mesh.normals.par_chunks_exact(Self::TRIANGLE_VERTEX_COUNT))
            .map(|(position, normal)| Vertex {
                position: Vec3::from_row_slice(position),
                color: Vec3::from_row_slice(normal),
            })
            .collect();

        Ok(Self {
            vertices,
            indices: mesh.indices,
        })
    }
}
