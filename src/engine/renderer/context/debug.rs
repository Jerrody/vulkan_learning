use ash::vk;
use std::ffi::CStr;
use tracing::{error, info, warn};
use track::Context;

use crate::engine::utils::cstring::cstr;

pub const VALIDATION_LAYER_EXTENSION_NAME: &CStr = cstr!("VK_LAYER_KHRONOS_validation");

pub unsafe extern "system" fn debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_types: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut std::ffi::c_void,
) -> vk::Bool32 {
    let message_type = format!("{message_types:?}");

    // NOTE: Spaces between `\n` and {} need for alignment with `tracing` messages.
    let message = format!("\n  [{message_type}]\n  {:?}", unsafe {
        CStr::from_ptr((*p_callback_data).p_message)
    });

    match message_severity {
        vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => error!(message),
        vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => warn!(message),
        vk::DebugUtilsMessageSeverityFlagsEXT::INFO => info!(message),
        _ => warn!(message),
    }

    vk::FALSE
}

pub struct DebugHandle {
    pub debug_loader: ash::extensions::ext::DebugUtils,
    pub debug_utils: vk::DebugUtilsMessengerEXT,
}

impl DebugHandle {
    #[inline(always)]
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> track::Result<Self> {
        info!("Validation Enabled. Vulkan will report Validation Info");

        let (debug_loader, debug_utils) = {
            let debug_loader = ash::extensions::ext::DebugUtils::new(entry, instance);

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
                )
                .pfn_user_callback(Some(debug_callback));

            let debug_utils = unsafe {
                debug_loader
                    .create_debug_utils_messenger(&debug_utils_info, None)
                    .track()?
            };

            (debug_loader, debug_utils)
        };

        Ok(Self {
            debug_loader,
            debug_utils,
        })
    }
}
