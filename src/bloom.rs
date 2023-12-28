//1. 将main pass的output texture先过一个render pass得到高亮部分
//2. ping pong法高斯模糊
//3. 将高斯模糊后结果送入后处理pass合并

use wgpu::util::DeviceExt;

use crate::texture;

/// Owns the render texture and controls tonemapping
#[allow(dead_code)]
pub struct BloomPipeline {
    pipeline: wgpu::ComputePipeline,
    bind_group: wgpu::BindGroup,
    workgroup_count: (u32, u32),
    pub texture1: texture::Texture,
    texture2: texture::Texture,
}

impl BloomPipeline {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, input: &wgpu::TextureView) -> Self {
        let width = config.width;
        let height = config.height;

        let format = wgpu::TextureFormat::Rgba16Float;

        let texture1 = texture::Texture::create_2d_texture(
            device,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            wgpu::FilterMode::Nearest,
            Some("Ping"),
        );

        let texture2 = texture::Texture::create_2d_texture(
            device,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            wgpu::FilterMode::Nearest,
            Some("Pong"),
        );

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Hdr::layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(0),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format,
                    },
                    count: None,
                },
            ],
        });

        let img_size = [width as i32, height as i32];
        // 计算工作组大小
        let workgroup_count = ((width + 15) / 16, (height + 15) / 16);
        let uniform_buf = device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[img_size, [1, 0]]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Hdr::bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(input),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture1.view),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("bloom_pre.wgsl"));
        let pipeline_layout = device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device
            .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "cs_main",
                label: None,
            });

        Self{
            pipeline,
            bind_group,
            workgroup_count,
            texture1,
            texture2
        }
    }
    pub fn process(&self, encoder: &mut wgpu::CommandEncoder) {
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &self.bind_group, &[]);
            cpass.dispatch_workgroups(self.workgroup_count.0, self.workgroup_count.1, 1);
        }
    }
}