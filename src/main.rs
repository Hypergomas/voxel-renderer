mod camera;
mod gfx;
mod window;
mod world;

use anyhow::Result;
use glam::{Vec2, Vec3};
use pollster::FutureExt;

fn main() -> Result<()> {
    env_logger::init();
    let (window, ev_loop) = window::new()?;
    let mut gfx = gfx::GFXState::new(&window, window.inner_size().into()).block_on()?;

    let mut time = 0.0f32;

    let mut camera = camera::Camera::new(Vec3::ZERO, Vec3::new(8.0, 8.0, 8.0), &gfx);
    let mut cam_angle = 0.0f32;
    let world = world::World::new(&gfx);
    let mut chunk = world::Chunk::new(&gfx);

    log::info!("Starting event loop");
    ev_loop.run(|event, target| {
        use winit::event::{Event, WindowEvent};

        window.request_redraw();

        match event {
            Event::WindowEvent { event, .. } => match event {
                WindowEvent::CloseRequested => target.exit(),
                WindowEvent::Resized(new_size) => gfx.resize(new_size.into()),
                WindowEvent::RedrawRequested => {
                    let delta = 1.0 / 240.0;
                    time += delta / 2.0;
                    cam_angle += 15.0 * delta;
                    let cam_angle_rad: f32 = cam_angle.to_radians();

                    let cam_pos = Vec2::new(cam_angle_rad.cos(), cam_angle_rad.sin())
                        .rotate(Vec2::new(0.0, -24.0));
                    camera.pos.x = 8.0 + cam_pos.x;
                    camera.pos.z = 8.0 + cam_pos.y;
                    camera.pos.y = 8.0 + time.sin() * 16.0;

                    window.set_title(&format!("Voxel renderer | Camera height: {}", camera.pos.y));

                    let ops = vec![camera.draw(&gfx.queue), chunk.draw(&gfx)];
                    let _ = gfx::render(&mut gfx, ops);
                }
                _ => {}
            },
            _ => {}
        }
    })?;

    log::info!("Finished program execution");
    Ok(())
}
