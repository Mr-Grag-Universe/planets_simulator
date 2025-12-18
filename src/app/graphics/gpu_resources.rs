use wgpu::{self, ColorWrites, PipelineLayout, ShaderModule};

pub struct GPU_Resources {
    pub instance: wgpu::Instance,
    pub adapter: wgpu::Adapter,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
}

impl GPU_Resources {
    pub async fn new() -> GPU_Resources {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor::default());
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        GPU_Resources {
            instance,
            adapter,
            device,
            queue,
        }
    }

    pub fn create_pipeline_layout(&self) -> wgpu::PipelineLayout {
        let pipeline_layout = self.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        pipeline_layout
    }

    pub fn create_render_pipeline(&self, pipeline_layout: &PipelineLayout, shader: &ShaderModule, swapchain_format: wgpu::TextureFormat) -> wgpu::RenderPipeline {
        let render_pipeline = self.device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: swapchain_format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all()
                })],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });
        render_pipeline
    }

    pub fn create_command_encoder(&self, desc: &wgpu::CommandEncoderDescriptor<'_>) -> wgpu::CommandEncoder {
        self.device.create_command_encoder(desc)
    }

    pub fn submit_to_queue<I>(&self, command_buffers: I) -> wgpu::SubmissionIndex
    where
        I: IntoIterator<Item = wgpu::CommandBuffer>
    {
        self.queue.submit(command_buffers)
    }
}

impl Default for GPU_Resources {
    fn default() -> GPU_Resources {
        pollster::block_on(GPU_Resources::new())
    }
}