//1. 将main pass的output texture先过一个render pass得到高亮部分
//2. ping pong法高斯模糊
//3. 将高斯模糊后结果送入后处理pass合并

use wgpu::util::DeviceExt;

use crate::{create_render_pipeline, texture};

/// Owns the render texture and controls tonemapping
#[allow(dead_code)]
pub struct BloomPipeline {
    pipeline_pre: wgpu::RenderPipeline,
    bind_group_layout_pre: wgpu::BindGroupLayout,
    bind_group_pre: wgpu::BindGroup,
    pipeline: wgpu::ComputePipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group1: wgpu::BindGroup,
    bind_group2: wgpu::BindGroup,
    workgroup_count: (u32, u32),
    pub texture1: texture::Texture,
    texture2: texture::Texture,
    format: wgpu::TextureFormat,
    uniform_buf: wgpu::Buffer,
}

impl BloomPipeline {
    pub fn new(device: &wgpu::Device, config: &wgpu::SurfaceConfiguration, input: &texture::Texture) -> Self {
        let width = config.width;
        let height = config.height;

        let format = wgpu::TextureFormat::Rgba16Float;

        let texture1 = texture::Texture::create_2d_texture(
            device,
            width,
            height,
            format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
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

        let bind_group_layout_pre = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom::layout"),
            entries: &[
                // This is the HDR texture
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bloom::layout"),
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

        let bind_group_pre = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom::bind_group"),
            layout: &bind_group_layout_pre,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&input.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&input.sampler),
                },
            ],
        });

        let bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom::bind_group1"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture2.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture1.view),
                },
            ],
        });

        let bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom::bind_group2"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture1.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture2.view),
                },
            ],
        });

        let shader1 = wgpu::include_wgsl!("bloom_pre.wgsl");
        let shader2 = device.create_shader_module(wgpu::include_wgsl!("blur.wgsl"));

        let pipeline_layout_pre = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout_pre],
            push_constant_ranges: &[],
        });
        let pipeline_pre = create_render_pipeline(
            device,
            &pipeline_layout_pre,
            format,
            None,
            // We'll use some math to generate the vertex data in
            // the shader, so we don't need any vertex buffers
            &[],
            wgpu::PrimitiveTopology::TriangleList,
            shader1,
            "bloom pre pipeline"
        );

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                layout: Some(&pipeline_layout),
                module: &shader2,
                entry_point: "cs_main",
                label: None,
            });

        Self{
            pipeline_pre,
            bind_group_layout_pre,
            bind_group_pre,
            bind_group_layout,
            pipeline,
            bind_group1,
            bind_group2,
            workgroup_count,
            texture1,
            texture2,
            format,
            uniform_buf
        }
    }

    pub fn resize(&mut self, device: &wgpu::Device, texture:&texture::Texture, width: u32, height: u32) {
        self.texture1 = texture::Texture::create_2d_texture(
            device,
            width,
            height,
            self.format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            wgpu::FilterMode::Nearest,
            Some("Ping"),
        );

        self.texture2 = texture::Texture::create_2d_texture(
            device,
            width,
            height,
            self.format,
            wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::STORAGE_BINDING,
            wgpu::FilterMode::Nearest,
            Some("Pong"),
        );
        self.bind_group_pre = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom::bind_group"),
            layout: &self.bind_group_layout_pre,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture.view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&texture.sampler),
                },
            ],
        });
        self.bind_group1 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom::bind_group1"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.texture2.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.texture1.view),
                },
            ],
        });

        self.bind_group2 = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bloom::bind_group2"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.texture1.view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.texture2.view),
                },
            ],
        });
    }

    pub fn process(&self, encoder: &mut wgpu::CommandEncoder) {
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Bloom::process"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.texture1.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });
            pass.set_pipeline(&self.pipeline_pre);
            pass.set_bind_group(0, &self.bind_group_pre, &[]);
            pass.draw(0..3, 0..1);
        }
        
        for _ in 0..0 {
            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                cpass.set_pipeline(&self.pipeline);
                cpass.set_bind_group(0, &self.bind_group1, &[]);
                cpass.dispatch_workgroups(self.workgroup_count.0, self.workgroup_count.1, 1);
            }

            {
                let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
                cpass.set_pipeline(&self.pipeline);
                cpass.set_bind_group(0, &self.bind_group2, &[]);
                cpass.dispatch_workgroups(self.workgroup_count.0, self.workgroup_count.1, 1);
            }
        }
        
    }
}