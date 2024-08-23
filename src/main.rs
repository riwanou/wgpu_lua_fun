use app::App;
use env_logger::Env;
use input::UserEvent;
use winit::event_loop::{self, EventLoop};

mod app;
mod input;
mod lua;
mod render;

fn main() {
    env_logger::Builder::from_env(
        Env::default().filter_or("RUST_LOG", "wgpu_lua_fun=info,wgpu=warn"),
    )
    .init();

    let event_loop = EventLoop::<UserEvent>::with_user_event().build().unwrap();
    event_loop.set_control_flow(event_loop::ControlFlow::Poll);
    let mut app = App::new(event_loop.create_proxy());
    event_loop.run_app(&mut app).unwrap();
}
