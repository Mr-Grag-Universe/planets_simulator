use std::sync::Arc;
use winit::window::Window;

use crate::app::graphics::gpu_resources::GPU_Resources;
use crate::app::graphics::surface::Surface;

pub struct Screen {
    pub window: Arc<Window>,
    pub resources: Arc<GPU_Resources>,
    pub surface: Surface,
    size: winit::dpi::PhysicalSize<u32>,
    background_color: wgpu::Color,
}


impl Screen {
    pub fn new(window: Arc<Window>, resources: Arc<GPU_Resources>) -> Screen {
        let surface = Surface::new(resources.clone(), window.clone());

        let size = window.inner_size();

        let state = Screen {
            window,
            resources,
            size,
            surface,
            background_color: wgpu::Color::BLACK,
        };
        state.configure_surface();

        state
    }

    pub fn get_window(&self) -> &Window {
        &self.window
    }

    pub fn get_width(&self) -> u32 {
        self.size.width
    }
    pub fn get_height(&self) -> u32 {
        self.size.height
    }
    pub fn get_ratio(&self) -> f32 {
        return self.size.width as f32 / self.size.height as f32
    }

    pub fn set_bg_color(&mut self, color: wgpu::Color) {
        self.background_color = color
    }
    pub fn get_bg_color(&self) -> wgpu::Color {
        self.background_color
    }

    pub fn configure_surface(&self) {
        let format = self.surface.get_format();
        let format = wgpu::TextureFormat::Depth32Float;
        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: format.clone(),
            // Request compatibility with the sRGB-format texture view we‘re going to create later.
            view_formats: vec![format.add_srgb_suffix()],
            alpha_mode: wgpu::CompositeAlphaMode::Auto,
            width: self.size.width,
            height: self.size.height,
            desired_maximum_frame_latency: 2,
            present_mode: wgpu::PresentMode::AutoVsync,
        };
        self.surface.configure(&self.resources, &surface_config);
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.size = new_size;
        self.configure_surface();
    }

    
    pub fn render<F>(&mut self, process_renderpass: F, pipeline: Option<wgpu::RenderPipeline>)
    where
        F: Fn(&mut wgpu::RenderPass<'_>, wgpu::RenderPipeline),
    {
        // Create texture view
        let surface_texture = self
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.surface.get_format().add_srgb_suffix()),
                ..Default::default()
            });

        // Renders a GREEN screen
        let mut encoder = self.resources.create_command_encoder(&Default::default());
        // Create the renderpass which will clear the screen.
        let mut renderpass = 
            encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(self.background_color),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                // multiview_mask: None,
            });

        // If you wanted to call any drawing commands, they would go here.
        if let Some(pipeline) = pipeline {
            process_renderpass(&mut renderpass, pipeline);
        }

        // End the renderpass.
        drop(renderpass);

        // Submit the command in the queue to execute
        self.resources.submit_to_queue([encoder.finish()]);
        self.window.pre_present_notify();
        surface_texture.present();
    }
}