use crate::mesh::{Mesh, Vertex};
use wgpu::util::DeviceExt as _;
use wgpu::Device;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [0.0, 0.5, 0.0],
        color: [1.0, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.5, -0.5, 0.0],
        color: [0.0, 1.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.5, -0.5, 0.0],
        color: [0.0, 0.0, 1.0],
        normal: [0.0, 0.0, 1.0],
    },
];

pub struct Triangle {
    vertex_buffer: wgpu::Buffer,
}

impl Triangle {
    pub fn new(device: &Device) -> Self {
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });
        Self { vertex_buffer }
    }
}

impl Mesh for Triangle {
    fn vertex_count(&self) -> usize {
        VERTICES.len()
    }

    fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
    }

    fn index_count(&self) -> usize {
        0
    }

    fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        None
    }
}
