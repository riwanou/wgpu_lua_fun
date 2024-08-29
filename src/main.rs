use std::env;

use app::App;
use env_logger::Env;
use input::UserEvent;
use winit::event_loop::{self, EventLoop};

mod app;
mod input;
mod lua;
mod render;
mod scene;
mod transform;

fn main() {
    env_logger::Builder::from_env(
        Env::default().filter_or("RUST_LOG", "wgpu_lua_fun=info,wgpu=warn"),
    )
    .init();
    let not_on_top = env::var("NOT_ON_TOP").is_ok();

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);

    let mut app = App::new(event_loop.create_proxy(), not_on_top);
    event_loop.run_app(&mut app).unwrap();
}
