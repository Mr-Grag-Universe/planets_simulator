use wgpu;
use std::sync::Arc;
use winit::window::Window;

use crate::app::graphics::gpu_resources::GPU_Resources;

pub struct Surface {
    pub surface: wgpu::Surface<'static>,
    pub surface_format: wgpu::TextureFormat,
}

impl Surface {
    pub fn new(resources: Arc<GPU_Resources>, window: Arc<Window>) -> Surface {
        let surface = resources.instance.create_surface(window.clone()).unwrap();
        let cap = surface.get_capabilities(&resources.adapter);
        let surface_format = cap.formats[0];

        Surface { surface, surface_format }
    }

    pub fn get_format(&self) -> wgpu::TextureFormat {
        self.surface_format
    }

    pub fn configure(&self, resources: &GPU_Resources, config: &wgpu::SurfaceConfiguration) {
        self.surface.configure(&resources.device, config);
    }

    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }
}