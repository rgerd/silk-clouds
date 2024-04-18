use std::time::Instant;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, DrawIndirect},
    vertex_attr_array, BindGroup, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType,
    Buffer, BufferDescriptor, BufferUsages, Color, ComputePipeline, ComputePipelineDescriptor,
    DepthStencilState, Extent3d, LoadOp, Operations, PushConstantRange, RenderPassColorAttachment,
    RenderPassDescriptor, RenderPipeline, ShaderStages, StoreOp, SurfaceError, TextureDescriptor,
    TextureDimension, TextureFormat, TextureUsages, TextureViewDimension, VertexAttribute,
    VertexBufferLayout,
};

use crate::{camera::Camera, graphics::Graphics};

pub struct CloudWorld {
    creation_instant: Instant,
    camera: Camera,
    density_pipeline: ComputePipeline,
    marching_cubes_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    indirect_draw_buffer: Buffer,
    cloud_vertex_buffer: Buffer,
    density_bind_group: BindGroup,
    marching_cubes_bind_group: BindGroup,
    main_bind_group: BindGroup,
}

const VOXELS_PER_CHUNK_DIM: u32 = 50;
const VERTICES_PER_VOXEL: u64 = 1 * 3; // Assumes an average of 1 triangle per voxel

impl CloudWorld {
    pub fn new(gfx: &Graphics) -> Self {
        let density_texture_desc = TextureDescriptor {
            label: Some("density_texture"),
            size: Extent3d {
                width: VOXELS_PER_CHUNK_DIM + 1,
                height: VOXELS_PER_CHUNK_DIM + 1,
                depth_or_array_layers: VOXELS_PER_CHUNK_DIM + 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::Rgba16Float,
            usage: TextureUsages::STORAGE_BINDING,
            view_formats: &[TextureFormat::Rgba16Float],
        };
        let density_texture = gfx.device().create_texture(&density_texture_desc);
        let density_texture_view = density_texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("density_texture_view"),
            ..Default::default()
        });

        // Density generation shader
        let density_shader = gfx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/cloud_density.wgsl"));
        let density_bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("density_bind_group_layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::COMPUTE,
                        ty: BindingType::StorageTexture {
                            access: wgpu::StorageTextureAccess::WriteOnly,
                            format: TextureFormat::Rgba16Float,
                            view_dimension: TextureViewDimension::D3,
                        },
                        count: None,
                    }],
                });
        let density_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("density_pipeline_layout"),
                    bind_group_layouts: &[&density_bind_group_layout],
                    push_constant_ranges: &[PushConstantRange {
                        stages: ShaderStages::COMPUTE,
                        range: 0..8,
                    }],
                });
        let density_pipeline = gfx
            .device()
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("density_pipeline"),
                layout: Some(&density_pipeline_layout),
                module: &density_shader,
                entry_point: "main",
            });
        let density_bind_group = gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("density_bind_group"),
            layout: &density_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&density_texture_view),
            }],
        });

        // Marching cubes shader
        let marching_cubes_shader = gfx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/marching_cubes.wgsl"));
        let marching_cubes_bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("marching_cubes_bind_group_layout"),
                    entries: &[
                        // Density data
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadOnly,
                                format: TextureFormat::Rgba16Float,
                                view_dimension: TextureViewDimension::D3,
                            },
                            count: None,
                        },
                        // Indirect draw buffer
                        BindGroupLayoutEntry {
                            binding: 1,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Vertex buffer
                        BindGroupLayoutEntry {
                            binding: 2,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: false },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Edge table
                        BindGroupLayoutEntry {
                            binding: 3,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                        // Tri table
                        BindGroupLayoutEntry {
                            binding: 4,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::Buffer {
                                ty: wgpu::BufferBindingType::Storage { read_only: true },
                                has_dynamic_offset: false,
                                min_binding_size: None,
                            },
                            count: None,
                        },
                    ],
                });
        let marching_cubes_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("marching_cubes_pipeline_layout"),
                    bind_group_layouts: &[&marching_cubes_bind_group_layout],
                    push_constant_ranges: &[PushConstantRange {
                        stages: ShaderStages::COMPUTE,
                        range: 0..8,
                    }],
                });
        let marching_cubes_pipeline =
            gfx.device()
                .create_compute_pipeline(&ComputePipelineDescriptor {
                    label: Some("marching_cubes_pipeline"),
                    layout: Some(&marching_cubes_pipeline_layout),
                    module: &marching_cubes_shader,
                    entry_point: "main",
                });

        // Render pipeline
        let render_bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("render_bind_group_layout"),
                    entries: &[BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                });
        let render_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("render_pipeline_layout"),
                    bind_group_layouts: &[&render_bind_group_layout],
                    push_constant_ranges: &[PushConstantRange {
                        stages: ShaderStages::VERTEX_FRAGMENT,
                        range: 0..8,
                    }],
                });
        let chunk_render_shader = gfx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/chunk_render.wgsl"));

        const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x4, 1 => Float32x4];
        const CLOUD_VERTEX_SIZE: u64 = 2 * 4 * 4;

        let aligned_vertex_desc = VertexBufferLayout {
            array_stride: CLOUD_VERTEX_SIZE,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        };

        let render_pipeline =
            gfx.device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("chunk_render_pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &chunk_render_shader,
                        entry_point: "vs_main",
                        buffers: &[aligned_vertex_desc],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &chunk_render_shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gfx.config().format,
                            blend: Some(wgpu::BlendState::REPLACE),
                            write_mask: wgpu::ColorWrites::ALL,
                        })],
                    }),
                    primitive: wgpu::PrimitiveState {
                        topology: wgpu::PrimitiveTopology::TriangleList,
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None, //Some(wgpu::Face::Back),
                        ..Default::default()
                    },
                    depth_stencil: Some(DepthStencilState {
                        format: wgpu::TextureFormat::Depth32Float,
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
            label: Some("world_bind_group"),
            layout: &render_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera.buffer_binding_resource(),
            }],
        });

        let cloud_vertex_buffer = gfx.device().create_buffer(&BufferDescriptor {
            label: Some("cloud_vertex_buffer"),
            size: VOXELS_PER_CHUNK_DIM as u64
                * VOXELS_PER_CHUNK_DIM as u64
                * VOXELS_PER_CHUNK_DIM as u64
                * CLOUD_VERTEX_SIZE
                * VERTICES_PER_VOXEL,
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let indirect_draw_buffer = gfx.device().create_buffer(&BufferDescriptor {
            label: Some("render_indirect_draw_buffer"),
            size: std::mem::size_of::<DrawIndirect>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let edge_table_buffer = gfx.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("marching_cubes_edge_table_buffer"),
            contents: bytemuck::cast_slice(&crate::marching_cubes::EDGE_TABLE),
            usage: BufferUsages::STORAGE,
        });

        let tri_table_buffer = gfx.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("marching_cubes_tri_table_buffer"),
            contents: bytemuck::cast_slice(&crate::marching_cubes::TRI_TABLE),
            usage: BufferUsages::STORAGE,
        });

        let marching_cubes_bind_group =
            gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("marching_cubes_compute_bind_group"),
                layout: &marching_cubes_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&density_texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: indirect_draw_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: cloud_vertex_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: edge_table_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: tri_table_buffer.as_entire_binding(),
                    },
                ],
            });

        Self {
            creation_instant: Instant::now(),
            camera,
            main_bind_group,
            density_bind_group,
            density_pipeline,
            marching_cubes_pipeline,
            marching_cubes_bind_group,
            cloud_vertex_buffer,
            indirect_draw_buffer,
            render_pipeline,
        }
    }

    pub fn update(&mut self) {
        let world_time = self.creation_instant.elapsed().as_secs_f32();
        self.camera.update(world_time);
    }

    pub fn render(&self, gfx: &Graphics) -> anyhow::Result<(), SurfaceError> {
        let output = gfx.surface().get_current_texture()?;
        let output_view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.camera.write_data_buffer(gfx.queue());
        let world_time = self.creation_instant.elapsed().as_secs_f32();

        // Render a 2x2x2 grid of chunks in 8 render passes.
        // Each chunk saturates the GPU with work.
        // The workgroup counts are conditioned on the workgroup sizes
        // to cover every voxel in the chunk without going over GPU limits.
        for chunk_id in 0..8 {
            let push_constants_slice = &[world_time, bytemuck::cast::<u32, f32>(chunk_id)];
            let push_constants = bytemuck::cast_slice(push_constants_slice);

            // Clear the indirect draw buffer
            // See wgpu::DrawIndirect
            gfx.queue().write_buffer(
                &self.indirect_draw_buffer,
                0,
                bytemuck::cast_slice(&[0_u32, 1_u32, 0_u32, 0_u32]),
            );

            let mut encoder =
                gfx.device()
                    .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                        label: Some("cloud_render_command_encoder"),
                    });

            // Generate density data
            // This step operates on the corners of the voxels
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("cloud_density_pass"),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.density_pipeline);
                compute_pass.set_push_constants(0, push_constants);
                compute_pass.set_bind_group(0, &self.density_bind_group, &[]);
                compute_pass.dispatch_workgroups(7, 7, 7);
            }

            // Marching cubes
            // This step operates on the centers of the voxels
            {
                let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                    label: Some("marching_cubes_compute_pass"),
                    timestamp_writes: None,
                });
                compute_pass.set_pipeline(&self.marching_cubes_pipeline);
                compute_pass.set_push_constants(0, push_constants);
                compute_pass.set_bind_group(0, &self.marching_cubes_bind_group, &[]);
                compute_pass.dispatch_workgroups(5, 5, 5);
            }

            // Render mesh
            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("cloud_render_pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &output_view,
                        resolve_target: None,
                        ops: Operations {
                            load: if chunk_id == 0 {
                                LoadOp::Clear(Color::BLACK)
                            } else {
                                LoadOp::Load
                            },
                            store: StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.camera.depth_texture().view,
                        depth_ops: Some(wgpu::Operations {
                            load: if chunk_id == 0 {
                                wgpu::LoadOp::Clear(1.0)
                            } else {
                                wgpu::LoadOp::Load
                            },
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_push_constants(ShaderStages::VERTEX_FRAGMENT, 0, push_constants);
                render_pass.set_bind_group(0, &self.main_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.cloud_vertex_buffer.slice(..));
                render_pass.draw_indirect(&self.indirect_draw_buffer, 0);
            }
            gfx.queue().submit(std::iter::once(encoder.finish()));
        }
        output.present();

        // Uncomment below to read back the vertex buffer.
        //
        // let (rx, tx) = flume::bounded::<Result<(), BufferAsyncError>>(1);
        // let draw_buffer_slice = self.cloud_vertex_buffer.slice(..);
        // draw_buffer_slice.map_async(wgpu::MapMode::Read, move |result| rx.send(result).unwrap());
        // gfx.device().poll(wgpu::Maintain::Wait);
        // pollster::block_on(tx.recv_async()).unwrap().unwrap();
        // {
        //     let draw_buffer_view = draw_buffer_slice.get_mapped_range();
        //     draw_buffer_view
        //         .chunks_exact(std::mem::size_of::<Vertex>())
        //         .map(|chunk| {
        //             let vertex = bytemuck::from_bytes::<Vertex>(chunk);
        //             println!("{:?}", vertex);
        //         })
        //         .count();
        // }
        // self.cloud_vertex_buffer.unmap();

        Ok(())
    }
}
