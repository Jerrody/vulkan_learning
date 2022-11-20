pub use crate::engine::utils::cstring::*;
pub use ash::vk;
pub use std::mem::ManuallyDrop;
pub use std::os::raw::c_char;
pub use tracing_unwrap::ResultExt;

pub struct Context {
    entry: ManuallyDrop<ash::Entry>,
    instance: ash::Instance,
    #[cfg(feature = "validation")]
    debug_utils: vk::DebugUtilsMessengerEXT,
    #[cfg(feature = "validation")]
    debug_loader: ash::extensions::ext::DebugUtils,
}

impl Context {
    const ENGINE_NAME: *const c_char = cstr!("NoEngine");
    const ENGINE_VERSION: u32 = vk::make_api_version(0, 0, 1, 0);
    const APPLICATION_NAME: *const c_char = cstr!("BlueScreen");
    const APPLICATION_VERSION: u32 = vk::make_api_version(0, 1, 0, 0);

    pub fn new(window: &winit::window::Window) -> Self {
        let entry = unsafe { ash::Entry::load().unwrap() };

        let application_info = vk::ApplicationInfo::default()
            .engine_name(to_cstr(Self::ENGINE_NAME))
            .engine_version(Self::ENGINE_VERSION)
            .application_name(to_cstr(Self::APPLICATION_NAME))
            .application_version(Self::APPLICATION_VERSION)
            .api_version(vk::API_VERSION_1_3);

        #[cfg(not(feature = "validation"))]
        let instance_extensions = [];
        #[cfg(feature = "validation")]
        let instance_extensions = [
            crate::engine::utils::validation::VALIDATION_LAYER_EXTENSION_NAME,
            ash::extensions::ext::DebugUtils::name().as_ptr(),
        ];

        let instance_info = vk::InstanceCreateInfo::default()
            .application_info(&application_info)
            .enabled_extension_names(&instance_extensions);

        let instance = unsafe { entry.create_instance(&instance_info, None).unwrap_or_log() };

        #[cfg(feature = "validation")]
        let debug_loader = ash::extensions::ext::DebugUtils::new(&entry, &instance);

        #[cfg(feature = "validation")]
        let debug_utils = {
            let debug_utils_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                .message_severity(
                    vk::DebugUtilsMessageSeverityFlagsEXT::ERROR
                        | vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
                        | vk::DebugUtilsMessageSeverityFlagsEXT::INFO,
                )
                .message_type(
                    vk::DebugUtilsMessageTypeFlagsEXT::GENERAL
                        | vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE
                        | vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
                );

            unsafe {
                debug_loader
                    .create_debug_utils_messenger(&debug_utils_info, None)
                    .unwrap()
            }
        };

        Self {
            entry: ManuallyDrop::new(entry),
            instance,
            #[cfg(feature = "validation")]
            debug_utils,
            #[cfg(feature = "validation")]
            debug_loader,
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_instance(None);

            #[cfg(feature = "validation")]
            self.debug_loader
                .destroy_debug_utils_messenger(self.debug_utils, None);

            ManuallyDrop::drop(&mut self.entry);
        }
    }
}
