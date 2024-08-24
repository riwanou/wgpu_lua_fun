use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::Result;
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

pub const RELOAD_DEBOUNCE: Duration = Duration::from_millis(200);

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
    pub fn new(proxy: EventLoopProxy<UserEvent>) -> Result<Self> {
        Ok(Self {
            current: Instant::now(),
            elapsed: Duration::default(),
            inputs: Inputs::default(),
            lua: LuaState::new("main")?,
            proxy,
            render_state: None,
            window: None,
        })
    }

    fn window(&self) -> Arc<Window> {
        self.window.clone().expect("Window not created")
    }

    fn render_state_mut(&mut self) -> &mut RenderState {
        self.render_state
            .as_mut()
            .expect("Render state not created")
    }

    pub fn init(&mut self) -> Result<()> {
        self.render_state =
            Some(pollster::block_on(RenderState::new(self.window())));
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let delta = self.current.elapsed();
        self.elapsed += delta;
        self.current = Instant::now();

        self.inputs.update();
        if self.inputs.key_pressed(KeyCode::Escape) {
            self.proxy.send_event(UserEvent::ExitApp)?;
        }

        self.lua.update(delta.as_secs_f32())?;

        let render_state = self.render_state_mut();
        render_state.hot_reload();
        render_state.render();

        Ok(())
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
                .expect("Could not create window"),
        ));
        self.init().unwrap();
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
                self.render_state_mut().resize(size);
            }
            WindowEvent::RedrawRequested => {
                self.update().unwrap();
                self.window().request_redraw();
            }
            _ => (),
        }
        self.inputs.on_event(event);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        self.window().request_redraw();
    }
}
