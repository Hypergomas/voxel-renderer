use std::sync::{Arc, OnceLock};

use glam::Vec3;
use wgpu::util::DeviceExt;

use crate::gfx::{GFXOperation, GFXState, Vertex};

static VERTICES: OnceLock<Arc<wgpu::Buffer>> = OnceLock::new();
static PIPELINE: OnceLock<Arc<wgpu::RenderPipeline>> = OnceLock::new();

pub struct Chunk {
    data: [Voxel; Self::U_WIDTH * Self::U_HEIGHT * Self::U_DEPTH],
    needs_rebuilding: bool,
    index_buffer: Arc<wgpu::Buffer>,
    index_count: u32,
}

impl Chunk {
    const WIDTH: u32 = 16;
    const HEIGHT: u32 = 16;
    const DEPTH: u32 = 16;

    const U_WIDTH: usize = 16;
    const U_HEIGHT: usize = 16;
    const U_DEPTH: usize = 16;

    pub fn new(gfx: &GFXState) -> Self {
        Self {
            data: [Voxel::default(); Self::U_WIDTH * Self::U_HEIGHT * Self::U_DEPTH],
            needs_rebuilding: true,
            index_buffer: Arc::new(gfx.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Chunk index buffer"),
                size: ((Self::WIDTH + 1) * (Self::HEIGHT + 1) * (Self::DEPTH + 1) * 3 * 2 * 6)
                    as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })),
            index_count: 0,
        }
    }

    pub fn build_mesh(&mut self, gfx: &GFXState) {
        for x in 0..Self::WIDTH {
            for z in 0..Self::DEPTH {
                self.data[Voxel::idx_at(x, 8, z) as usize].empty = true;
                self.data[Voxel::idx_at(x, 9, z) as usize].empty = true;
                self.data[Voxel::idx_at(x, 10, z) as usize].empty = true;
            }
        }

        let mut indices: Vec<[[u32; 3]; 2]> = vec![];

        for x in 0..Self::DEPTH {
            for y in 0..Self::HEIGHT {
                for z in 0..Self::WIDTH {
                    if self.data[Voxel::idx_at(x, y, z) as usize].empty {
                        continue;
                    }

                    // Top face
                    if y + 1 < Self::HEIGHT {
                        let idx = Voxel::idx_at(x, y + 1, z) as usize;
                        if self.data[idx].empty {
                            indices.push(Voxel::get_face_indices(Face::TOP, x, y, z));
                        }
                    } else {
                        indices.push(Voxel::get_face_indices(Face::TOP, x, y, z));
                    }

                    // Bottom face
                    if y > 0 {
                        let idx = Voxel::idx_at(x, y - 1, z) as usize;
                        if self.data[idx].empty {
                            indices.push(Voxel::get_face_indices(Face::BOTTOM, x, y, z));
                        }
                    } else {
                        indices.push(Voxel::get_face_indices(Face::BOTTOM, x, y, z));
                    }

                    // Eastern face
                    if x + 1 < Self::WIDTH {
                        let idx = Voxel::idx_at(x + 1, y, z) as usize;
                        if self.data[idx].empty {
                            indices.push(Voxel::get_face_indices(Face::EAST, x, y, z));
                        }
                    } else {
                        indices.push(Voxel::get_face_indices(Face::EAST, x, y, z));
                    }

                    // Western face
                    if x > 0 {
                        let idx = Voxel::idx_at(x - 1, y, z) as usize;
                        if self.data[idx].empty {
                            indices.push(Voxel::get_face_indices(Face::WEST, x, y, z));
                        }
                    } else {
                        indices.push(Voxel::get_face_indices(Face::WEST, x, y, z));
                    }

                    // Northern face
                    if z + 1 < Self::DEPTH {
                        let idx = Voxel::idx_at(x, y, z + 1) as usize;
                        if self.data[idx].empty {
                            indices.push(Voxel::get_face_indices(Face::NORTH, x, y, z));
                        }
                    } else {
                        indices.push(Voxel::get_face_indices(Face::NORTH, x, y, z));
                    }

                    // Southern face
                    if z > 0 {
                        let idx = Voxel::idx_at(x, y, z - 1) as usize;
                        if self.data[idx].empty {
                            indices.push(Voxel::get_face_indices(Face::SOUTH, x, y, z));
                        }
                    } else {
                        indices.push(Voxel::get_face_indices(Face::SOUTH, x, y, z));
                    }
                }
            }
        }

        gfx.queue
            .write_buffer(&self.index_buffer, 0, bytemuck::cast_slice(&indices));
        self.index_count = indices.len() as u32 * 3 * 2;
    }

    pub fn draw(&mut self, gfx: &GFXState) -> Box<dyn GFXOperation> {
        let vertex_buffer = VERTICES
            .get_or_init(|| Self::build_vertex_grid(gfx))
            .clone();
        let pipeline = PIPELINE.get_or_init(|| Self::build_pipeline(gfx)).clone();

        if self.needs_rebuilding {
            self.build_mesh(gfx);
            self.needs_rebuilding = false;
        }

        Box::new(ChunkDrawOperation {
            vertex_buffer,
            index_buffer: self.index_buffer.clone(),
            index_count: self.index_count,
            pipeline,
        })
    }

    fn build_vertex_grid(gfx: &GFXState) -> Arc<wgpu::Buffer> {
        let mut vertices: Vec<Vertex> = vec![];

        for z in 0..Self::DEPTH + 1 {
            for y in 0..Self::HEIGHT + 1 {
                for x in 0..Self::WIDTH + 1 {
                    vertices.push(Vertex::new(Vec3::new(
                        -0.5 + 1.0 * x as f32,
                        -0.5 + 1.0 * y as f32,
                        -0.5 + 1.0 * z as f32,
                    )));
                }
            }
        }

        Arc::new(
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chunk vertex buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        )
    }

    fn build_pipeline(gfx: &GFXState) -> Arc<wgpu::RenderPipeline> {
        let shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Chunk shader"),
                source: wgpu::ShaderSource::Wgsl(assets::shaders::CHUNK.into()),
            });

        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Chunk pipeline layout"),
                bind_group_layouts: &[crate::camera::BIND_LAYOUT.get().unwrap()],
                push_constant_ranges: &[],
            });

        Arc::new(
            gfx.device
                .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                    label: Some("Chunk pipeline"),
                    layout: Some(&pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &shader,
                        entry_point: "vs_main",
                        buffers: &[Vertex::desc()],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &shader,
                        entry_point: "fs_main",
                        targets: &[Some(wgpu::ColorTargetState {
                            format: gfx.config.format,
                            blend: Some(wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING),
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
                }),
        )
    }
}

struct ChunkDrawOperation {
    vertex_buffer: Arc<wgpu::Buffer>,
    index_buffer: Arc<wgpu::Buffer>,
    index_count: u32,
    pipeline: Arc<wgpu::RenderPipeline>,
}

impl GFXOperation for ChunkDrawOperation {
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        pass.draw_indexed(0..self.index_count, 0, 0..1);
    }
}

/// Represents the center of a voxel. The actual render coordinates span 0.5 to all sides.
#[derive(Clone, Copy)]
pub struct Voxel {
    pub color: Vec3,
    pub empty: bool,
}

impl Voxel {
    pub const fn idx_at(x: u32, y: u32, z: u32) -> u32 {
        x + y * Chunk::WIDTH + (z * Chunk::WIDTH * Chunk::HEIGHT)
    }

    pub const fn vertex_idx_at(x: u32, y: u32, z: u32) -> u32 {
        x + y * (Chunk::WIDTH + 1) + (z * (Chunk::WIDTH + 1) * (Chunk::HEIGHT + 1))
    }

    pub fn get_face_indices(face: Face, x: u32, y: u32, z: u32) -> [[u32; 3]; 2] {
        match face {
            Face::TOP => [
                [
                    Voxel::vertex_idx_at(x + 1, y + 1, z),
                    Voxel::vertex_idx_at(x + 1, y + 1, z + 1),
                    Voxel::vertex_idx_at(x, y + 1, z + 1),
                ],
                [
                    Voxel::vertex_idx_at(x, y + 1, z),
                    Voxel::vertex_idx_at(x + 1, y + 1, z),
                    Voxel::vertex_idx_at(x, y + 1, z + 1),
                ],
            ],
            Face::BOTTOM => [
                [
                    Voxel::vertex_idx_at(x + 1, y, z + 1),
                    Voxel::vertex_idx_at(x + 1, y, z),
                    Voxel::vertex_idx_at(x, y, z),
                ],
                [
                    Voxel::vertex_idx_at(x, y, z + 1),
                    Voxel::vertex_idx_at(x + 1, y, z + 1),
                    Voxel::vertex_idx_at(x, y, z),
                ],
            ],
            Face::EAST => [
                [
                    Voxel::vertex_idx_at(x + 1, y, z + 1),
                    Voxel::vertex_idx_at(x + 1, y + 1, z + 1),
                    Voxel::vertex_idx_at(x + 1, y + 1, z),
                ],
                [
                    Voxel::vertex_idx_at(x + 1, y, z + 1),
                    Voxel::vertex_idx_at(x + 1, y + 1, z),
                    Voxel::vertex_idx_at(x + 1, y, z),
                ],
            ],
            Face::WEST => [
                [
                    Voxel::vertex_idx_at(x, y, z),
                    Voxel::vertex_idx_at(x, y + 1, z),
                    Voxel::vertex_idx_at(x, y + 1, z + 1),
                ],
                [
                    Voxel::vertex_idx_at(x, y, z + 1),
                    Voxel::vertex_idx_at(x, y, z),
                    Voxel::vertex_idx_at(x, y + 1, z + 1),
                ],
            ],
            Face::NORTH => [
                [
                    Voxel::vertex_idx_at(x, y, z + 1),
                    Voxel::vertex_idx_at(x, y + 1, z + 1),
                    Voxel::vertex_idx_at(x + 1, y + 1, z + 1),
                ],
                [
                    Voxel::vertex_idx_at(x + 1, y, z + 1),
                    Voxel::vertex_idx_at(x, y, z + 1),
                    Voxel::vertex_idx_at(x + 1, y + 1, z + 1),
                ],
            ],
            Face::SOUTH => [
                [
                    Voxel::vertex_idx_at(x + 1, y, z),
                    Voxel::vertex_idx_at(x + 1, y + 1, z),
                    Voxel::vertex_idx_at(x, y + 1, z),
                ],
                [
                    Voxel::vertex_idx_at(x, y, z),
                    Voxel::vertex_idx_at(x + 1, y, z),
                    Voxel::vertex_idx_at(x, y + 1, z),
                ],
            ],
        }
    }
}

impl Default for Voxel {
    fn default() -> Self {
        Self {
            color: Vec3::X,
            empty: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Face {
    TOP,
    BOTTOM,
    EAST,
    WEST,
    NORTH,
    SOUTH,
}
