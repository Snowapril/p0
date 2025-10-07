use std::sync::Arc;

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

    pub fn renderable(&self) -> bool {
        self.window.is_some() && self.swapchain.is_some()
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

        if self.swapchain.is_none() {
            panic!("Failed to create swapchain after {:?} retry", RETRY_COUNT);
        }
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // state.render();
                // Emits a new redraw requested event.
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
