mod backend;
mod utils;

use backend::context::Context;
use track::Context as TrackContext;

pub struct Engine {
    context: Context,
}

impl Engine {
    pub fn new(window: &winit::window::Window) -> track::Result<Self> {
        let context = Context::new(window).track()?;

        Ok(Self { context })
    }
}
