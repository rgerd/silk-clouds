use std::f64::consts::PI;

use glm::{Mat4, Vec3};
use nalgebra::Point3;
use wgpu::{util::DeviceExt as _, BindingResource, Extent3d};

use crate::{graphics::Graphics, texture};

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    buffer: wgpu::Buffer,

    depth_texture: texture::Texture,
}

impl Camera {
    pub fn new(gfx: &Graphics) -> Self {
        let device = gfx.device();

        let uniform = CameraUniform::new();
        let buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: Point3::<f32>::new(0.0, 1.0, 2.0),
            // have it look at the origin
            target: Point3::<f32>::new(0.0, 0.0, 0.0),
            // which way is "up"
            up: Vec3::new(0.0, 1.0, 0.0),
            aspect: 1.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,

            buffer,

            depth_texture: texture::Texture::create_depth_texture(
                device,
                Extent3d {
                    width: gfx.size().width,
                    height: gfx.size().height,
                    depth_or_array_layers: 1,
                },
                "Depth texture",
            ),
        }
    }

    pub fn buffer_binding_resource(&self) -> BindingResource {
        self.buffer.as_entire_binding()
    }

    pub fn depth_texture(&self) -> &texture::Texture {
        &self.depth_texture
    }

    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = Mat4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
        return proj * view;
    }

    pub fn update(&mut self, world_time: f32) {
        let time = (world_time * 0.15) % (PI * 2.0) as f32;
        self.eye.x = time.cos() * 14.0;
        self.eye.y = time.sin() * 8.0;
        self.eye.z = time.sin() * 14.0;
    }

    pub fn write_data_buffer(&self, queue: &wgpu::Queue) {
        let view_proj = self.build_view_projection_matrix();
        let data = CameraUniform {
            view_proj: view_proj.into(),
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: Mat4::identity().into(),
        }
    }
}
