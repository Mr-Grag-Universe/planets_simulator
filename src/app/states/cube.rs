use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::app::graphics::gpu_resources::GPU_Resources;
use crate::app::graphics::screen::Screen;
use bytemuck::{Pod, Zeroable};
use glam;
use std::f64::consts::PI;
use std::borrow::Cow;

struct Cube {
    pub size: f32,
    pub color: wgpu::Color,
}

pub struct StateCube {
    pub screen: Screen,
    pub cube: Cube,
}


impl StateCube {
    pub fn new(window: Arc<Window>, resources: Arc<GPU_Resources>) -> StateCube {
        let mut state = StateCube {
            screen: Screen::new(window.clone(), resources.clone()),
            cube: Cube{ size: 1.0, color: wgpu::Color::BLUE },
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
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });
        let render_pipeline = self.screen.resources.create_render_pipeline(&pipeline_layout, &shader, self.screen.surface.get_format());

        self.screen.render(|rpass, render_pipeline| {
            // Установите пайплайн
            // renderpass.set_pipeline(&self.your_pipeline);

            // Передайте куб с его размерами и цветом
            // Пример отрисовки:
            // renderpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // renderpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
            // renderpass.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0);

            // rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&render_pipeline);
            rpass.draw(0..3, 0..1);

        }, Some(render_pipeline));
    }
}
