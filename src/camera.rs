use std::f64::consts::PI;

use glm::{Mat4, Vec3};
use nalgebra::Point3;
use wgpu::util::DeviceExt as _;

use crate::{graphics::Graphics, texture};

#[rustfmt::skip]
const OPENGL_TO_WGPU_MATRIX: Mat4 = Mat4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

pub struct Camera {
    eye: Point3<f32>,
    target: Point3<f32>,
    up: Vec3,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    buffer: wgpu::Buffer,
    bind_group_layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,

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
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Camera Bind Group Layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera Bind Group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Camera {
            // position the camera 1 unit up and 2 units back
            // +z is out of the screen
            eye: Point3::<f32>::new(0.0, 1.0, 2.0),
            // have it look at the origin
            target: Point3::<f32>::new(0.0, 0.0, 0.0),
            // which way is "up"
            up: Vec3::new(0.0, 1.0, 0.0),
            aspect: 800.0 / 600.0,
            fovy: 45.0,
            znear: 0.1,
            zfar: 100.0,

            buffer,
            bind_group_layout,
            bind_group,

            depth_texture: texture::Texture::create_depth_texture(
                device,
                gfx.config(),
                "Depth texture",
            ),
        }
    }

    pub fn depth_texture(&self) -> &texture::Texture {
        &self.depth_texture
    }

    fn build_view_projection_matrix(&self) -> Mat4 {
        let view = Mat4::look_at_rh(&self.eye, &self.target, &self.up);
        let proj = Mat4::new_perspective(self.aspect, self.fovy, self.znear, self.zfar);
        return proj * view;
    }

    pub fn update(&mut self, queue: &wgpu::Queue) {
        let time = 1.0
            * (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64()
                % (PI * 2.0)) as f32;
        self.eye.x = time.cos() * 8.0;
        self.eye.y = time.sin() * 4.0;
        self.eye.z = time.sin() * 8.0;
        let mat = self.build_view_projection_matrix();
        let data = CameraUniform {
            view_proj: mat.into(),
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[data]));
    }

    pub fn bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
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
