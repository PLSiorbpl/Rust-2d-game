use crate::rendering::render_target::RenderTarget;
use crate::rendering::renderer::Renderer;
use log::{error, warn};
use std::cell::OnceCell;
use std::process::abort;
use std::sync::Arc;
use thiserror::Error;
use wgpu::wgt::TextureViewDescriptor;
use wgpu::{
    CompositeAlphaMode, CreateSurfaceError, CurrentSurfaceTexture, Instance, PresentMode, Surface,
    SurfaceConfiguration, TextureFormat, TextureUsages,
};
use winit::dpi::{PhysicalSize, Size};
use winit::error::OsError;
use winit::event_loop::ActiveEventLoop;
use winit::window::WindowAttributes;

pub struct WindowConfiguration {
    pub width: u32,
    pub height: u32,
    pub title: &'static str,
}

pub struct Window {
    pub(super) surface: Surface<'static>,
    surface_config: OnceCell<SurfaceConfiguration>,
    window_config: WindowConfiguration,
    pub(super) window: Arc<winit::window::Window>,
}

#[derive(Error, Debug)]
pub enum CreateWindowError {
    #[error("Failed to create window: {0}")]
    OsError(#[from] OsError),
    #[error("Failed to create surface: {0}")]
    CreateSurfaceError(#[from] CreateSurfaceError),
}

impl Window {
    pub fn new(
        instance: &Instance,
        event_loop: &ActiveEventLoop,
        window_config: WindowConfiguration,
    ) -> Result<Self, CreateWindowError> {
        let window = Arc::new(event_loop.create_window(
            WindowAttributes::default()
                .with_inner_size(PhysicalSize::new(window_config.width, window_config.height))
                .with_title(window_config.title),
        )?);
        let surface = instance.create_surface(Arc::clone(&window))?;

        Ok(
            Self {
                surface,
                surface_config: OnceCell::new(),
                window_config,
                window
            }
        )
    }

    pub(super) fn configure(&mut self, renderer: &Renderer) {
        if self.surface_config.get().is_none() {
            let surface_caps = self.surface.get_capabilities(&renderer.adapter);
            let surface_format = surface_caps
                .formats
                .iter()
                .copied()
                .find(|f| f.is_srgb())
                .unwrap_or(surface_caps.formats[0]);

            let config = SurfaceConfiguration {
                usage: TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: self.window_config.width,
                height: self.window_config.height,
                present_mode: PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: CompositeAlphaMode::Auto,
                view_formats: vec![],
            };

            self.surface_config.set(config).unwrap_or_else(|err| {
                warn!(
                    "Failed to set surface config, retrying next frame. {:?}",
                    err
                );
            });
        }

        self.surface
            .configure(&renderer.device, self.surface_config.get().unwrap());
    }

    pub fn resize(&mut self, renderer: &Renderer, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.window_config.width = width;
            self.window_config.height = height;
            self.configure(renderer);
        }
    }

    pub fn request_redraw(&self) {
        self.window.request_redraw();
    }

    pub fn acquire_render_target(&mut self, renderer: &Renderer) -> Option<RenderTarget> {
        let current_surface_texture = self.surface.get_current_texture();
        let surface_texture = match current_surface_texture {
            CurrentSurfaceTexture::Success(st) => st,
            CurrentSurfaceTexture::Suboptimal(st) => {
                warn!("Suboptimal surface texture!");
                self.configure(renderer);
                st
            }
            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Occluded
            | CurrentSurfaceTexture::Validation => {
                return None;
            }
            CurrentSurfaceTexture::Outdated => {
                warn!("Outdated surface texture!");
                self.configure(renderer);
                return None;
            }
            CurrentSurfaceTexture::Lost => {
                error!("Lost device!");
                abort()
            }
        };

        Some(RenderTarget::Surface {
            view: surface_texture
                .texture
                .create_view(&TextureViewDescriptor::default()),
            surface_texture,
        })
    }
}
