use ash::{extensions::khr, vk};
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use track::Context;

pub struct SurfaceHandle {
    pub surface_loader: khr::Surface,
    pub surface: vk::SurfaceKHR,
}

impl SurfaceHandle {
    #[inline]
    pub fn new(
        entry: &ash::Entry,
        instance: &ash::Instance,
        window: &winit::window::Window,
    ) -> track::Result<Self> {
        let surface_loader = ash::extensions::khr::Surface::new(entry, instance);
        let surface = unsafe {
            ash_window::create_surface(
                entry,
                instance,
                window.raw_display_handle(),
                window.raw_window_handle(),
                None,
            )
        }
        .track()?;

        Ok(Self {
            surface_loader,
            surface,
        })
    }
}
