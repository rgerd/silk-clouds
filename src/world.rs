use wgpu::{CommandEncoder, Device, TextureFormat};

use crate::shapes::Cube;

use crate::{
    camera::Camera,
    mesh::{Mesh, Vertex},
};

pub struct World {
    mesh_render_pipeline: wgpu::RenderPipeline,
    meshes: Vec<Box<dyn Mesh>>,
    camera: Camera,
}

impl World {
    pub fn new(device: &Device, output_format: TextureFormat) -> Self {
        let meshes: Vec<Box<dyn Mesh>> = vec![Box::new(Cube::new(device))];

        let camera = Camera::new(device);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Mesh Render Pipeline Layout"),
                bind_group_layouts: &[camera.bind_group_layout()],
                push_constant_ranges: &[],
            });

        let mesh_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: output_format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),

            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            mesh_render_pipeline,
            meshes,
            camera,
        }
    }

    pub fn render(
        &mut self,
        encoder: &mut CommandEncoder,
        queue: &mut wgpu::Queue,
        view: &wgpu::TextureView,
    ) {
        self.camera.update(queue);
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    }),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        render_pass.set_pipeline(&self.mesh_render_pipeline);
        render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
        for mesh in &self.meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));

            if let Some(index_buffer) = mesh.index_buffer() {
                render_pass.set_index_buffer(index_buffer.slice(..), mesh.index_format().unwrap());
                render_pass.draw_indexed(0..(mesh.index_count() as u32), 0, 0..1);
            } else {
                render_pass.draw(0..(mesh.vertex_count() as u32), 0..3);
            }
        }
    }
}
