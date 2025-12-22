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
use crate::app::graphics::surface;
use crate::physics::geometry::{Geometry, Mesh, generate_transform};
use crate::physics::ball::Ball;


#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _col: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4],
    light_direction: [f32; 3],
    _padding1: f32,  // Выравнивание до 16 байт
    light_color: [f32; 3],
    _padding2: f32,  // Выравнивание до 16 байт
    ambient_strength: f32,
    _padding3: [f32; 3],  // Добиваем до 16 байт
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

    bind_group: Option<wgpu::BindGroup>,
    pipeline_layout: Option<wgpu::PipelineLayout>,
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,

    uniform_buf: Option<wgpu::Buffer>,
    uniform_buf_flag_true: Option<wgpu::Buffer>,
    uniform_buf_flag_false: Option<wgpu::Buffer>,

    entities: Vec<Entity>,
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
        self.init();
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

    fn create_entity(&self, color: wgpu::Color) -> Entity {
        let (vertices_s, indices_s) = self.get_vertices_indices_surface();
        let (vertices_e, indices_e) = self.get_vertices_indices_edges();

        let length_s = vertices_s.len() as u16;
        let vertices: Vec<Vertex> = [vertices_s, vertices_e].concat();
        let indices: Vec<u16> = indices_s
            .iter()
            .cloned()
            .chain(indices_e.iter().map(|&x| x + length_s))
            .collect();
        assert_eq!(indices_s.len() + indices_e.len(), indices.len(), "Lengthes must be the same!");

        let v_buf = self.resources.buffer_fabric.create_vertex_buffer_init(&vertices, None);
        let i_buf = self.resources.buffer_fabric.create_index_buffer_init(&indices, None);
        let mx_total = generate_transform(self.screen.get_ratio());

        Entity {
            mx_world: mx_total,
            color: color,
            vertex_buf: v_buf,
            index_buf: i_buf,
            index_format: wgpu::IndexFormat::Uint16,
            index_count: indices.len(),
            uniform_offset: 0
        }
    }

    pub fn init(&mut self) {
        self.gtools.init(self.resources.clone());

        // let (vertices, indices) = self.get_vertices_indices_surface();
        // let vertex_buf = self.resources.buffer_fabric.create_vertex_buffer(&vertices, None);
        // let index_buf = self.resources.buffer_fabric.create_index_buffer(&indices, None);
        let entity = self.create_entity(wgpu::Color::WHITE);
        self.gtools.push_entity(entity);

        let bind_group_layout = self.resources.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::all(),
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<Uniforms>() as u64),
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = self.resources.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        
        let mx_total = generate_transform(self.screen.get_ratio());
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        
        let uniforms = [Uniforms {
            transform: mx_total.to_cols_array_2d(),
            light_direction: [0.0, 0.0, -1.0],
            _padding1: 0.0,
            light_color: [1.0, 1.0, 1.0],
            _padding2: 0.0,
            ambient_strength: 0.3,
            _padding3: [0.0; 3],
        }];
        let uniform_buf = self.resources.buffer_fabric.create_buffer_init(
            &uniforms, 
            "Uniform Buffer", 
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
        );
        self.gtools.uniform_buf = Some(uniform_buf);

        let vertex_size = size_of::<Vertex>();
        let vertex_buffers = [wgpu::VertexBufferLayout {
            array_stride: vertex_size as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 0,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: size_of::<[f32; 4]>() as wgpu::BufferAddress,
                    shader_location: 1,
                },
            ],
        }];

        
        let bind_group = self.screen.resources.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout.clone(),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.gtools.uniform_buf.clone().unwrap().as_entire_binding(),
                },
            ],
            label: None,
        });

        let shader = self.resources.device.create_shader_module(wgpu::include_wgsl!("shaders/ball.wgsl"));
        
        self.gtools.set_pipeline_layout(pipeline_layout);
        self.gtools.set_bind_group(bind_group);
        self.gtools.set_bind_group_layout(bind_group_layout);
        self.gtools.init_pipeline(shader, &vertex_buffers, &[Some(self.screen.surface.get_format().into())]);
    }

    fn transform_mesh_to_vertices_indices(mesh: Mesh, color: wgpu::Color) -> (Vec<Vertex>, Vec<u16>) {
        let vertices = mesh.vertices.iter().map(|v| {
            Vertex { 
                _pos: [v[0] as f32, v[1] as f32, v[2] as f32, 1.0], 
                _col: [color.r as f32, color.g as f32, color.b as f32, color.a as f32],
            }
        }).collect();

        let mut indices = Vec::new();
        for index_triplet in mesh.indices {
            indices.push(index_triplet[0] as u16);
            indices.push(index_triplet[1] as u16);
            indices.push(index_triplet[2] as u16);
        }

        (vertices, indices)
    }

    fn get_vertices_indices_surface(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mesh = self.ball.get_surface_mesh();
        Self::transform_mesh_to_vertices_indices(mesh, wgpu::Color{r:0.0, g:1.0, b:1.0, a:1.0})
    }

    fn get_vertices_indices_edges(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mesh = self.ball.get_edges_mesh(0.01);
        Self::transform_mesh_to_vertices_indices(mesh, wgpu::Color{r:1.0, g:0.0, b:1.0, a:1.0})
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
    pub fn set_bind_group_layout(&mut self, bind_group_layout: wgpu::BindGroupLayout) {
        self.bind_group_layout = Some(bind_group_layout);
    }
    pub fn set_bind_group(&mut self, bind_group: wgpu::BindGroup) {
        self.bind_group = Some(bind_group);
    }
    pub fn push_entity(&mut self, entity: Entity) {
        self.entities.push(entity)
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
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth32Float,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::Less,
                    stencil: wgpu::StencilState::default(),
                    bias: wgpu::DepthBiasState::default(),
                }),
                // depth_stencil: None,
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
                // format: Some(wgpu::TextureFormat::Depth32Float),
                ..Default::default()
            });

        let depth_texture = self.resources.clone().unwrap().device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth_texture"),
            size: wgpu::Extent3d {
                width: screen.get_width(),
                height: screen.get_height(),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let depth_view = depth_texture.create_view(&wgpu::TextureViewDescriptor::default());

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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0), // очистить максимальным значением глубины
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            rpass.push_debug_group("Prepare data for draw.");
            rpass.set_pipeline(&self.pipeline.as_ref().unwrap());
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.entities[0].index_buf.slice(..), wgpu::IndexFormat::Uint16);
            rpass.set_vertex_buffer(0, self.entities[0].vertex_buf.slice(..));
            rpass.draw_indexed(0..self.entities[0].index_count as u32, 0, 0..1);
            rpass.pop_debug_group();
            rpass.insert_debug_marker("Draw!");
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
            bind_group: None,
            pipeline: None,
            pipeline_layout: None,
            entities: vec![],
            bind_group_layout: None,

            uniform_buf: None,
            uniform_buf_flag_true: None,
            uniform_buf_flag_false: None,
        }
    }
}


