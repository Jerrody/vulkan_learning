use super::{device::DeviceHandle, surface::SurfaceHandle};
use ash::{extensions::khr, vk};
use smallvec::{SmallVec, ToSmallVec};
use tracing_unwrap::ResultExt;
use track::Context;

type Images = (SmallVec<[vk::Image; 4]>, SmallVec<[vk::ImageView; 4]>);

pub struct SwapchainHandle {
    swapchain_loader: khr::Swapchain,
    swapchain: vk::SwapchainKHR,
    images: SmallVec<[vk::Image; 4]>,
    image_views: SmallVec<[vk::ImageView; 4]>,
}

impl SwapchainHandle {
    pub fn new(
        instance: &ash::Instance,
        device_handle: &super::device::DeviceHandle,
        surface_handle: &super::surface::SurfaceHandle,
        window: &winit::window::Window,
    ) -> track::Result<Self> {
        let min_image_count = Self::choose_min_image_count(device_handle.surface_capabilities);
        let image_extent =
            Self::choose_extent(device_handle.surface_capabilities, window.inner_size());

        let (swapchain_loader, swapchain) = Self::create_swapchain(
            instance,
            device_handle,
            surface_handle,
            min_image_count,
            image_extent,
        )
        .track()?;

        let (images, image_views) =
            Self::create_images(device_handle, &swapchain_loader, swapchain).track()?;

        Ok(Self {
            swapchain_loader,
            swapchain,
            images,
            image_views,
        })
    }

    #[inline(always)]
    fn choose_min_image_count(surface_capabilities: vk::SurfaceCapabilitiesKHR) -> u32 {
        let max_image_count = surface_capabilities.max_image_count;

        let mut min_image_count = surface_capabilities.min_image_count + 1;
        if min_image_count > 0 && min_image_count > max_image_count {
            min_image_count = max_image_count;
        }

        min_image_count
    }

    #[inline(always)]
    fn choose_extent(
        surface_capabilities: vk::SurfaceCapabilitiesKHR,
        window_extent: winit::dpi::PhysicalSize<u32>,
    ) -> vk::Extent2D {
        match surface_capabilities.current_extent {
            vk::Extent2D {
                width: u32::MAX,
                height: u32::MAX,
            } => vk::Extent2D {
                width: window_extent.width,
                height: window_extent.height,
            },
            extent => extent,
        }
    }

    #[inline]
    fn create_swapchain(
        instance: &ash::Instance,
        device_handle: &DeviceHandle,
        surface_handle: &SurfaceHandle,
        min_image_count: u32,
        image_extent: vk::Extent2D,
    ) -> track::Result<(khr::Swapchain, vk::SwapchainKHR)> {
        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface_handle.surface)
            .image_format(device_handle.surface_format.format)
            .image_color_space(device_handle.surface_format.color_space)
            .present_mode(device_handle.present_mode)
            .min_image_count(min_image_count)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .image_extent(image_extent)
            .pre_transform(device_handle.surface_capabilities.current_transform)
            .clipped(true);

        let swapchain_loader =
            ash::extensions::khr::Swapchain::new(instance, &device_handle.device);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_info, None)
                .track()?
        };

        Ok((swapchain_loader, swapchain))
    }

    fn create_images(
        device_handle: &DeviceHandle,
        swapchain_loader: &khr::Swapchain,
        swapchain: vk::SwapchainKHR,
    ) -> track::Result<Images> {
        let images: SmallVec<[vk::Image; 4]> = unsafe {
            swapchain_loader
                .get_swapchain_images(swapchain)
                .track()?
                .to_smallvec()
        };

        let image_views = images
            .iter()
            .map(|&image| {
                let image_view_info = vk::ImageViewCreateInfo::default()
                    .image(image)
                    .format(device_handle.surface_format.format)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .subresource_range(vk::ImageSubresourceRange {
                        level_count: 1,
                        layer_count: 1,
                        ..Default::default()
                    });

                unsafe {
                    device_handle
                        .device
                        .create_image_view(&image_view_info, None)
                        .unwrap_or_log()
                }
            })
            .collect::<SmallVec<[vk::ImageView; 4]>>();

        Ok((images, image_views))
    }
}
