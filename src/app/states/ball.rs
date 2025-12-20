use std::sync::Arc;
use wgpu::{Buffer, PipelineLayout};
use wgpu::naga::common::wgsl;
use wgpu::util::DeviceExt;
use winit::window::Window;
use bytemuck::{Pod, Zeroable};
use glam;
use std::f64::consts::PI;
use std::borrow::Cow;

use crate::app::graphics::gpu_resources::GPU_Resources;
use crate::app::graphics::screen::Screen;
use crate::physics::geometry::{Geometry, Mesh, generate_transform};
use crate::physics::ball::Ball;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
}

struct Entity {
    mx_world: glam::Mat4,
    color: wgpu::Color,
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_format: wgpu::IndexFormat,
    index_count: usize,
    uniform_offset: wgpu::DynamicOffset,
}

struct GraphicsTools {
    resources: Option<Arc<GPU_Resources>>,

    vertex_buf: Option<wgpu::Buffer>,
    index_buf: Option<wgpu::Buffer>,
    pub index_count: usize,
    bind_group: Option<wgpu::BindGroup>,
    uniform_buf: Option<wgpu::Buffer>,
    pipeline_layout: Option<wgpu::PipelineLayout>,
    pipeline: Option<wgpu::RenderPipeline>,
}

pub struct StateBall {
    pub screen: Screen,
    pub ball: Ball,
    pub resources: Arc<GPU_Resources>,
    pub gtools: GraphicsTools
}


impl StateBall {
    pub fn configure_surface(&self) {
        self.screen.configure_surface();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.screen.resize(new_size);
    }

    pub fn new(window: Arc<Window>, resources: Arc<GPU_Resources>) -> StateBall {
        let mut screen = Screen::new(window.clone(), resources.clone());
        screen.set_bg_color(wgpu::Color::BLACK);
        screen.configure_surface();

        let ball = Ball::new(1.0);

        let mut gtools = GraphicsTools::default();

        let mut state = StateBall { 
            screen, 
            ball, 
            resources: resources.clone(), 
            gtools 
        };
        state.init();
        
        state
    }

    fn create_surface_entity(&self) -> Entity {
        let (vertices, indices) = self.get_vertices_indices_surface();
        let v_buf = self.resources.buffer_fabric.create_vertex_buffer(&vertices, None);
        let i_buf = self.resources.buffer_fabric.create_index_buffer(&indices, None);

        Entity {
            mx_world: generate_transform(self.screen.get_ratio()),
            color: wgpu::Color::WHITE,
            vertex_buf: v_buf,
            index_buf: i_buf,
            index_format: wgpu::IndexFormat::Uint16,
            index_count: indices.len(),
            uniform_offset: 0
        }
    }

    pub fn init(&mut self) {
        self.gtools.init(self.resources.clone());

        let vertex_size = size_of::<Vertex>();
        let (vertices, indices) = self.get_vertices_indices_surface();
        let vertex_buf = self.resources.buffer_fabric.create_vertex_buffer(&vertices, None);
        let index_buf = self.resources.buffer_fabric.create_index_buffer(&indices, None);
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

        self.get_vertices_indices_edges();

        let bind_group_layout = self.resources.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
        let pipeline_layout = self.resources.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        
        let mx_total = generate_transform(self.screen.get_ratio());
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = self.resources.buffer_fabric.create_buffer(
            mx_ref, 
            "Uniform Buffer", 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        );

        // Create bind group
        let bind_group = self.resources.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = self.resources.device.create_shader_module(wgpu::include_wgsl!("shaders/ball.wgsl"));
        
        self.gtools.set_vertex_buffer(vertex_buf);
        self.gtools.set_index_buffer(index_buf);
        self.gtools.set_bind_group(bind_group);
        self.gtools.set_pipeline_layout(pipeline_layout);
        self.gtools.index_count = indices.len();
        self.gtools.init_pipeline(shader, &vertex_buffers, &[Some(self.screen.surface.get_format().into())]);
    }

    fn transform_mesh_to_vertices_indices(mesh: Mesh) -> (Vec<Vertex>, Vec<u16>) {
        let vertices = mesh.vertices.iter().map(|v| {
            Vertex { _pos: [v[0] as f32, v[1] as f32, v[2] as f32, 1.0] }
        }).collect();

        let mut indices = Vec::new();
        for index_triplet in mesh.indices {
            indices.push(index_triplet[0] as u16);
            indices.push(index_triplet[1] as u16);
            indices.push(index_triplet[2] as u16);
        }

        (vertices, indices)
    }

    pub fn get_vertices_indices_surface(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mesh = self.ball.get_surface_mesh();
        Self::transform_mesh_to_vertices_indices(mesh)
    }

    pub fn get_vertices_indices_edges(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mesh = self.ball.get_edges_mesh(0.1);
        Self::transform_mesh_to_vertices_indices(mesh)
    }

    pub fn render(&mut self) {
        self.gtools.render(&self.screen)
    }
}

impl GraphicsTools {
    pub fn init(&mut self, resources: Arc<GPU_Resources>) {
        self.resources = Some(resources.clone());
    }

    pub fn set_pipeline_layout(&mut self, pipeline_layout: wgpu::PipelineLayout) {
        self.pipeline_layout = Some(pipeline_layout);
    }
    pub fn set_bind_group(&mut self, bind_group: wgpu::BindGroup) {
        self.bind_group = Some(bind_group);
    }
    pub fn set_index_buffer(&mut self, buf: wgpu::Buffer) {
        self.index_buf = Some(buf);
    }
    pub fn set_vertex_buffer(&mut self, buf: wgpu::Buffer) {
        self.vertex_buf = Some(buf);
    }

    pub fn init_pipeline(&mut self, 
                         shader: wgpu::ShaderModule, 
                         vertex_buffers: &[wgpu::VertexBufferLayout],
                         fragment_target: &[Option<wgpu::ColorTargetState>]) 
    {
        let pipeline = self.resources.as_ref().unwrap().device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor 
        {
                label: None,
                layout: Some(&self.pipeline_layout.as_ref().unwrap()),
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
                    targets: &fragment_target,
                }),
                primitive: wgpu::PrimitiveState {
                    cull_mode: None, // Some(wgpu::Face::Back),
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState::default(),
                multiview: None,
                cache: None,
        });
        self.pipeline = Some(pipeline);
    }

    pub fn render(&self, screen: &Screen) {
        let mut encoder = self.resources.as_ref().unwrap().create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        let surface_texture = screen
            .surface
            .get_current_texture()
            .expect("failed to acquire next swapchain texture");
        let texture_view = surface_texture
            .texture
            .create_view(&wgpu::TextureViewDescriptor {
                format: Some(screen.surface.get_format().add_srgb_suffix()),
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
                        load: wgpu::LoadOp::Clear(screen.get_bg_color()),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline.as_ref().unwrap());
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.as_ref().unwrap().slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.vertex_buf.as_ref().unwrap().slice(..));
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        self.resources
            .as_ref().unwrap()
            .submit_to_queue(Some(encoder.finish()));
        screen.window.pre_present_notify();
        surface_texture.present();
    }
}

impl Default for GraphicsTools {
    fn default() -> Self {
        GraphicsTools {
            resources: None,
            vertex_buf: None,
            index_buf: None,
            index_count: 0,
            bind_group: None,
            uniform_buf: None,
            pipeline: None,
            pipeline_layout: None
        }
    }
}


