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
    pub(super) render_pipeline: wgpu::RenderPipeline,
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

        // TODO understand this shit

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../Shaders/shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[],
                immediate_size: 0,
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"), // 1.
                buffers: &[], // 2.
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState { // 3.
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState { // 4.
                    format: window.as_ref().unwrap().surface_config.as_ref().unwrap().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList, // 1.
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw, // 2.
                cull_mode: Some(wgpu::Face::Back),
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None, // 1.
            multisample: wgpu::MultisampleState {
                count: 1, // 2.
                mask: !0, // 3.
                alpha_to_coverage_enabled: false, // 4.
            },
            multiview_mask: None, // 5.
            cache: None, // 6.
        });

        Self {
            device,
            adapter,
            queue,
            render_pipeline,
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

    pub(crate) fn render_triangle(&mut self, windows: &mut Window) {
        let Some(render_target) = windows.acquire_render_target(&self) else { return; };

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        {
            let mut _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &render_target.view(),
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 1.0,
                            g: 0.0,
                            b: 0.0,
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

            _render_pass.set_pipeline(&self.render_pipeline);
            _render_pass.draw(0..3, 0..1);
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        render_target.present();
    }
}
