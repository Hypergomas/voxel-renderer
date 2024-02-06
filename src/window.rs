use anyhow::Result;
use winit::{
    dpi::PhysicalSize,
    event_loop::EventLoop,
    window::{Window, WindowBuilder, WindowButtons},
};

pub fn new() -> Result<(Window, EventLoop<()>)> {
    let ev_loop = EventLoop::new()?;
    log::info!("Created event loop");

    let window = WindowBuilder::new()
        .with_enabled_buttons(WindowButtons::CLOSE | WindowButtons::MINIMIZE)
        .with_inner_size(PhysicalSize::new(1024, 1024))
        .with_resizable(false)
        .with_title("Voxel renderer")
        .build(&ev_loop)?;
    log::info!("Created window");

    Ok((window, ev_loop))
}
