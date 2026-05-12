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

    pub(crate) fn clear_screen(&mut self, windows: &mut Window) {
        let Some(render_target) = windows.acquire_render_target(&self) else { return; };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_target.view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.3,
                            g: 0.9,
                            b: 0.1,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
                multiview_mask: None,
            });
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        render_target.present();
    }
}
