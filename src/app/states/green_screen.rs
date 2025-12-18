use std::sync::Arc;
use winit::window::Window;

use crate::app::graphics::gpu_resources::GPU_Resources;
use crate::app::graphics::screen::Screen;

pub struct StateGreenScreen {
    pub screen: Screen
}

impl StateGreenScreen {
    pub fn new(window: Arc<Window>, resources: Arc<GPU_Resources>) -> StateGreenScreen {
        let mut state = StateGreenScreen {
            screen: Screen::new(window.clone(), resources.clone())
        };
        state.screen.set_bg_color(wgpu::Color::BLUE);
        state
    }

    pub fn configure_surface(&self) {
        self.screen.configure_surface()
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.screen.resize(new_size)
    }

    pub fn render(&mut self) {
        self.screen.render(|renderpass, pipeline| {
            // Ваша логика отрисовки здесь, например:
            // renderpass.set_pipeline(&self.your_pipeline);
            // renderpass.draw_indexed(0..CUBE_INDICES.len() as u32, 0, 0);
        }, None);
    }
}