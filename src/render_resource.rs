use crate::error::ResourceError;

bitflags::bitflags! {
    #[derive(Default, Copy, Clone)]
    pub struct ResourceFlag: u32 {
        const NONE = 0x00000000;
        const ALLOW_UAV = 0x00000001;
        const RENDER_TARGET = 0x00000002;
        // Add additional flags as needed
    }
}

pub struct ResourceInfo {
    pub flags: ResourceFlag,
    pub request_size: u64,
    pub allocation_size: u64,
}

pub struct TextureInfo {
    pub base_info: ResourceInfo,
    pub extent: wgpu::Extent3d,
    pub format: wgpu::TextureFormat,
}

pub struct TextureCreateInfo {
    pub extent: wgpu::Extent3d,
    pub format: wgpu::TextureFormat,
    pub num_mips: u32,
}

impl TextureCreateInfo {
    pub fn request_size(&self) -> u64 {
        self.format.theoretical_memory_footprint(self.extent)
    }
}

pub struct TextureViewCreateInfo {
    pub base_mip: u32,
    pub num_mips: u32,
    pub base_slice: u32,
    pub num_slices: u32,
}

pub trait RenderResource {
    fn resource_flag(&self) -> ResourceFlag;
    fn request_size(&self) -> u64;
    fn allocation_size(&self) -> u64;
}

pub trait RenderResourceView {
    fn resource(&self) -> Result<std::sync::Weak<dyn RenderResource>, ResourceError>;
}
