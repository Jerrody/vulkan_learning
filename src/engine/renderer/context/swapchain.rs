use super::surface::SurfaceHandle;
use ash::{extensions::khr, vk};
use smallvec::{SmallVec, ToSmallVec};
use track::Context;

type Images = (SmallVec<[vk::Image; 3]>, SmallVec<[vk::ImageView; 3]>);

pub struct SwapchainHandle {
    pub swapchain_loader: khr::Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: SmallVec<[vk::Image; 3]>,
    pub image_views: SmallVec<[vk::ImageView; 3]>,
    pub image_extent: vk::Extent2D,
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
            &device_handle.device,
            device_handle.surface_format,
            device_handle.present_mode,
            device_handle.surface_capabilities,
            surface_handle,
            min_image_count,
            image_extent,
        )
        .track()?;

        let (images, image_views) = Self::create_images(
            &device_handle.device,
            device_handle.surface_format.format,
            &swapchain_loader,
            swapchain,
        )
        .track()?;

        Ok(Self {
            swapchain_loader,
            swapchain,
            images,
            image_views,
            image_extent,
        })
    }
    #[inline]
    #[allow(clippy::too_many_arguments)]
    fn create_swapchain(
        instance: &ash::Instance,
        device: &ash::Device,
        surface_format: vk::SurfaceFormatKHR,
        present_mode: vk::PresentModeKHR,
        surface_capabilities: vk::SurfaceCapabilitiesKHR,
        surface_handle: &SurfaceHandle,
        min_image_count: u32,
        image_extent: vk::Extent2D,
    ) -> track::Result<(khr::Swapchain, vk::SwapchainKHR)> {
        let swapchain_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface_handle.surface)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .present_mode(present_mode)
            .min_image_count(min_image_count)
            .image_array_layers(1)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .image_extent(image_extent)
            .pre_transform(surface_capabilities.current_transform)
            .clipped(true);

        let swapchain_loader = ash::extensions::khr::Swapchain::new(instance, device);

        let swapchain = unsafe {
            swapchain_loader
                .create_swapchain(&swapchain_info, None)
                .track()?
        };

        Ok((swapchain_loader, swapchain))
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

    #[inline(always)]
    fn choose_min_image_count(surface_capabilities: vk::SurfaceCapabilitiesKHR) -> u32 {
        let max_image_count = surface_capabilities.max_image_count;

        let mut min_image_count = surface_capabilities.min_image_count + 1;
        if min_image_count > 0 && min_image_count > max_image_count {
            min_image_count = max_image_count;
        }

        min_image_count
    }

    fn create_images(
        device: &ash::Device,
        format: vk::Format,
        swapchain_loader: &khr::Swapchain,
        swapchain: vk::SwapchainKHR,
    ) -> track::Result<Images> {
        let images = unsafe {
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
                    .format(format)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .subresource_range(vk::ImageSubresourceRange {
                        aspect_mask: vk::ImageAspectFlags::COLOR,
                        level_count: 1,
                        layer_count: 1,
                        ..Default::default()
                    });

                unsafe { device.create_image_view(&image_view_info, None).unwrap() }
            })
            .collect();

        Ok((images, image_views))
    }
}
