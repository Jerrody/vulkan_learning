mod backend;
mod utils;

use backend::context::Context;

pub struct Engine {
    context: Context,
}

impl Engine {
    fn new(window: &winit::window::Window) -> Self {
        let context = Context::new(window);

        Self { context }
    }
}
