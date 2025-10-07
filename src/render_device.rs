use crate::error::DeviceError;

pub struct RenderDevice {
    pub(crate) instance: wgpu::Instance,
    pub(crate) adapter: wgpu::Adapter,
    pub(crate) device: wgpu::Device,
    pub(crate) queue: wgpu::Queue,
}

impl RenderDevice {
    pub async fn new() -> Result<RenderDevice, DeviceError> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .map_err(|err| {
                DeviceError::Unavailable(format!(
                    "Failed to get adapter from current device {:?}",
                    err
                ))
            })?;
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .map_err(|err| {
                DeviceError::Unavailable(format!("Failed to create logical device {:?}", err))
            })?;
        Ok(RenderDevice {
            instance,
            adapter,
            device,
            queue,
        })
    }

    pub fn instance(&self) -> &wgpu::Instance {
        &self.instance
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }
}
