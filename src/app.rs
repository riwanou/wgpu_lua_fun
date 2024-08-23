use std::sync::Arc;
use std::time::{Duration, Instant};

use log::info;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

use crate::input::{Inputs, UserEvent};
use crate::lua::LuaState;
use crate::render::state::RenderState;

pub struct App {
    current: Instant,
    elapsed: Duration,
    inputs: Inputs,
    lua: LuaState,
    proxy: EventLoopProxy<UserEvent>,
    render_state: Option<RenderState>,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Self {
        Self {
            current: Instant::now(),
            elapsed: Duration::default(),
            inputs: Inputs::default(),
            lua: LuaState::new("main").unwrap(),
            proxy,
            render_state: None,
            window: None,
        }
    }

    pub fn init(&mut self) {
        let window = self.window.clone().unwrap();
        self.render_state = Some(pollster::block_on(RenderState::new(window)));
    }

    pub fn update(&mut self) {
        let delta = self.current.elapsed();
        self.elapsed += delta;
        self.current = Instant::now();

        self.inputs.update();
        if self.inputs.key_pressed(KeyCode::Escape) {
            self.proxy.send_event(UserEvent::ExitApp).unwrap();
        }

        self.lua.update(delta.as_secs_f32()).unwrap();

        self.render_state.as_mut().unwrap().render();
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.window = Some(Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("bloup")
                        .with_inner_size(LogicalSize::new(720, 550))
                        .with_position(LogicalPosition::new(880, 0))
                        .with_window_level(
                            winit::window::WindowLevel::AlwaysOnTop,
                        ),
                )
                .unwrap(),
        ));
        self.init();
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        match event {
            UserEvent::ExitApp => {
                info!("User event: exit app");
                event_loop.exit();
            }
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                self.render_state.as_mut().unwrap().resize(size);
            }
            WindowEvent::RedrawRequested => {
                self.update();
                self.window.as_ref().unwrap().request_redraw();
            }
            _ => (),
        }
        self.inputs.on_event(event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window.as_ref().unwrap().request_redraw();
    }
}
