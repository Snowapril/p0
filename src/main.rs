use std::sync::{Arc, Weak};
use thiserror::Error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
mod common;
use common::app_info;

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum DeviceError {
    #[error("Out of memory")]
    OutOfMemory,
    #[error("Device is lost")]
    Lost,
    #[error("Unexpected error variant (driver implementation is at fault)")]
    Unexpected(String),
}

struct SwapChain {
    surface: wgpu::Surface<'static>,
    surface_format: wgpu::TextureFormat,
    window: Arc<Window>,
    size: winit::dpi::PhysicalSize<u32>,
}

impl SwapChain {
    fn new(device: &RenderDevice, window: Arc<Window>) -> Result<SwapChain, DeviceError> {
        let instance = device.instance();
        let adapter = device.adapter();

        let surface = instance.create_surface(window.clone()).map_err(|err| {
            DeviceError::Unexpected(format!("Failed to create surface {:?}", err))
        })?;
        let cap = surface.get_capabilities(&adapter);
        let surface_format = cap.formats[0];
        log::info!("Surface format {:?} selected", surface_format);

        let size = window.inner_size();

        Ok(SwapChain {
            surface,
            surface_format,
            window,
            size,
        })
    }
}

struct RenderDevice {
    instance: wgpu::Instance,
    adapter: wgpu::Adapter,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl RenderDevice {
    async fn new() -> Result<RenderDevice, DeviceError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|err| {
                DeviceError::Unexpected(format!(
                    "Failed to get adapter from current device {:?}",
                    err
                ))
            })?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|err| {
                DeviceError::Unexpected(format!("Failed to create logical device {:?}", err))
            })?;
        Ok(RenderDevice {
            instance,
            adapter,
            device,
            queue,
        })
    }

    fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }
}

struct Engine {
    render_device: RenderDevice,
    swapchain: Option<SwapChain>,
}

impl Engine {
    fn new() -> Result<Engine, DeviceError> {
        Ok(Engine {
            render_device: pollster::block_on(RenderDevice::new())?,
            swapchain: None,
        })
    }
}

impl ApplicationHandler for Engine {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // Create window object
        let window = Arc::new(
            event_loop
                .create_window(Window::default_attributes())
                .unwrap(),
        );

        const RETRY_COUNT: u8 = 3;
        for iteration in [0..RETRY_COUNT] {
            if let Ok(swapchain) = SwapChain::new(&self.render_device, window.clone()) {
                self.swapchain = Some(swapchain);
                break;
            }
        }

        if self.swapchain.is_none() {
            panic!("Failed to create swapchain");
        }
        window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        // let state = self.state.as_mut().unwrap();
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                // state.render();
                // Emits a new redraw requested event.
                // state.get_window().request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                // state.resize(size);
            }
            _ => (),
        }
    }
}

// Initialize logging in platform dependant ways.
fn init_logger() {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "wasm32")] {
            // As we don't have an environment to pull logging level from, we use the query string.
            let query_string = web_sys::window().unwrap().location().search().unwrap();
            let query_level: Option<log::LevelFilter> = parse_url_query_string(&query_string, "RUST_LOG")
                .and_then(|x| x.parse().ok());

            // We keep wgpu at Error level, as it's very noisy.
            let base_level = query_level.unwrap_or(log::LevelFilter::Info);
            let wgpu_level = query_level.unwrap_or(log::LevelFilter::Error);

            // On web, we use fern, as console_log doesn't have filtering on a per-module level.
            fern::Dispatch::new()
                .level(base_level)
                .level_for("wgpu_core", wgpu_level)
                .level_for("wgpu_hal", wgpu_level)
                .level_for("naga", wgpu_level)
                .chain(fern::Output::call(console_log::log))
                .apply()
                .unwrap();
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        } else {
            // parse_default_env will read the RUST_LOG environment variable and apply it on top
            // of these default filters.
            env_logger::builder()
                .filter_level(log::LevelFilter::Info)
                // We keep wgpu at Error level, as it's very noisy.
                .filter_module("wgpu_core", log::LevelFilter::Info)
                .filter_module("wgpu_hal", log::LevelFilter::Error)
                .filter_module("naga", log::LevelFilter::Error)
                .parse_default_env()
                .init();
        }
    }
}

fn main() {
    init_logger();

    log::info!(
        "Enabled backends: {:?}",
        wgpu::Instance::enabled_backend_features()
    );

    let event_loop = EventLoop::new().unwrap();

    // When the current loop iteration finishes, immediately begin a new
    // iteration regardless of whether or not new events are available to
    // process. Preferred for applications that want to render as fast as
    // possible, like games.
    event_loop.set_control_flow(ControlFlow::Poll);

    // When the current loop iteration finishes, suspend the thread until
    // another event arrives. Helps keeping CPU utilization low if nothing
    // is happening, which is preferred if the application might be idling in
    // the background.
    // event_loop.set_control_flow(ControlFlow::Wait);

    let mut engine = Engine::new()
        .map_err(|err| {
            log::error!("Failed to initialize p0 engine {:?}", err);
            panic!("fatal exit");
        })
        .unwrap();
    event_loop.run_app(&mut engine).unwrap();
}
