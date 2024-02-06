pub mod chunk;

use crate::gfx::{GFXOperation, GFXState};
pub use chunk::*;
use std::sync::Arc;

pub struct World {
    pipeline: Arc<wgpu::RenderPipeline>,
}

impl World {
    pub fn new(gfx: &GFXState) -> Self {
        let shader = gfx
            .device
            .create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("Triangle shader"),
                source: wgpu::ShaderSource::Wgsl(assets::shaders::TRIANGLE.into()),
            });

        let pipeline_layout = gfx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Triangle pipeline layout"),
                bind_group_layouts: &[crate::camera::BIND_LAYOUT.get().unwrap()],
                push_constant_ranges: &[],
            });

        let pipeline = gfx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("Triangle pipeline"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: "vs_main",
                    buffers: &[],
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
            });

        Self {
            pipeline: Arc::new(pipeline),
        }
    }

    pub fn draw(&self) -> Box<dyn GFXOperation> {
        Box::new(WorldDrawOperation {
            pipeline: self.pipeline.clone(),
        })
    }
}

struct WorldDrawOperation {
    pipeline: Arc<wgpu::RenderPipeline>,
}

impl GFXOperation for WorldDrawOperation {
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        pass.set_pipeline(&self.pipeline);
        pass.draw(0..3, 0..1);
    }
}
