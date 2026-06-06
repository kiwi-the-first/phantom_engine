pub mod app;
pub use app::App;
pub mod audio;
pub mod asset_manager;
pub mod game_loader;
pub mod renderer;

// use crate::app::App;

// use anyhow::Result;
// use winit::event_loop::{ControlFlow, EventLoop};

// fn main() -> Result<()> {
//     env_logger::init();

//     let event_loop = EventLoop::new().unwrap();

//     event_loop.set_control_flow(ControlFlow::Poll);

//     let mut app = App::default();

//     event_loop.run_app(&mut app)?;
//     Ok(())
// }
