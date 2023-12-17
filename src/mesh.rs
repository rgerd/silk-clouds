#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub trait Mesh {
    fn vertex_count(&self) -> usize;
    fn vertex_buffer(&self) -> &wgpu::Buffer;

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
