use std::sync::{Arc, Weak};
use wgpu::TextureViewDescriptor;

use crate::error::ResourceError;
use crate::render_device::RenderDevice;
use crate::render_resource::{
    RenderResource, RenderResourceView, ResourceFlag, TextureCreateInfo, TextureInfo,
    TextureViewCreateInfo,
};

pub struct Texture {
    pub info: TextureInfo,
    pub texture: wgpu::Texture,
}

pub struct TextureView {
    pub parent: Weak<Texture>,
    pub view: wgpu::TextureView,
}

impl Texture {
    // Texture::new() returns Arc<Texture>
    pub fn new(device: &RenderDevice, create_info: TextureCreateInfo, name: &str) -> Arc<Texture> {
        let device: &wgpu::Device = device.device();

        let texture_desc = wgpu::TextureDescriptor {
            label: Some(name),
            size: create_info.extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_DST
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        };

        let texture = device.create_texture(&texture_desc);

        Arc::new(Texture {
            texture,
            info: TextureInfo {
                base_info: crate::render_resource::ResourceInfo {
                    flags: crate::render_resource::ResourceFlag::NONE,
                    request_size: create_info.request_size(),
                    allocation_size: 0, // TODO : check how to know actual device memory footprint
                },
                extent: create_info.extent,
                format: create_info.format,
            },
        })
    }

    // TextureView creation now expects Arc<Texture>, returns TextureView with Weak<Texture>
    pub fn create_view(self: &Arc<Self>, view_info: TextureViewCreateInfo) -> TextureView {
        let texture_view = self.texture.create_view(&TextureViewDescriptor {
            label: None,
            format: Some(self.info.format),
            dimension: None,
            usage: None,
            aspect: wgpu::TextureAspect::All,
            base_mip_level: view_info.base_mip,
            mip_level_count: Some(view_info.num_mips),
            base_array_layer: view_info.base_slice,
            array_layer_count: Some(view_info.num_slices),
        });
        TextureView {
            parent: Arc::downgrade(self),
            view: texture_view,
        }
    }
}

impl RenderResource for Texture {
    fn resource_flag(&self) -> ResourceFlag {
        self.info.base_info.flags
    }
    fn request_size(&self) -> u64 {
        self.info.base_info.request_size
    }
    fn allocation_size(&self) -> u64 {
        self.info.base_info.allocation_size
    }
}

impl RenderResourceView for TextureView {
    fn resource(&self) -> Result<std::sync::Weak<dyn RenderResource>, ResourceError> {
        match self.parent.upgrade() {
            Some(arc) => Ok(Arc::downgrade(&arc) as std::sync::Weak<dyn RenderResource>),
            None => Err(ResourceError::Orphan),
        }
    }
}
