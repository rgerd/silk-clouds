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

pub struct Terrain {
    creation_instant: Instant,
    camera: Camera,
    compute_pipeline: ComputePipeline,
    geometry_compute_pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    indirect_draw_buffer: Buffer,
    terrain_vertex_buffer: Buffer,
    compute_bind_group: BindGroup,
    geometry_compute_bind_group: BindGroup,
    render_bind_group: BindGroup,
}

const VOXELS_PER_CHUNK_DIM: u32 = 64;
const VERTICES_PER_VOXEL: u64 = 3 * 3;

impl Terrain {
    pub fn new(gfx: &Graphics) -> Self {
        let terrain_texture_desc = TextureDescriptor {
            label: Some("terrain_texture"),
            size: Extent3d {
                width: VOXELS_PER_CHUNK_DIM + 1,
                height: VOXELS_PER_CHUNK_DIM + 1,
                depth_or_array_layers: VOXELS_PER_CHUNK_DIM + 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D3,
            format: TextureFormat::R32Float,
            usage: TextureUsages::STORAGE_BINDING,
            view_formats: &[TextureFormat::R32Float],
        };
        let texture = gfx.device().create_texture(&terrain_texture_desc);
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("terrain_texture_view"),
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
                            access: wgpu::StorageTextureAccess::WriteOnly,
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
        let compute_pipeline = gfx
            .device()
            .create_compute_pipeline(&ComputePipelineDescriptor {
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

        // Geometry generation shader
        let geometry_compute_shader = gfx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/terrain_geometry.wgsl"));
        let geometry_compute_bind_group_layout =
            gfx.device()
                .create_bind_group_layout(&BindGroupLayoutDescriptor {
                    label: Some("terrain_geometry_compute_bind_group_layout"),
                    entries: &[
                        // Density data
                        BindGroupLayoutEntry {
                            binding: 0,
                            visibility: ShaderStages::COMPUTE,
                            ty: BindingType::StorageTexture {
                                access: wgpu::StorageTextureAccess::ReadOnly,
                                format: TextureFormat::R32Float,
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
        let geometry_compute_pipeline_layout =
            gfx.device()
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("terrain_geometry_compute_pipeline_layout"),
                    bind_group_layouts: &[&geometry_compute_bind_group_layout],
                    push_constant_ranges: &[],
                });
        let geometry_compute_pipeline =
            gfx.device()
                .create_compute_pipeline(&ComputePipelineDescriptor {
                    label: Some("terrain_geometry_compute_pipeline"),
                    layout: Some(&geometry_compute_pipeline_layout),
                    module: &geometry_compute_shader,
                    entry_point: "main",
                });

        // Render pipeline
        let bind_group_layout = gfx
            .device()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("terrain_bind_group_layout"),
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
                    label: Some("terrain_render_pipeline_layout"),
                    bind_group_layouts: &[&bind_group_layout],
                    push_constant_ranges: &[],
                });
        let terrain_render_shader = gfx
            .device()
            .create_shader_module(wgpu::include_wgsl!("./shaders/terrain.wgsl"));

        const ATTRIBS: [VertexAttribute; 3] =
            vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4];
        const TERRAIN_VERTEX_SIZE: u64 = 3 * 4 * 4;

        let aligned_vertex_desc = VertexBufferLayout {
            array_stride: TERRAIN_VERTEX_SIZE,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &ATTRIBS,
        };

        let render_pipeline =
            gfx.device()
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("terrain_render_pipeline"),
                    layout: Some(&render_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &terrain_render_shader,
                        entry_point: "vs_main",
                        buffers: &[aligned_vertex_desc],
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
                        front_face: wgpu::FrontFace::Ccw,
                        cull_mode: None, //Some(wgpu::Face::Back),
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
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera.buffer_binding_resource(),
            }],
        });

        let terrain_vertex_buffer = gfx.device().create_buffer(&BufferDescriptor {
            label: Some("terrain_vertex_buffer"),
            // Consider using index buffer
            // 48 bytes gives 3 triangles, 36 bytes gives 4 triangles
            size: VOXELS_PER_CHUNK_DIM as u64
                * VOXELS_PER_CHUNK_DIM as u64
                * VOXELS_PER_CHUNK_DIM as u64
                * TERRAIN_VERTEX_SIZE
                * VERTICES_PER_VOXEL,
            usage: BufferUsages::STORAGE | BufferUsages::VERTEX,
            mapped_at_creation: false,
        });

        let indirect_draw_buffer = gfx.device().create_buffer(&BufferDescriptor {
            label: Some("terrain_indirect_draw_buffer"),
            size: std::mem::size_of::<DrawIndirect>() as u64,
            usage: BufferUsages::STORAGE | BufferUsages::INDIRECT | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let edge_table_buffer = gfx.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_edge_table_buffer"),
            contents: bytemuck::cast_slice(&crate::marching_cubes::EDGE_TABLE),
            usage: BufferUsages::STORAGE,
        });

        let tri_table_buffer = gfx.device().create_buffer_init(&BufferInitDescriptor {
            label: Some("terrain_tri_table_buffer"),
            contents: bytemuck::cast_slice(&crate::marching_cubes::TRI_TABLE),
            usage: BufferUsages::STORAGE,
        });

        let geometry_compute_bind_group =
            gfx.device().create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("terrain_geometry_compute_bind_group"),
                layout: &geometry_compute_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: indirect_draw_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: terrain_vertex_buffer.as_entire_binding(),
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
            render_bind_group: main_bind_group,
            compute_bind_group,
            compute_pipeline,
            geometry_compute_pipeline,
            geometry_compute_bind_group,
            terrain_vertex_buffer,
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

        // Clear the indirect draw buffer
        // See wgpu::DrawIndirect
        gfx.queue().write_buffer(
            &self.indirect_draw_buffer,
            0,
            bytemuck::cast_slice(&[0_u32, 1_u32, 0_u32, 0_u32]),
        );

        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("terrain_render_command_encoder"),
            });

        // Generate density data
        // This step operates on the corners of the voxels
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("terrain_compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.compute_pipeline);
            let world_time = self.creation_instant.elapsed().as_secs_f32();
            compute_pass.set_push_constants(0, bytemuck::cast_slice(&[world_time]));
            compute_pass.set_bind_group(0, &self.compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(13, 13, 13);
        }

        // Marching cubes
        // This step operates on the centers of the voxels
        {
            let mut compute_pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("terrain_geometry_compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.geometry_compute_pipeline);
            compute_pass.set_bind_group(0, &self.geometry_compute_bind_group, &[]);
            compute_pass.dispatch_workgroups(16, 16, 8);
        }

        // Render mesh
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
            render_pass.set_vertex_buffer(0, self.terrain_vertex_buffer.slice(..));
            // TODO: set index buffer
            render_pass.draw_indirect(&self.indirect_draw_buffer, 0);
        }

        gfx.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        // let (rx, tx) = flume::bounded::<Result<(), BufferAsyncError>>(1);
        // let draw_buffer_slice = self.terrain_vertex_buffer.slice(..);
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
        // println!();
        // self.terrain_vertex_buffer.unmap();
        Ok(())
    }
}
