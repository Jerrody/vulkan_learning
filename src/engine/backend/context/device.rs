use ash::vk;
use tracing::info;
use tracing_unwrap::ResultExt;
use track::Context;

use crate::engine::backend::context::debug;

pub struct DeviceHandle {
    _physical_device: vk::PhysicalDevice,
    pub device: ash::Device,
    pub device_properties: vk::PhysicalDeviceProperties,
    pub queue_family_index: u32,
    pub queue_graphics: vk::Queue, // TODO: Make a "Queue Manager" for the queues and store them not explicitly in `DeviceHandle`.
    pub surface_capabilities: vk::SurfaceCapabilitiesKHR,
    pub surface_format: vk::SurfaceFormatKHR,
    pub present_mode: vk::PresentModeKHR,
}

impl DeviceHandle {
    pub fn new(
        instance: &ash::Instance,
        surface_handle: &super::surface::SurfaceHandle,
    ) -> track::Result<Self> {
        let (physical_device, device_properties, queue_family_index, surface_format, present_mode) = unsafe {
            instance
                .enumerate_physical_devices()
                .track()?
                .iter()
                .filter_map(|&physical_device| {
                    let queue_family_index = match instance
                        .get_physical_device_queue_family_properties(physical_device)
                        .into_iter()
                        .enumerate()
                        .position(|(queue_family_index, queue_family_property)| {
                            queue_family_property
                                .queue_flags
                                .contains(vk::QueueFlags::GRAPHICS)
                                && surface_handle
                                    .surface_loader
                                    .get_physical_device_surface_support(
                                        physical_device,
                                        queue_family_index as u32,
                                        surface_handle.surface,
                                    )
                                    .unwrap_or_log()
                        }) {
                        Some(queue_family_index) => queue_family_index,
                        None => return None,
                    };

                    let formats = surface_handle
                        .surface_loader
                        .get_physical_device_surface_formats(
                            physical_device,
                            surface_handle.surface,
                        )
                        .unwrap_or_log();

                    let format = match formats.into_iter().find(|surface_format| {
                        (surface_format.format == vk::Format::R8G8B8A8_SRGB
                            || surface_format.format == vk::Format::B8G8R8A8_SRGB)
                            && surface_format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
                    }) {
                        Some(surface_format) => surface_format,
                        None => return None,
                    };

                    let present_mode = surface_handle
                        .surface_loader
                        .get_physical_device_surface_present_modes(
                            physical_device,
                            surface_handle.surface,
                        )
                        .unwrap_or_log()
                        .into_iter()
                        .find(|&present_mode| present_mode == vk::PresentModeKHR::MAILBOX)
                        .unwrap_or(vk::PresentModeKHR::FIFO);

                    let device_properties =
                        instance.get_physical_device_properties(physical_device);

                    Some((
                        physical_device,
                        device_properties,
                        queue_family_index as u32,
                        format,
                        present_mode,
                    ))
                })
                .max_by_key(
                    |(_, device_properties, _, _, _)| match device_properties.device_type {
                        vk::PhysicalDeviceType::DISCRETE_GPU => 3,
                        vk::PhysicalDeviceType::INTEGRATED_GPU => 2,
                        _ => 0,
                    },
                )
                .unwrap_or_else(|| panic!("Failed to find compitable device"))
        };

        let device_name = unsafe {
            std::ffi::CStr::from_ptr(device_properties.device_name.as_ptr())
                .to_str()
                .unwrap_or_log()
                .to_string()
        };
        info!("Found compitable device: {device_name}");

        let surface_capabilities = unsafe {
            surface_handle
                .surface_loader
                .get_physical_device_surface_capabilities(physical_device, surface_handle.surface)
                .unwrap_or_log()
        };

        let device_extension_names = [ash::extensions::khr::Swapchain::name().as_ptr()];
        let device_layer_names = [
            #[cfg(feature = "validation")]
            debug::VALIDATION_LAYER_EXTENSION_NAME.as_ptr(),
        ];
        let mut device_features3 = vk::PhysicalDeviceVulkan13Features::default()
            .dynamic_rendering(true)
            .synchronization2(true);

        let queue_create_info = [vk::DeviceQueueCreateInfo::default()
            .queue_family_index(queue_family_index)
            .queue_priorities(&[1.0])];

        let device_info = vk::DeviceCreateInfo::default()
            .enabled_extension_names(&device_extension_names)
            .enabled_layer_names(&device_layer_names)
            .queue_create_infos(&queue_create_info)
            .push_next(&mut device_features3);
        let device = unsafe {
            instance
                .create_device(physical_device, &device_info, None)
                .track()?
        };

        let queue_graphics = unsafe { device.get_device_queue(queue_family_index, 0) };

        Ok(Self {
            _physical_device: physical_device,
            device,
            device_properties,
            queue_family_index,
            queue_graphics,
            surface_capabilities,
            surface_format,
            present_mode,
        })
    }
}

impl Drop for DeviceHandle {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_device(None);
        }
    }
}
