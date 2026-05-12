use wgpu::{SurfaceTexture, Texture, TextureView};
use crate::rendering::render_target::RenderTarget::Surface;

pub enum RenderTarget {
    Surface {
        surface_texture: SurfaceTexture,
        view: TextureView,
    },

    Texture {
        view: TextureView,
    },
}

impl RenderTarget {
    pub(super) fn view(&self) -> &TextureView {
        match self {
            RenderTarget::Surface { view, .. } => { view }
            RenderTarget::Texture { view } => { view }
        }
    }

    pub(super) fn surface_texture(&self) -> Option<&SurfaceTexture> {
        match self {
            RenderTarget::Surface { surface_texture, .. } => { Some(surface_texture) }
            RenderTarget::Texture { .. } => { None }
        }
    }

    pub(super) fn present(self) {
        match self {
            RenderTarget::Surface { view, surface_texture} => { surface_texture.present(); }
            RenderTarget::Texture { view} => { }
        }
    }
}
