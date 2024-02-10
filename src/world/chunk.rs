use std::sync::{Arc, OnceLock};

use glam::Vec3;
use itertools::Itertools;
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

    const EVENS_SIZE: u32 = Self::WIDTH * Self::HEIGHT;
    const ODDS_SIZE: u32 = Self::WIDTH * (Self::HEIGHT + 1) + (Self::WIDTH + 1) * Self::HEIGHT;

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

    pub fn voxel(&self, x: u32, y: u32, z: u32) -> Option<&Voxel> {
        let idx = x + y * Chunk::WIDTH + (z * Chunk::WIDTH * Chunk::HEIGHT);
        if x >= Self::WIDTH || y >= Self::HEIGHT || z >= Self::DEPTH {
            return None;
        }

        self.data.get(idx as usize)
    }

    pub fn voxel_mut(&mut self, x: u32, y: u32, z: u32) -> Option<&mut Voxel> {
        let idx = x + y * Chunk::WIDTH + (z * Chunk::WIDTH * Chunk::HEIGHT);
        if x >= Self::WIDTH || y >= Self::HEIGHT || z >= Self::DEPTH {
            return None;
        }

        self.data.get_mut(idx as usize)
    }

    /// Calculates the total amount of vertices in this chunk.
    pub fn vertex_count() -> u32 {
        (Self::EVENS_SIZE * (Self::DEPTH + 1)) + (Self::ODDS_SIZE * Self::DEPTH)
    }

    pub fn vertex_at(x: u32, y: u32, z: u32, direction: Direction) -> u32 {
        match direction {
            Direction::TOP => {
                (z + 1) * Self::EVENS_SIZE
                    + z * Self::ODDS_SIZE
                    + x
                    + (y + 1) * Self::HEIGHT
                    + (y + 1) * (Self::HEIGHT + 1)
            }
            Direction::BOTTOM => {
                (z + 1) * Self::EVENS_SIZE
                    + z * Self::ODDS_SIZE
                    + x
                    + y * Self::HEIGHT
                    + y * (Self::HEIGHT + 1)
            }
            Direction::NORTH => (z * Self::EVENS_SIZE + z * Self::ODDS_SIZE + x + y * Self::HEIGHT),
            Direction::SOUTH => {
                (z + 1) * Self::EVENS_SIZE + (z + 1) * Self::ODDS_SIZE + x + y * Self::HEIGHT
            }
            Direction::EAST => {
                (z + 1) * Self::EVENS_SIZE
                    + z * Self::ODDS_SIZE
                    + (x + 1)
                    + (y + 1) * Self::HEIGHT
                    + y * (Self::HEIGHT + 1)
            }
            Direction::WEST => {
                (z + 1) * Self::EVENS_SIZE
                    + z * Self::ODDS_SIZE
                    + x
                    + (y + 1) * Self::HEIGHT
                    + y * (Self::HEIGHT + 1)
            }
        }
    }

    pub fn build_mesh(&mut self, gfx: &GFXState) {
        let mut indices: Vec<[u32; 3]> = vec![];

        for z in 0..1 {
            for y in 0..Self::HEIGHT {
                for x in 0..Self::WIDTH {
                    let voxel = self.voxel(x, y, z).unwrap();
                    if voxel.empty {
                        continue;
                    }

                    indices.push([
                        Self::vertex_at(x, y, z, Direction::EAST),
                        Self::vertex_at(x, y, z, Direction::TOP),
                        Self::vertex_at(x, y, z, Direction::WEST),
                    ]);
                    indices.push([
                        Self::vertex_at(x, y, z, Direction::BOTTOM),
                        Self::vertex_at(x, y, z, Direction::EAST),
                        Self::vertex_at(x, y, z, Direction::WEST),
                    ]);

                    if !self.voxel(x + 1, y, z).map(|v| v.empty).unwrap_or(true) {
                        indices.push([
                            Self::vertex_at(x, y, z, Direction::EAST),
                            Self::vertex_at(x, y, z, Direction::TOP),
                            Self::vertex_at(x + 1, y, z, Direction::TOP),
                        ]);
                        indices.push([
                            Self::vertex_at(x + 1, y, z, Direction::BOTTOM),
                            Self::vertex_at(x, y, z, Direction::EAST),
                            Self::vertex_at(x, y, z, Direction::BOTTOM),
                        ]);
                    }

                    if y > 0 {
                        if !self.voxel(x, y - 1, z).map(|v| v.empty).unwrap_or(true) {
                            indices.push([
                                Self::vertex_at(x, y - 1, z, Direction::WEST),
                                Self::vertex_at(x, y, z, Direction::BOTTOM),
                                Self::vertex_at(x, y, z, Direction::WEST),
                            ]);
                            indices.push([
                                Self::vertex_at(x, y - 1, z, Direction::EAST),
                                Self::vertex_at(x, y, z, Direction::EAST),
                                Self::vertex_at(x, y, z, Direction::BOTTOM),
                            ]);
                        }
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
        let pipeline = PIPELINE
            .get_or_init(|| Self::build_pipeline(gfx, true))
            .clone();

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

    /// Build the `Chunk`'s vertex grid in a diamond pattern.
    /// In this pattern, each voxel has one vertex at a distance of 0.5 units in any given direction.
    fn build_vertex_grid(gfx: &GFXState) -> Arc<wgpu::Buffer> {
        // Horizontal (X axis) lines
        let evens_line = || (0..Self::WIDTH).into_iter().map(|x| x as f32).collect_vec();
        let odds_line = || {
            (0..Self::WIDTH + 1)
                .into_iter()
                .map(|x| x as f32 - 0.5)
                .collect_vec()
        };

        // Vertical stacks
        let evens_stack = std::iter::once_with(|| {
            (0..Self::HEIGHT)
                .into_iter()
                .map(|y| {
                    (
                        (0..Self::WIDTH).into_iter().map(|x| x as f32).collect_vec(),
                        y as f32,
                    )
                })
                .collect_vec()
        });
        let odds_stack = std::iter::once_with(|| {
            (0..Self::HEIGHT * 2 + 1)
                .into_iter()
                .map(|y| {
                    (
                        if y % 2 == 0 {
                            evens_line()
                        } else {
                            odds_line()
                        },
                        y as f32 * 0.5 - 0.5,
                    )
                })
                .collect_vec()
        });
        let mut stacks = evens_stack.interleave(odds_stack).cycle();

        let depth = (0..Self::DEPTH * 2 + 1)
            .into_iter()
            .map(|z| z as f32 * 0.5 - 0.5);

        let vertices = depth
            .clone()
            .map(|z| {
                stacks
                    .next()
                    .map(|stack: Vec<(Vec<f32>, f32)>| {
                        stack
                            .into_iter()
                            .map(|(line, y)| {
                                line.into_iter()
                                    .map(move |x| Vertex::new(Vec3::new(x, y, z)))
                                    .collect_vec()
                            })
                            .flatten()
                            .collect_vec()
                    })
                    .unwrap()
            })
            .flatten()
            .collect_vec();

        // Create vertex buffer
        Arc::new(
            gfx.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Chunk vertex buffer"),
                    contents: bytemuck::cast_slice(&vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                }),
        )
    }

    fn build_pipeline(gfx: &GFXState, wireframe: bool) -> Arc<wgpu::RenderPipeline> {
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
                        cull_mode: None, //Some(wgpu::Face::Back),
                        polygon_mode: if wireframe {
                            wgpu::PolygonMode::Line
                        } else {
                            wgpu::PolygonMode::Fill
                        },
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

impl Default for Voxel {
    fn default() -> Self {
        Self {
            color: Vec3::X,
            empty: false,
        }
    }
}

#[derive(Clone, Copy)]
pub enum Direction {
    /// Positive Y axis (+Y).
    TOP,
    /// Negative Y axis (-Y).
    BOTTOM,
    /// Negative X axis (-X).
    EAST,
    /// Positive X axis (+X).
    WEST,
    /// Negative Z axis (-Z).
    NORTH,
    /// Positive Z axis (+Z).
    SOUTH,
}
