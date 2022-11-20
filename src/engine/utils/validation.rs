use ash::vk;
use std::ffi::CStr;
use std::os::raw::c_char;
use tracing::{error, info, warn};

use super::cstring::cstr;

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
