use std::sync::Arc;
use winit::window::Window;
use crate::app::graphics::gpu_resources::GPU_Resources;
use crate::app::graphics::screen::Screen;
use std::borrow::Cow;

pub struct StateTriangle {
    pub screen: Screen,
}

impl StateTriangle {
    pub fn new(window: Arc<Window>, resources: Arc<GPU_Resources>) -> StateTriangle {
        let mut state = StateTriangle {
            screen: Screen::new(window.clone(), resources.clone()),
        };
        state.screen.set_bg_color(wgpu::Color::BLACK);
        state
    }

    pub fn configure_surface(&self) {
        self.screen.configure_surface();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.screen.resize(new_size);
    }

    pub fn init() {
    }

    pub fn render(&mut self) {
        let pipeline_layout = self.screen.resources.create_pipeline_layout();
        let shader = self.screen.resources.device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shaders/triangle.wgsl"))),
        });
        let render_pipeline = self.screen.resources.create_render_pipeline(&pipeline_layout, &shader, self.screen.surface.get_format());

        self.screen.render(|rpass, render_pipeline| {
            rpass.set_pipeline(&render_pipeline);
            rpass.draw(0..3, 0..1);

        }, Some(render_pipeline));
    }
}
