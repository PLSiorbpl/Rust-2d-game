use wgpu::{SurfaceTexture, TextureView};

pub enum RenderTarget {
    Surface {
        surface_texture: SurfaceTexture,
        view: TextureView,
    },

    Texture {
        view: TextureView,
    },
}
