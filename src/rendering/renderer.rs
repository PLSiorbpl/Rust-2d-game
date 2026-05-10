use crate::rendering::window::Window;
use log::error;
use std::process::abort;
use wgpu::{
    Adapter, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance, Limits,
    MemoryHints, PowerPreference, Queue, RequestAdapterOptions, Trace,
};

pub struct Renderer {
    pub(super) device: Device,
    pub(super) adapter: Adapter,
    pub(super) queue: Queue,
}

impl Renderer {
    pub async fn new(instance: &Instance, window: Option<&Window>) -> Self {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: window.map(|e| &e.surface),
            })
            .await
            .unwrap_or_else(|err| {
                error!("Failed to fetch an adapter!: {:?}", err);
                abort()
            });

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: None,
                required_features: Features::empty(),
                required_limits: Limits::defaults(),
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: MemoryHints::MemoryUsage,
                trace: Trace::Off,
            })
            .await
            .unwrap_or_else(|err| {
                error!("Failed to fetch a device and queue!: {:?}", err);
                abort()
            });

        Self {
            device,
            adapter,
            queue,
        }
    }
}
