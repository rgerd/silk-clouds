#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub location: [f32; 3],
    pub scale: [f32; 3],
    pub color: [f32; 3],
}

impl Instance {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![3 => Float32x3, 4 => Float32x3, 5 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait Mesh {
    fn vertex_count(&self) -> usize;
    fn vertex_buffer(&self) -> &wgpu::Buffer;

    fn instance_count(&self) -> usize {
        1
    }
    fn instance_buffer(&self) -> Option<&wgpu::Buffer> {
        None
    }

    fn index_count(&self) -> usize {
        0
    }
    fn index_buffer(&self) -> Option<&wgpu::Buffer> {
        None
    }
    fn index_format(&self) -> Option<wgpu::IndexFormat> {
        None
    }
}
