mod debug;
mod device;
mod shader;
mod surface;
mod swapchain;

pub use crate::engine::utils::cstring::*;
pub use ash::vk;
use raw_window_handle::HasRawDisplayHandle;
use std::ffi::CStr;
pub use std::mem::ManuallyDrop;
pub use std::os::raw::c_char;
pub use tracing_unwrap::ResultExt;
use track::Context as TrackContext; // Renamed the `Context`'s name due to name collision with backend's `Context`.

use self::device::DeviceHandle;
use self::surface::SurfaceHandle;
use self::swapchain::SwapchainHandle;

pub struct Context {
    entry: ManuallyDrop<ash::Entry>,
    instance: ash::Instance,
    #[cfg(feature = "validation")]
    debug_handle: debug::DebugHandle,
    surface_handle: SurfaceHandle,
    device_handle: DeviceHandle,
    swapchain_handle: SwapchainHandle,
}

// TODO Return an error and process an error in the place of creation of instance.
impl Context {
    const ENGINE_NAME: &CStr = unsafe { cstr("NoEngine\0") };
    const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
    const APPLICATION_NAME: &CStr = unsafe { cstr("Triangle\0") };
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);

    pub fn new(window: &winit::window::Window) -> track::Result<Self> {
        let entry = unsafe { ash::Entry::load().track()? };

        let application_info = vk::ApplicationInfo::default()
            .engine_name(Self::ENGINE_NAME)
            .engine_version(Self::ENGINE_VERSION)
            .application_name(Self::APPLICATION_NAME)
            .application_version(Self::APPLICATION_VERSION)
            .api_version(vk::API_VERSION_1_3);

        let instance_extensions = {
            let surface_extensions =
                ash_window::enumerate_required_extensions(window.raw_display_handle()).track()?;
            let debug_utils_ext = vec![ash::extensions::ext::DebugUtils::name().as_ptr()];

            [debug_utils_ext, (*surface_extensions).to_owned()].concat()
        };

        #[cfg(feature = "validation")]
        let instance_layers = [debug::VALIDATION_LAYER_EXTENSION_NAME.as_ptr()];

        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_layer_names(
                #[cfg(feature = "validation")]
                &instance_layers,
                #[cfg(not(feature = "validation"))]
                &[],
            )
            .enabled_extension_names(&instance_extensions);

        let instance = unsafe { entry.create_instance(&instance_info, None).track()? };

        #[cfg(feature = "validation")]
        let debug_handle = debug::DebugHandle::new(&entry, &instance).track()?;
        let surface_handle = SurfaceHandle::new(&entry, &instance, window).track()?;
        let device_handle = DeviceHandle::new(&instance, &surface_handle).track()?;
        let swapchain_handle =
            swapchain::SwapchainHandle::new(&instance, &device_handle, &surface_handle, window)
                .track()?;

        Ok(Self {
            entry: ManuallyDrop::new(entry),
            instance,
            #[cfg(feature = "validation")]
            debug_handle,
            surface_handle,
            device_handle,
            swapchain_handle,
        })
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);
            ManuallyDrop::drop(&mut self.entry);
        }
    }
}
