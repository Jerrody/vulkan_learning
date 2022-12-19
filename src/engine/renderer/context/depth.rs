use ash::vk;
use track::Context;

pub struct DepthBuffer {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
}

impl DepthBuffer {
    pub const DEPTH_BUFFER_FORMAT: vk::Format = vk::Format::D32_SFLOAT;
    pub const DEPTH_BUFFER_USAGE: vk::ImageUsageFlags =
        vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT;

    pub fn new(
        device: &ash::Device,
        resources: &mut super::resources::Resources,
        extent: vk::Extent3D,
    ) -> track::Result<Self> {
        let image_info = vk::ImageCreateInfo::default()
            .format(Self::DEPTH_BUFFER_FORMAT)
            .usage(Self::DEPTH_BUFFER_USAGE)
            .extent(extent)
            .samples(vk::SampleCountFlags::TYPE_1)
            .tiling(vk::ImageTiling::OPTIMAL)
            .mip_levels(1)
            .array_layers(1)
            .image_type(vk::ImageType::TYPE_2D);

        let allocation_info = vma::AllocationCreateInfo {
            required_flags: vk::MemoryPropertyFlags::DEVICE_LOCAL,
            usage: vma::MemoryUsage::AUTO,
            ..Default::default()
        };

        let image = resources
            .allocate_depth_image(&image_info, &allocation_info)
            .track()?;

        let image_view_info = vk::ImageViewCreateInfo::default()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(Self::DEPTH_BUFFER_FORMAT)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::DEPTH,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            });

        let image_view = unsafe { device.create_image_view(&image_view_info, None).track()? };

        Ok(Self { image, image_view })
    }
}
