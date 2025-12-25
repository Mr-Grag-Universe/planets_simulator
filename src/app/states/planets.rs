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
use crate::app::graphics::graphycs_geometry::GraphicsGeometry;
use crate::physics::geometry::{Mesh, Point3};
use crate::physics::ball::Ball;
use crate::physics::coords::Coord;
use crate::app::graphics::planet::Planet;
use serde;
use std::fs;
use std::error::Error;
use image;
use image::GenericImageView;

#[derive(Debug, serde::Deserialize)]
struct json_Planet {
    name: String,
    year_dur_re: u32,
    R_au: f64,
    radius_re: f64,
    texture_path: String,
    move_direction: String,
    day_dur_re: f64,
    is_giant: bool
}

#[derive(Debug, serde::Deserialize)]
struct json_Config {
    planets: Vec<json_Planet>,
}


fn load_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: &str,
) -> Result<wgpu::TextureView, Box<dyn Error>> {
    // Загружаем изображение
    let img = image::ImageReader::open(path)?.decode()?;
    let rgba = img.to_rgba8();
    let dimensions = img.dimensions();

    // Создаём описание текстуры
    let texture_size = wgpu::Extent3d {
        width: dimensions.0,
        height: dimensions.1,
        depth_or_array_layers: 1,
    };

    // Создаём текстуру на GPU
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(&format!("Texture: {}", path)),
        size: texture_size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,  // Формат с гамма-коррекцией
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });

    // Копируем данные изображения в текстуру GPU
    queue.write_texture(
        // Вместо wgpu::ImageCopyTexture используем wgpu::TexelCopyTextureInfo
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &rgba,
        // Вместо wgpu::ImageDataLayout используем wgpu::TexelCopyBufferLayout
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            // 4 байта на пиксель (RGBA)
            // ПРИМЕЧАНИЕ: bytes_per_row должен быть кратен 256 для оптимальной производительности[citation:1]
            bytes_per_row: Some(4 * dimensions.0),
            rows_per_image: Some(dimensions.1),
        },
        texture_size,
    );

    // Создаём "вид" (view) текстуры для использования в шейдере (без изменений)
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
    Ok(view)
}

fn load_config(file_path: &str) -> Result<json_Config, Box<dyn Error>> {
    let contents = fs::read_to_string(file_path)?;
    let config: json_Config = serde_json::from_str(&contents)?;
    Ok(config)
}


const ORIGIN_POS: [f32; 3] = [0.0, 0.0, 0.0];
const BASE_ANGLE_SPEED: f32 = PI as f32 / 40.0;
const R: f32 = 100.0;
const PLANET_RADIUS: f64 = 2.0;


pub fn generate_transform(aspect_ratio: f32) -> glam::Mat4 {
    let projection = glam::Mat4::perspective_rh(PI as f32 / 4.0, aspect_ratio, 1.0, -1.0);
    let view = glam::Mat4::look_at_rh(
        glam::Vec3::new(R*6.0, R*4.0, R*6.0),
        glam::Vec3::ZERO,
        glam::Vec3::Z,
    );
    projection * view
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    _pos: [f32; 4],
    _col: [f32; 4],
    _normal: [f32; 3],
    _uv: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    transform: [[f32; 4]; 4],
    light_origin: [f32; 3],
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
    bind_groups: Option<Vec<wgpu::BindGroup>>,
    pipeline_layout: Option<wgpu::PipelineLayout>,
    pipeline: Option<wgpu::RenderPipeline>,
    bind_group_layout: Option<wgpu::BindGroupLayout>,

    uniform_buf: Option<wgpu::Buffer>,

    entities: Vec<Entity>,
}

pub struct StatePlanets {
    pub screen: Screen,
    pub planets: Vec<Planet>,
    pub resources: Arc<GPU_Resources>,
    pub gtools: GraphicsTools,
    
    pub planet_textures: Vec<wgpu::TextureView>,
    pub texture_sampler: Option<wgpu::Sampler>,
}


impl StatePlanets {
    pub fn configure_surface(&self) {
        self.screen.configure_surface();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.screen.resize(new_size);
        self.init();
    }

    pub fn load_planets(resources: Arc<GPU_Resources>) -> (Vec<Planet>, Vec<wgpu::TextureView>) {
        let config = load_config("src/app/states/configs/planets.json").unwrap();

        let mut planet_textures = Vec::new();
        let mut planets = Vec::new();
        for (i, json_planet) in config.planets.iter().enumerate() {
            let texture_view = load_texture(
                &resources.device,
                &resources.queue,
                &json_planet.texture_path,
            ).unwrap_or_else(|_| {
                // Если текстура не загрузилась, создаём розовую текстуру 1x1
                eprintln!("Failed to load texture: {}", json_planet.texture_path);
                
                let texture = resources.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("fallback_texture"),
                    size: wgpu::Extent3d { 
                        width: 1, 
                        height: 1, 
                        depth_or_array_layers: 1 
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                });
                
                // Розовый цвет для отладки
                let rgba = [255, 0, 255, 255];
                resources.queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: 0,
                        origin: wgpu::Origin3d::ZERO,
                        aspect: wgpu::TextureAspect::All,
                    },
                    &rgba,
                    wgpu::TexelCopyBufferLayout {
                        offset: 0,
                        bytes_per_row: Some(4),
                        rows_per_image: Some(1),
                    },
                    wgpu::Extent3d { 
                        width: 1, 
                        height: 1, 
                        depth_or_array_layers: 1 
                    },
                );
                
                texture.create_view(&wgpu::TextureViewDescriptor::default())
            });
            
            planet_textures.push(texture_view);

            let ball = Ball::new(1.0);
            let mut scale = json_planet.radius_re;
            if json_planet.is_giant {
                scale = scale.sqrt();
            }
            scale *=  PLANET_RADIUS;
            let planet = GraphicsGeometry::new(
                Box::new(ball), 
                (0.0, 0.0, 0.0), // identity rotation
                scale, 
                Point3::new((ORIGIN_POS[0] + (R*json_planet.R_au as f32)) as f64, ORIGIN_POS[1] as f64, ORIGIN_POS[2] as f64)
            );
            let mut planet = Planet { 
                geom_obj: planet, 
                texture: 0, 
                angle_speed: BASE_ANGLE_SPEED * 365.0 / json_planet.year_dur_re as f32 
            };
            if json_planet.move_direction == "ccw" {
                planet.angle_speed *= -1.0;
            }
            planets.push(planet);
        }

        (planets, planet_textures)
    }

    pub fn new(window: Arc<Window>, resources: Arc<GPU_Resources>) -> StatePlanets {
        let mut screen = Screen::new(window.clone(), resources.clone());
        screen.set_bg_color(wgpu::Color::BLACK);
        screen.configure_surface();


        let mut gtools = GraphicsTools::default();

        let (planets, planet_textures) = Self::load_planets(resources.clone());
        let texture_sampler = resources.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("texture_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });


        let mut state = StatePlanets { 
            screen, 
            planets, 
            resources: resources.clone(), 
            gtools,
            planet_textures,
            texture_sampler: Some(texture_sampler)
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

    fn create_entity_for_planet(&self, planet_index: usize) -> Entity {
        let planet = &self.planets[planet_index];
        
        // Получаем mesh только для одной планеты
        let mesh_surface = planet.geom_obj.get_surface();
        let mesh_edges = planet.geom_obj.get_edges(0.01);
        
        // Преобразуем в вершины и индексы
        let (vertices_s, indices_s) = Self::transform_mesh_to_vertices_indices(
            mesh_surface, 
            wgpu::Color{r:0.0, g:1.0, b:1.0, a:1.0}
        );
        let (vertices_e, indices_e) = Self::transform_mesh_to_vertices_indices(
            mesh_edges, 
            wgpu::Color{r:1.0, g:0.0, b:1.0, a:1.0}
        );
        
        let length_s = vertices_s.len() as u16;
        let vertices: Vec<Vertex> = [vertices_s, vertices_e].concat();
        let indices: Vec<u16> = indices_s
            .iter()
            .cloned()
            .chain(indices_e.iter().map(|&x| x + length_s))
            .collect();
        
        let v_buf = self.resources.buffer_fabric.create_vertex_buffer_init(&vertices, None);
        let i_buf = self.resources.buffer_fabric.create_index_buffer_init(&indices, None);
        
        Entity {
            mx_world: glam::Mat4::IDENTITY, // Это больше не используется в вашем шейдере
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
        self.init_entities();


        let bind_group_layout = self.resources.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                // Uniform буфер
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
                // Сэмплер
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Текстура
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
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
            light_origin: [0.0, 0.0, 0.0],
            _padding1: 0.0,
            light_color: [1.0, 1.0, 1.0],
            _padding2: 0.0,
            ambient_strength: 1.0,
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
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x3,
                    offset: (size_of::<[f32; 4]>()*2) as wgpu::BufferAddress,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,  // ← UV-координаты!
                    offset: (size_of::<[f32; 4]>() * 2 + size_of::<[f32; 3]>()) as wgpu::BufferAddress,
                    shader_location: 3,
                },
            ],
        }];

        let shader = self.resources.device.create_shader_module(wgpu::include_wgsl!("shaders/planets.wgsl"));
        
        self.gtools.set_pipeline_layout(pipeline_layout);
        self.gtools.set_bind_group_layout(bind_group_layout);
        self.gtools.bind_groups = Some(self.create_bind_groups_for_planets(self.gtools.bind_group_layout.as_ref().unwrap()));
        self.gtools.init_pipeline(shader, &vertex_buffers, &[Some(self.screen.surface.get_format().into())]);
    }

    fn create_bind_groups_for_planets(&self, bind_group_layout: &wgpu::BindGroupLayout) -> Vec<wgpu::BindGroup> {
        let mut bind_groups = Vec::new();
        
        for (i, texture_view) in self.planet_textures.iter().enumerate() {
            let bind_group = self.resources.device.create_bind_group(&wgpu::BindGroupDescriptor {
                layout: &bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.gtools.uniform_buf.as_ref().unwrap().as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(
                            self.texture_sampler.as_ref().unwrap()
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(texture_view),
                    },
                ],
                label: Some(&format!("Planet {} bind group", i)),
            });
            bind_groups.push(bind_group);
        }
        
        bind_groups
    }

    pub fn update(&mut self) {
        for planet in &mut self.planets {
            let ball = &mut planet.geom_obj;
            let mut coord = Coord::new_cartesian(ball.center.x, ball.center.y, ball.center.z);
            coord.set_spherical(coord.r, coord.azimuth + planet.angle_speed as f64, coord.elevation);
            ball.center = Point3::new(coord.x, coord.y, coord.z);
        }
        self.init_entities();
    }

    fn init_entities(&mut self) {
        self.gtools.entities = Vec::new();
        for i in 0..self.planets.len() {
            let entity = self.create_entity_for_planet(i);
            self.gtools.push_entity(entity);
        }
    }

    fn transform_mesh_to_vertices_indices(mesh: Mesh, color: wgpu::Color) -> (Vec<Vertex>, Vec<u16>) {
        let center = mesh.vertices.iter().fold([0.0; 3], |acc, v| {
            [acc[0] + v[0] as f32, acc[1] + v[1] as f32, acc[2] + v[2] as f32]
        });
        let vertex_count = mesh.vertices.len() as f32;
        let center = [center[0] / vertex_count, center[1] / vertex_count, center[2] / vertex_count];
    
        let vertices = mesh.vertices.iter().map(|ver| {
            let normal = [
                ver[0] as f32 - center[0],
                ver[1] as f32 - center[1],
                ver[2] as f32 - center[2],
            ];

            let x = normal[0];
            let y = normal[1];
            let z = normal[2];
            
            // u = азимутальный угол (долгота), v = полярный угол (широта)
            let u = 0.5 + (z.atan2(x)) / (2.0 * std::f32::consts::PI);
            let v = 0.5 - (y.asin() / std::f32::consts::PI);

            Vertex { 
                _pos: [ver[0] as f32, ver[1] as f32, ver[2] as f32, 1.0], 
                _col: [color.r as f32, color.g as f32, color.b as f32, color.a as f32],
                _normal: normal,
                _uv: [u, v],
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
        let mut vertices_all = Vec::new();
        let mut indices_all = Vec::new();
        for planet in &self.planets {
            let mesh = planet.geom_obj.get_surface();
            let (vertices, indices) = Self::transform_mesh_to_vertices_indices(mesh, wgpu::Color{r:0.0, g:1.0, b:1.0, a:1.0});
            let offset = vertices_all.len() as u16;
            vertices_all.extend(vertices);
            indices_all.extend(indices.iter().map(|&index| index + offset));
        }
        (vertices_all, indices_all)
    }

    fn get_vertices_indices_edges(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices_all = Vec::new();
        let mut indices_all = Vec::new();
        for planet in &self.planets {
            let mesh = planet.geom_obj.get_edges(0.01);
            let (vertices, indices) = Self::transform_mesh_to_vertices_indices(mesh, wgpu::Color{r:1.0, g:0.0, b:1.0, a:1.0});
            let offset = vertices_all.len() as u16;
            vertices_all.extend(vertices);
            indices_all.extend(indices.iter().map(|&index| index + offset));
        }
        (vertices_all, indices_all)
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

            for (i, entity) in self.entities.iter().enumerate() {
                if let Some(bind_group) = self.bind_groups.as_ref().unwrap().get(i) {
                    rpass.set_bind_group(0, bind_group, &[]);
                    rpass.set_index_buffer(entity.index_buf.slice(..), entity.index_format);
                    rpass.set_vertex_buffer(0, entity.vertex_buf.slice(..));
                    rpass.draw_indexed(0..entity.index_count as u32, 0, 0..1);
                }
            }

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
            bind_groups: None,
            pipeline: None,
            pipeline_layout: None,
            entities: vec![],
            bind_group_layout: None,

            uniform_buf: None,
        }
    }
}