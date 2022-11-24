use ash::vk;
use std::ffi::CStr;
use std::os::raw::c_char;
use tracing::{error, info, warn};
use track::Context;

use crate::engine::utils::cstring::cstr;

pub const VALIDATION_LAYER_EXTENSION_NAME: *const c_char = cstr!("VK_LAYER_KHRONOS_validation");

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
    debug_loader: ash::extensions::ext::DebugUtils,
    pub debug_utils: vk::DebugUtilsMessengerEXT,
}

impl DebugHandle {
    #[inline(always)]
    pub fn new(entry: &ash::Entry, instance: &ash::Instance) -> track::Result<Self> {
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

impl Drop for DebugHandle {
    fn drop(&mut self) {
        unsafe {
            self.debug_loader
                .destroy_debug_utils_messenger(self.debug_utils, None);
        }
    }
}
