mod asset_system;
mod renderer;
mod utils;

use ash::prelude::VkResult;
use smallvec::SmallVec;
use tracing::info;
use track::Context as TrackContext;

use self::asset_system::mesh;

pub struct Engine {
    renderer: renderer::Renderer,
    meshes: SmallVec<[mesh::Mesh; Self::DEFAULT_STACK_BASED_MESHES_SIZE]>,
}

impl Engine {
    pub const DEFAULT_STACK_BASED_MESHES_SIZE: usize = 1024;

    pub fn new(window: &winit::window::Window) -> track::Result<Self> {
        info!("Initializing Renderer");
        let mut renderer = unsafe { renderer::Renderer::new(window).track()? };

        let mut meshes = SmallVec::new();

        let mesh = renderer
            .upload_asset(r"D:\Загрузки\test_models\monkey.obj")
            .track()?;

        meshes.push(mesh);

        Ok(Self { renderer, meshes })
    }

    #[inline(always)]
    pub fn draw(&self) -> VkResult<()> {
        unsafe { self.renderer.draw(&self.meshes[0]) }
    }
}
