use crate::mesh::{Mesh, Vertex};
use wgpu::util::DeviceExt as _;
use wgpu::Device;

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        color: [0.0, 0.0, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        color: [0.0, 0.5, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        color: [0.5, 0.0, 0.0],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        color: [0.5, 0.0, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        color: [0.0, 0.5, 0.5],
        normal: [0.0, 0.0, 1.0],
    },
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

pub struct Pentagon {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
}

impl Pentagon {
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

        Self {
            vertex_buffer,
            index_buffer,
        }
    }
}

impl Mesh for Pentagon {
    fn vertex_count(&self) -> usize {
        VERTICES.len()
    }

    fn vertex_buffer(&self) -> &wgpu::Buffer {
        &self.vertex_buffer
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
