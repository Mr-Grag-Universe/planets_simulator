use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;
use crate::app::graphics::gpu_resources::GPU_Resources;
use crate::app::graphics::screen::Screen;
use bytemuck::{Pod, Zeroable};
use glam;
use std::f64::consts::PI;
use std::borrow::Cow;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
}

fn vertex(pos: [i8; 3], x: [i8; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
    }
}

struct Cube {
    pub size: f32,
    pub color: wgpu::Color,
}

impl Cube {
    // pub fn create_v_i(&self) -> (Vec<Vertex>, Vec<u16>) {
    //     let vertices = [
    //         // bottom
    //         vertex([-1, -1, -1]),
    //         vertex([1, -1, -1]),
    //         vertex([-1, 1, -1]),
    //         vertex([1, 1, -1]),
    //         // top
    //         vertex([-1, -1, 1]),
    //         vertex([1, -1, 1]),
    //         vertex([-1, 1, 1]),
    //         vertex([1, 1, 1]),
    //     ];

    //     let indices: &[u16] = &[
    //         0, 1, 3, 0, 2, 3, // bottom (xy-plane)
    //         4, 5, 7, 4, 6, 7, // top (xy-plane + 1)
    //         0, 1, 5, 0, 4, 5, // (xz-plane)
    //         2, 3, 7, 2, 6, 7, // (xz-plane + 1)
    //         0, 2, 6, 0, 4, 6, // (yz-plane)
    //         1, 3, 7, 1, 5, 7, // (yz-plane + 1)
    //     ];
        
    //     (vertices.to_vec(), indices.to_vec())
    // }

    fn create_v_i(&self) -> (Vec<Vertex>, Vec<u16>) {
        let vertex_data = [
            // top (0, 0, 1)
            vertex([-1, -1, 1], [0, 0]),
            vertex([1, -1, 1], [1, 0]),
            vertex([1, 1, 1], [1, 1]),
            vertex([-1, 1, 1], [0, 1]),
            // bottom (0, 0, -1)
            vertex([-1, 1, -1], [1, 0]),
            vertex([1, 1, -1], [0, 0]),
            vertex([1, -1, -1], [0, 1]),
            vertex([-1, -1, -1], [1, 1]),
            // right (1, 0, 0)
            vertex([1, -1, -1], [0, 0]),
            vertex([1, 1, -1], [1, 0]),
            vertex([1, 1, 1], [1, 1]),
            vertex([1, -1, 1], [0, 1]),
            // left (-1, 0, 0)
            vertex([-1, -1, 1], [1, 0]),
            vertex([-1, 1, 1], [0, 0]),
            vertex([-1, 1, -1], [0, 1]),
            vertex([-1, -1, -1], [1, 1]),
            // front (0, 1, 0)
            vertex([1, 1, -1], [1, 0]),
            vertex([-1, 1, -1], [0, 0]),
            vertex([-1, 1, 1], [0, 1]),
            vertex([1, 1, 1], [1, 1]),
            // back (0, -1, 0)
            vertex([1, -1, 1], [0, 0]),
            vertex([-1, -1, 1], [1, 0]),
            vertex([-1, -1, -1], [1, 1]),
            vertex([1, -1, -1], [0, 1]),
        ];

        let index_data: &[u16] = &[
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 10, 11, 8, // right
            12, 13, 14, 14, 15, 12, // left
            16, 17, 18, 18, 19, 16, // front
            20, 21, 22, 22, 23, 20, // back
        ];

        (vertex_data.to_vec(), index_data.to_vec())
    }
}


pub struct StateCube {
    pub screen: Screen,
    pub cube: Cube,
    pub resources: Arc<GPU_Resources>,

    
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}


fn generate_matrix(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::perspective_rh(PI as f32 / 4.0, aspect_ratio, 1.0, 10.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(1.5f32, -5.0, 3.0),
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    projection * view
}


impl StateCube {
    pub fn configure_surface(&self) {
        self.screen.configure_surface();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.screen.resize(new_size);
    }

    pub fn new(window: Arc<Window>, resources: Arc<GPU_Resources>) -> StateCube {
        let mut screen = Screen::new(window.clone(), resources.clone());
        let cube = Cube { size: 1.0, color: wgpu::Color::BLUE, };

        let vertex_size = size_of::<Vertex>();
        let (vertex_data, index_data) = cube.create_v_i();

        // create buffers for vertices and indices to pass to shading phase
        let vertex_buf = resources.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_buf = resources.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&index_data),
            usage: wgpu::BufferUsages::INDEX,
        });

        // Create pipeline layout
        let bind_group_layout = resources.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(64),
                    },
                    count: None,
                }
            ],
        });
        let pipeline_layout = resources.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let mx_total = generate_matrix(screen.get_ratio());
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = resources.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(mx_ref),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        // Create bind group
        let bind_group = resources.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = resources.device.create_shader_module(wgpu::include_wgsl!("shaders/cube.wgsl"));

        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
            ],
        }];

        let pipeline = resources.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &vertex_buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(screen.surface.get_format().into())],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });


        screen.set_bg_color(wgpu::Color::BLACK);
        screen.configure_surface();

        StateCube {
            screen,
            cube,
            resources: resources.clone(),
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group,
            uniform_buf,
            pipeline,
        }
    }

    pub fn render(&mut self) {
        let mut encoder = self.resources.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        let surface_texture = self
            .screen.surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                // Without add_srgb_suffix() the image we will be working with
                // might not be "gamma correct".
                format: Some(self.screen.surface.get_format().add_srgb_suffix()),
                ..Default::default()
            });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &texture_view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        self.resources.queue.submit(Some(encoder.finish()));
        self.screen.window.pre_present_notify();
        surface_texture.present();
    }
}