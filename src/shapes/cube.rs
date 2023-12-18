use std::f32::consts::PI;

use crate::mesh::{Instance, Mesh, Vertex};
use wgpu::util::DeviceExt as _;
use wgpu::Device;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.5, -0.5, -0.5],
        color: [0.2, 0.2, 0.2],
    },
    Vertex {
        position: [-0.5, -0.5, 0.5],
        color: [0.2, 0.2, 0.2],
    },
    Vertex {
        position: [-0.5, 0.5, -0.5],
        color: [0.2, 0.2, 0.2],
    },
    Vertex {
        position: [-0.5, 0.5, 0.5],
        color: [0.2, 0.2, 0.2],
    },
    Vertex {
        position: [0.5, -0.5, -0.5],
        color: [0.2, 0.2, 0.2],
    },
    Vertex {
        position: [0.5, -0.5, 0.5],
        color: [0.2, 0.2, 0.2],
    },
    Vertex {
        position: [0.5, 0.5, -0.5],
        color: [0.2, 0.2, 0.2],
    },
    Vertex {
        position: [0.5, 0.5, 0.5],
        color: [0.2, 0.2, 0.2],
    },
];

const INDICES: &[u16] = &[
    0, 1, 2, 2, 1, 3, // Left
    4, 6, 5, 5, 6, 7, // Right
    0, 4, 1, 1, 4, 5, // Bottom
    2, 3, 6, 6, 3, 7, // Top
    1, 5, 3, 3, 5, 7, // Front
    0, 2, 4, 4, 2, 6, // Back
];

const INSTANCE_BOX_SIDE_LEN: usize = 64;

pub struct Cube {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
}

impl Cube {
    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        let mut instances = Vec::<Instance>::new();
        for i in 0..INSTANCE_BOX_SIDE_LEN {
            for j in 0..INSTANCE_BOX_SIDE_LEN {
                for k in 0..INSTANCE_BOX_SIDE_LEN {
                    instances.push(Instance {
                        location: [
                            (i as i32 - INSTANCE_BOX_SIDE_LEN as i32 / 2) as f32 * 0.1,
                            (j as i32 - INSTANCE_BOX_SIDE_LEN as i32 / 2) as f32 * 0.1,
                            (k as i32 - INSTANCE_BOX_SIDE_LEN as i32 / 2) as f32 * 0.1,
                        ],
                        scale: [
                            (i as f32 / INSTANCE_BOX_SIDE_LEN as f32 * PI * 4.0).cos() * 0.05
                                + 0.05,
                            (j as f32 / INSTANCE_BOX_SIDE_LEN as f32 * PI * 4.0).cos() * 0.05
                                + 0.05,
                            (k as f32 / INSTANCE_BOX_SIDE_LEN as f32 * PI * 4.0).cos() * 0.05
                                + 0.05,
                        ],
                        color: [
                            i as f32 / INSTANCE_BOX_SIDE_LEN as f32,
                            j as f32 / INSTANCE_BOX_SIDE_LEN as f32,
                            0.2,
                        ],
                    });
                }
            }
        }

        let instance_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Instance Buffer"),
            contents: bytemuck::cast_slice(instances.as_slice()),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            instance_buffer,
        }
    }
}

impl Mesh for Cube {
    fn vertex_count(&self) -> usize {
        VERTICES.len()
    }

    fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    fn instance_count(&self) -> usize {
        INSTANCE_BOX_SIDE_LEN * INSTANCE_BOX_SIDE_LEN * INSTANCE_BOX_SIDE_LEN
    }

    fn instance_buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.instance_buffer)
    }

    fn index_count(&self) -> usize {
        INDICES.len()
    }

    fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.index_buffer)
    }

    fn index_format(&self) -> Option<wgpu::IndexFormat> {
        Some(wgpu::IndexFormat::Uint16)
    }
}
