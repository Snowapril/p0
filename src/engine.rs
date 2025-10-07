use std::sync::Arc;

use wgpu::Device;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};

use crate::{error::DeviceError, render_device::RenderDevice, swapchain::SwapChain};

pub struct Engine {
    pub(crate) render_device: RenderDevice,
    pub(crate) window: Option<Arc<Window>>,
    // swapchain must have weak-ref to window handle. if window handle destroyed, swapchain is no more available.
    pub(crate) swapchain: Option<SwapChain>,
}

impl Engine {
    pub fn new() -> Result<Engine, DeviceError> {
        Ok(Engine {
            render_device: pollster::block_on(RenderDevice::new())?,
            window: None,
            swapchain: None,
        })
    }

    pub fn render(&mut self) -> Result<(), DeviceError> {
        let window: &Arc<Window> = self.window.as_ref().ok_or(DeviceError::Unexpected)?;
        let swapchain: &SwapChain = self.swapchain.as_ref().ok_or(DeviceError::Unexpected)?;
        // Create texture view
        let surface_texture = swapchain
            .surface()
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(swapchain.surface_format().add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self
            .render_device
            .device()
            .create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let renderpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &texture_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        // If you wanted to call any drawing commands, they would go here.

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.render_device
            .command_queue()
            .submit([encoder.finish()]);
        window.pre_present_notify();
        surface_texture.present();

        Ok(())
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // TODO : check the current window handle or swapchain is no more valid.

        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        const RETRY_COUNT: u8 = 3;
        for _ in [0..RETRY_COUNT] {
            if let Ok(swapchain) = SwapChain::new(&self.render_device, window.clone()) {
                self.swapchain = Some(swapchain);
                break;
            }
        }

        if let Some(swapchain) = self.swapchain.as_mut() {
            swapchain.configure_surface(&self.render_device, window.inner_size());
        } else {
            panic!("Failed to create swapchain after {:?} retry", RETRY_COUNT);
        }

        window.request_redraw();
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                log::info!("Terminate the app as close button pressed");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                match self.render() {
                    Ok(_) => {
                        // Emits a new redraw requested event.
                        if let Some(window) = &self.window {
                            window.request_redraw();
                        }
                    }
                    Err(err) => {
                        log::error!("rendering failure {:?}", err);
                    }
                }
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                if let Some(swapchain) = self.swapchain.as_mut() {
                    swapchain.configure_surface(&self.render_device, size);
                }
            }
            _ => (),
        }
    }
}
