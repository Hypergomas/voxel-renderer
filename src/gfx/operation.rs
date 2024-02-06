pub trait GFXOperation {
    fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>);
}
