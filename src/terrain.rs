use std::{f64::consts::PI, time::Instant};

use wgpu::{
    BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Color,
    ComputePipeline, DepthStencilState, Extent3d, LoadOp, Operations, PushConstantRange,
    RenderPassColorAttachment, RenderPassDescriptor, RenderPipeline, SamplerBindingType,
    ShaderStages, StoreOp, SurfaceError, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureSampleType, TextureUsages, TextureViewDimension,
};

use crate::{
    camera::Camera,
    graphics::Graphics,
    mesh::{Mesh, Vertex},
    shapes::Cube,
    world,
};

pub struct Terrain {
    creation_instant: Instant,
    camera: Camera,
    texture: Texture,
    terrain_data: Vec<f32>,
    cube: Cube,
    render_bind_group: BindGroup,
    compute_bind_group: BindGroup,
    compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
}

impl Terrain {
    pub fn new(gfx: &Graphics) -> Self {
        let terrain_texture_desc = TextureDescriptor {
            label: Some("terrain_texture"),
            size: Extent3d {
                width: 65,
                height: 65,
                depth_or_array_layers: 65,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::R32Float,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::STORAGE_BINDING,
            view_formats: &[TextureFormat::R32Float],
        };
        let texture = gfx.device().create_texture(&terrain_texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("terrain_texture_view"),
            ..Default::default()
        });
        let sampler = gfx.device().create_sampler(&wgpu::SamplerDescriptor {
            label: Some("terrain_sampler"),
            ..Default::default()
        });

        // Compute pipeline
        let compute_shader = gfx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/terrain_compute.wgsl"));
        let compute_bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("terrain_compute_bind_group_layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::ReadWrite,
                            format: TextureFormat::R32Float,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    }],
                });
        let compute_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("terrain_compute_pipeline_layout"),
                    bind_group_layouts: &[&compute_bind_group_layout],
                    push_constant_ranges: &[PushConstantRange {
                        stages: ShaderStages::COMPUTE,
                        range: 0..4,
                    }],
                });
        let compute_pipeline =
            gfx.device()
                .create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
                    label: Some("terrain_compute_pipeline"),
                    layout: Some(&compute_pipeline_layout),
                    module: &compute_shader,
                    entry_point: "main",
                });
        let compute_bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain_compute_bind_group"),
            layout: &compute_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&texture_view),
            }],
        });

        // Render pipeline
        let bind_group_layout = gfx
            .device()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("terrain_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Texture {
                            multisampled: false,
                            view_dimension: TextureViewDimension::D3,
                            sample_type: TextureSampleType::Float { filterable: false },
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 2,
                        visibility: ShaderStages::VERTEX_FRAGMENT,
                        ty: BindingType::Sampler(SamplerBindingType::NonFiltering),
                        count: None,
                    },
                ],
            });
        let render_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("terrain_render_pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let terrain_render_shader = gfx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/terrain.wgsl"));

        let render_pipeline =
            gfx.device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("terrain_render_pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &terrain_render_shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &terrain_render_shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gfx.config().format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: crate::texture::Texture::DEPTH_FORMAT,
                        depth_write_enabled: true,
                        depth_compare: wgpu::CompareFunction::Less,
                        stencil: Default::default(),
                        bias: Default::default(),
                    }),
                    multisample: wgpu::MultisampleState::default(),
                    multiview: None,
                });

        let camera = Camera::new(gfx);
        let main_bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain_bind_group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera.buffer_binding_resource(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });

        Self {
            creation_instant: Instant::now(),
            camera,
            terrain_data: vec![0.0_f32; 65 * 65 * 65],
            cube: Cube::new(gfx.device()),
            texture,
            render_bind_group: main_bind_group,
            render_pipeline,
            compute_bind_group,
            compute_pipeline,
        }
    }

    pub fn update(&mut self) {
        let world_time = self.creation_instant.elapsed().as_secs_f32();
        self.camera.update(world_time);

        for i in 0..65 {
            for j in 0..65 {
                for k in 0..65 {
                    let array_idx = i * 65 * 65 + j * 65 + k;
                    let _wt = (world_time as f64) * 3.0;
                    let _i = ((((i as f64 / 65.0) - 0.5) * PI * 2.0 + _wt).sin() * 0.5 + 0.5) * 0.3;
                    let _j = (((j as f64 / 65.0) - 0.5) * PI * 1.0 + _wt).cos() * 0.5 + 0.5;
                    let _k = ((((k as f64 / 65.0) - 0.5) * PI * 9.0 + _wt).sin() * 0.5 + 0.5) * 0.1;
                    // self.terrain_data[array_idx] = ((PI * 2.0 * domain_idx as f64 / 65.0
                    //     + world_time as f64 * 3.0)
                    //     .sin() as f32
                    //     + 1.0)
                    //     * 0.5;
                    self.terrain_data[array_idx] = ((_i + _j + _k) as f32).powf(1.0 / 3.0);
                }
            }
        }
    }

    pub fn render(&self, gfx: &Graphics) -> anyhow::Result<(), SurfaceError> {
        let output = gfx.surface().get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.camera.write_data_buffer(gfx.queue());
        // gfx.queue().write_texture(
        //     self.texture.as_image_copy(),
        //     bytemuck::cast_slice(&self.terrain_data),
        //     ImageDataLayout {
        //         offset: 0,
        //         bytes_per_row: Some(65 * 4),
        //         rows_per_image: Some(65),
        //     },
        //     Extent3d {
        //         width: 65,
        //         height: 65,
        //         depth_or_array_layers: 65,
        //     },
        // );

        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("terrain_render_command_encoder"),
            });
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("terrain_compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            let world_time = self.creation_instant.elapsed().as_secs_f32();
            compute_pass.set_push_constants(0, bytemuck::cast_slice(&[world_time]));
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 65, 65);
        }

        {
            let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("terrain_render_pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &output_view,
                    resolve_target: None,
                    ops: Operations {
                        load: LoadOp::Clear(Color::BLACK),
                        store: StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.camera.depth_texture().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.render_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.cube.vertex_buffer().slice(..));
            render_pass.set_index_buffer(
                self.cube.index_buffer().unwrap().slice(..),
                self.cube.index_format().unwrap(),
            );
            render_pass.draw_indexed(0..(self.cube.index_count() as u32), 0, 0..(65 * 65 * 65));
        }

        gfx.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
