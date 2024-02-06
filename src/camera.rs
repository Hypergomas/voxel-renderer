use std::{
    f32::consts::PI,
    sync::{Arc, OnceLock},
};

use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;

use crate::gfx::{GFXOperation, GFXState};

pub static BIND_LAYOUT: OnceLock<Arc<wgpu::BindGroupLayout>> = OnceLock::new();

pub struct Camera {
    pub pos: Vec3,
    pub target: Vec3,
    buffer: wgpu::Buffer,
    binding: Arc<wgpu::BindGroup>,
}

impl Camera {
    pub fn new(pos: Vec3, target: Vec3, gfx: &GFXState) -> Self {
        let uniform = CameraUniform(Mat4::IDENTITY.to_cols_array_2d());

        let buffer = gfx
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera buffer"),
                contents: bytemuck::cast_slice(&[uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let bind_layout = BIND_LAYOUT.get_or_init(|| {
            Arc::new(
                gfx.device
                    .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                        label: Some("Camera bind layout"),
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
                    }),
            )
        });

        let binding = Arc::new(gfx.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Camera binding"),
            layout: bind_layout.as_ref(),

            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        }));

        Self {
            pos,
            target,
            buffer,
            binding,
        }
    }

    pub fn draw(&self, queue: &wgpu::Queue) -> Box<dyn GFXOperation> {
        let uniform = CameraUniform(
            (Mat4::perspective_lh(PI / 3.0, 1.0, 0.1, 1000.0)
                * Mat4::look_at_lh(self.pos, self.target, Vec3::Y))
            .to_cols_array_2d(),
        );
        queue.write_buffer(&self.buffer, 0, bytemuck::cast_slice(&[uniform]));
        Box::new(CameraDrawOperation {
            binding: self.binding.clone(),
        })
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform([[f32; 4]; 4]);

struct CameraDrawOperation {
    binding: Arc<wgpu::BindGroup>,
}

impl GFXOperation for CameraDrawOperation {
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_bind_group(0, &self.binding, &[]);
    }
}
