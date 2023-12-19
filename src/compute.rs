use std::ops::Range;

use wgpu::{
    include_wgsl, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, Buffer, BufferDescriptor, BufferUsages, CommandEncoderDescriptor,
    ComputePassDescriptor, ComputePipeline, ComputePipelineDescriptor, PipelineLayoutDescriptor,
    PushConstantRange, ShaderStages,
};

use crate::graphics::Graphics;

pub struct Compute {
    out_buffer: Buffer,
    staging_buffer: Buffer,
    bind_group: BindGroup,
    pipeline: ComputePipeline,
}

impl Compute {
    pub fn new(gfx: &Graphics) -> Self {
        let bind_group_layout = gfx
            .device()
            .create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Compute bind group layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let out_buffer = gfx.device().create_buffer(&BufferDescriptor {
            label: Some("compute_out_buffer"),
            size: 12 * 4,
            usage: BufferUsages::COPY_SRC | BufferUsages::STORAGE,
            mapped_at_creation: false,
        });

        let staging_buffer = gfx.device().create_buffer(&BufferDescriptor {
            label: Some("compute_staging_buffer"),
            size: 12 * 4,
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        let bind_group = gfx.device().create_bind_group(&BindGroupDescriptor {
            label: Some("Compute bind group"),
            layout: &bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: out_buffer.as_entire_binding(),
            }],
        });

        let shader = gfx
            .device()
            .create_shader_module(include_wgsl!("shaders/compute.wgsl"));
        let pipeline_layout = gfx
            .device()
            .create_pipeline_layout(&PipelineLayoutDescriptor {
                label: Some("Compute Pipeline Layout"),
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[PushConstantRange {
                    stages: ShaderStages::COMPUTE,
                    range: Range { start: 0, end: 32 },
                }],
            });
        let pipeline = gfx
            .device()
            .create_compute_pipeline(&ComputePipelineDescriptor {
                label: Some("Compute Pipeline"),
                layout: Some(&pipeline_layout),
                module: &shader,
                entry_point: "main",
            });

        Self {
            pipeline,
            out_buffer,
            staging_buffer,
            bind_group,
        }
    }

    pub fn run(&mut self, gfx: &Graphics) -> anyhow::Result<()> {
        let mut encoder = gfx
            .device()
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("compute_encoder"),
            });
        {
            let mut compute_pass = encoder.begin_compute_pass(&ComputePassDescriptor {
                label: Some("compute_pass"),
                timestamp_writes: None,
            });
            compute_pass.set_pipeline(&self.pipeline);
            compute_pass.set_push_constants(0, &[1, 2, 3, 4, 5, 6, 7, 8]);
            compute_pass.set_bind_group(0, &self.bind_group, &[]);
            compute_pass.dispatch_workgroups(1, 1, 1);
        }
        encoder.copy_buffer_to_buffer(&self.out_buffer, 0, &self.staging_buffer, 0, 4 * 12);
        gfx.queue().submit(std::iter::once(encoder.finish()));
        pollster::block_on(self.print_output(gfx));
        Ok(())
    }

    async fn print_output(&self, gfx: &Graphics) {
        let (tx, rx) = flume::bounded(1);
        let staging_slice = self.staging_buffer.slice(..);

        staging_slice.map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
        gfx.device().poll(wgpu::Maintain::Wait);

        rx.recv_async().await.unwrap().unwrap();
        {
            let mapped_range = staging_slice.get_mapped_range();
            let mut local_buffer = [2_u32; 12];
            local_buffer.copy_from_slice(bytemuck::cast_slice(&mapped_range));
            dbg!(local_buffer);
        }
        self.staging_buffer.unmap();
    }
}
