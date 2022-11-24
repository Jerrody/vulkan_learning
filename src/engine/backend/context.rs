mod debug;
mod device;
mod surface;
mod swapchain;

pub use crate::engine::utils::cstring::*;
pub use ash::vk;
use raw_window_handle::HasRawDisplayHandle;
pub use std::mem::ManuallyDrop;
pub use std::os::raw::c_char;
pub use tracing_unwrap::ResultExt;

use self::device::DeviceHandle;
use self::surface::SurfaceHandle;
use self::swapchain::SwapchainHandle;

pub struct Context {
    entry: ManuallyDrop<ash::Entry>,
    instance: ash::Instance,
    #[cfg(feature = "validation")]
    debug_handle: debug::DebugHandle,
    surface_handle: SurfaceHandle,
    swapchain_handle: SwapchainHandle,
}

// TODO Return an error and process an error in the place of creation of instance.
impl Context {
    const ENGINE_NAME: *const c_char = cstr!("NoEngine");
    const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
    const APPLICATION_NAME: *const c_char = cstr!("BlueScreen");
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);

    pub fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { ash::Entry::load().unwrap_or_log() };

        let application_info = vk::ApplicationInfo::default()
            .engine_name(to_cstr(Self::ENGINE_NAME))
            .engine_version(Self::ENGINE_VERSION)
            .application_name(to_cstr(Self::APPLICATION_NAME))
            .application_version(Self::APPLICATION_VERSION)
            .api_version(vk::API_VERSION_1_3);

        let instance_extensions = {
            let surface_extensions =
                ash_window::enumerate_required_extensions(window.raw_display_handle())
                    .unwrap_or_log();

            let extensions = vec![
                #[cfg(feature = "validation")]
                debug::VALIDATION_LAYER_EXTENSION_NAME,
                ash::extensions::ext::DebugUtils::name().as_ptr(),
            ];

            [extensions, (*surface_extensions).to_owned()].concat()
        };

        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&instance_extensions);

        let instance = unsafe { entry.create_instance(&instance_info, None).unwrap_or_log() };

        #[cfg(feature = "validation")]
        let debug_handle = debug::DebugHandle::new(&entry, &instance).unwrap_or_log();
        let surface_handle = SurfaceHandle::new(&entry, &instance, window).unwrap_or_log();
        let device_handle = DeviceHandle::new(&instance, &surface_handle).unwrap_or_log();
        let swapchain_handle =
            swapchain::SwapchainHandle::new(&instance, &device_handle, &surface_handle, window)
                .unwrap_or_log();

        Self {
            entry: ManuallyDrop::new(entry),
            instance,
            #[cfg(feature = "validation")]
            debug_handle,
            surface_handle,
            swapchain_handle,
        }
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
