use std::sync::{Arc, Weak};

use crate::{error::DeviceError, render_device::RenderDevice};

pub struct SwapChain {
    pub(crate) surface: wgpu::Surface<'static>,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) window: Weak<winit::window::Window>,
    pub(crate) size: winit::dpi::PhysicalSize<u32>,
}

impl SwapChain {
    pub fn new(
        device: &RenderDevice,
        window: Arc<winit::window::Window>,
    ) -> Result<SwapChain, DeviceError> {
        let instance = device.instance();
        let adapter = device.adapter();

        let surface = instance.create_surface(window.clone()).map_err(|err| {
            DeviceError::Unavailable(format!("Failed to create surface {:?}", err))
        })?;
        let cap = surface.get_capabilities(&adapter);
        // TODO : decide surface format candidates and if no candidate availabe, terminate the app
        let surface_format = cap.formats[0];
        log::info!("Surface format {:?} selected", surface_format);

        let size = window.inner_size();

        Ok(SwapChain {
            surface,
            surface_format,
            window: Arc::downgrade(&window),
            size,
        })
    }

    pub fn configure_surface(&self, device: &RenderDevice) {
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: self.surface_format,
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![self.surface_format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&device.device(), &surface_config);
    }
}
