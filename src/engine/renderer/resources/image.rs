use ash::vk;
use track::Context;

pub struct Image {
    pub image: vk::Image,
    pub allocation: vma::Allocation,
}

impl Image {
    pub fn new(
        allocator: vma::Allocator,
        image_info: &vk::ImageCreateInfo,
        allocation_info: &vma::AllocationCreateInfo,
    ) -> track::Result<Self> {
        let (image, allocation, _) =
            unsafe { vma::create_image(allocator, image_info, allocation_info).track()? };

        Ok(Self { image, allocation })
    }
}
