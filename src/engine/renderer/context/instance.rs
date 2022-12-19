use std::ffi::CStr;

use ash::vk;
use raw_window_handle::HasRawDisplayHandle;
use tracing::info;
use track::Context;

use crate::{cstr, engine::renderer::context::debug};

pub struct InstaceHandle {
    pub entry: ash::Entry,
    pub instance: ash::Instance,
}

impl InstaceHandle {
    const ENGINE_NAME: &CStr = cstr!("NoEngine");
    const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
    const APPLICATION_NAME: &CStr = cstr!("Triangle");
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);

    pub fn new(window: &winit::window::Window) -> track::Result<Self> {
        let entry = unsafe { ash::Entry::load().track()? };

        info!("Setting up application Info");

        let application_info = vk::ApplicationInfo::default()
            .engine_name(Self::ENGINE_NAME)
            .engine_version(Self::ENGINE_VERSION)
            .application_name(Self::APPLICATION_NAME)
            .application_version(Self::APPLICATION_VERSION)
            .api_version(vk::API_VERSION_1_3);

        info!("Setting up Vulkan Instance Info");

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

        info!("Created Vulkan Instance");

        Ok(Self { entry, instance })
    }
}
