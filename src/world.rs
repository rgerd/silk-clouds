use std::time::Instant;

use winit::event::WindowEvent;

use crate::graphics::Graphics;
use crate::mesh::Instance;
use crate::shapes::Cube;

use crate::texture;
use crate::{
    camera::Camera,
    mesh::{Mesh, Vertex},
};

pub struct World {
    creation_instant: Instant,
    last_elapsed: f32,
    mesh_render_pipeline: wgpu::RenderPipeline,
    meshes: Vec<Box<dyn Mesh>>,
    camera: Camera,
}

impl World {
    pub fn new(gfx: &Graphics) -> Self {
        let device = gfx.device();
        let output_format = gfx.config().format;

        let meshes: Vec<Box<dyn Mesh>> = vec![Box::new(Cube::new(device))];

        let camera = Camera::new(gfx);

        let shader = device.create_shader_module(wgpu::include_wgsl!("shaders/shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Mesh Render Pipeline Layout"),
                bind_group_layouts: &[/*camera.bind_group_layout()*/],
                push_constant_ranges: &[],
            });

        let mesh_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Mesh Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc(), Instance::desc()],
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
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            creation_instant: Instant::now(),
            last_elapsed: 0.0,
            mesh_render_pipeline,
            meshes,
            camera,
        }
    }

    pub fn input(&mut self, window_event: &WindowEvent) {
        // match window_event {
        //     WindowEvent::KeyboardInput { event, .. } => {
        //         println!("KeyboardInput: {:?}", event);
        //     }
        //     WindowEvent::MouseInput { button, .. } => {
        //         println!("MouseInput: {:?}", button);
        //     }
        //     _ => {}
        // }
    }

    pub fn update(&mut self) {
        let world_time = self.creation_instant.elapsed().as_secs_f32();
        self.camera.update(world_time);
    }

    pub fn render(&mut self, gfx: &Graphics) -> Result<(), wgpu::SurfaceError> {
        let output = gfx.surface().get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = gfx
            .device()
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        self.camera.write_data_buffer(gfx.queue());

        {
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
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.camera.depth_texture().view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.mesh_render_pipeline);
            // render_pass.set_bind_group(0, self.camera.bind_group(), &[]);
            for mesh in &self.meshes {
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer().slice(..));

                if let Some(instance_buffer) = mesh.instance_buffer() {
                    render_pass.set_vertex_buffer(1, instance_buffer.slice(..));
                }

                if let Some(index_buffer) = mesh.index_buffer() {
                    render_pass
                        .set_index_buffer(index_buffer.slice(..), mesh.index_format().unwrap());
                    render_pass.draw_indexed(
                        0..(mesh.index_count() as u32),
                        0,
                        0..(mesh.instance_count() as u32),
                    );
                } else {
                    render_pass.draw(
                        0..(mesh.vertex_count() as u32),
                        0..(mesh.instance_count() as u32),
                    );
                }
            }
        }

        gfx.queue().submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}
