use crate::rendering::window::Window;
use log::error;
use std::process::abort;
use wgpu::{Adapter, CommandEncoder, Device, DeviceDescriptor, ExperimentalFeatures, Features, Instance, Limits, MemoryHints, PowerPreference, Queue, RequestAdapterOptions, Trace};
use wgpu::util::DeviceExt;
use crate::rendering::render_target::RenderTarget;

pub struct Renderer {
    pub(super) device: Device,
    pub(super) adapter: Adapter,
    pub(super) queue: Queue,
    pub(super) render_pipeline: wgpu::RenderPipeline,
    pub(super) vertex_buffer: wgpu::Buffer,
    pub(super) index_buffer: wgpu::Buffer,
    pub(super) num_indices: u32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

const VERTICES: &[Vertex] = &[
    // Eye right
    Vertex { position: [0.7, 0.9, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [0.7, 0.7, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [0.9, 0.9, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [0.9, 0.7, 0.0], color: [1.0, 1.0, 1.0] },

    // Eye left
    Vertex { position: [-0.7, 0.7, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-0.7, 0.9, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-0.9, 0.7, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-0.9, 0.9, 0.0], color: [1.0, 1.0, 1.0] },

    // Mouth
    Vertex { position: [0.6, -0.7, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-0.6, -0.7, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [-0.6, -0.8, 0.0], color: [1.0, 1.0, 1.0] },
    Vertex { position: [0.6, -0.8, 0.0], color: [1.0, 1.0, 1.0] },
];

const INDICES: &[u16] = &[
    0, 1, 2,
    2, 1, 3,

    4, 5, 6,
    6, 5, 7,

    8, 9, 10,
    8, 10, 11,
];

impl Renderer {
    pub async fn new(instance: &Instance, window: &mut Window) -> Self {
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&window.surface),
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

        window.configure_internal(&adapter, &device);

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
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc()],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: window.surface_config.as_ref().unwrap().format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
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

        let vertex_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(VERTICES),
                usage: wgpu::BufferUsages::VERTEX,
            }
        );

        let index_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(INDICES),
                usage: wgpu::BufferUsages::INDEX,
            }
        );
        let num_indices = INDICES.len() as u32;

        Self {
            device,
            adapter,
            queue,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
        }
    }

    pub fn start_frame(&self) -> CommandEncoder {
        self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        })
    }

    pub fn end_frame(&self, render_target: RenderTarget, encoder: CommandEncoder) {
        self.queue.submit(std::iter::once(encoder.finish()));
        render_target.present();
    }

    pub fn clear_screen(&mut self, render_target: &RenderTarget, encoder: &mut CommandEncoder) {
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

    pub fn render_triangle(&mut self, render_target: &RenderTarget, encoder: &mut CommandEncoder) {
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
        _render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        _render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
        _render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
    }
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
