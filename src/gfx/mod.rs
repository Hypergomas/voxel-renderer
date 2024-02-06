mod operation;
mod state;
mod vertex;

use anyhow::Result;
pub use operation::*;
pub use state::*;
pub use vertex::*;

pub fn render(state: &mut GFXState, ops: Vec<Box<dyn GFXOperation>>) -> Result<()> {
    let output = state.surface.get_current_texture()?;
    let view = output
        .texture
        .create_view(&wgpu::TextureViewDescriptor::default());
    let mut encoder = state
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render encoder"),
        });

    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Render pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });

        for op in &ops {
            op.draw(&mut pass);
        }
    }

    state.queue.submit(std::iter::once(encoder.finish()));
    output.present();

    Ok(())
}
