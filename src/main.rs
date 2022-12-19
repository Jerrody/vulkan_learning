#![feature(panic_info_message)]
#![feature(const_cstr_methods)]

mod engine;
mod logging;

use mimalloc::MiMalloc;
use winit::{
    event::{self, Event, WindowEvent},
    platform::windows::WindowBuilderExtWindows,
};

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
fn main() {
    let event_loop = winit::event_loop::EventLoop::new();
    let window = winit::window::WindowBuilder::new()
        .with_resizable(false)
        .with_min_inner_size(winit::dpi::LogicalSize::new(640, 480))
        .with_theme(Some(winit::window::Theme::Dark))
        .with_title("Vulkan Learning")
        .build(&event_loop)
        .unwrap();

    let _log_guard = logging::init_logging();

    let engine = engine::Engine::new(&window).unwrap();

    event_loop.run(move |event, _, control_flow| match event {
        Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                input:
                    event::KeyboardInput {
                        state: event::ElementState::Pressed,
                        virtual_keycode: Some(event::VirtualKeyCode::Escape),
                        ..
                    },
                ..
            } => control_flow.set_exit(),
            _ => (),
        },
        Event::MainEventsCleared => engine.draw().unwrap(),
        _ => (),
    });
}
