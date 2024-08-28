use std::sync::{Arc, Once};
use std::time::{Duration, Instant};

use anyhow::Result;
use log::info;
use threadpool::ThreadPool;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalPosition, LogicalSize};
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoopProxy};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId, WindowLevel};

use crate::input::{Inputs, UserEvent};
use crate::lua::LuaState;
use crate::render::state::RenderState;
use crate::scene::Scene;

pub const RELOAD_DEBOUNCE: Duration = Duration::from_millis(200);

static INIT: Once = Once::new();
static mut THREAD_POOL: Option<Arc<ThreadPool>> = None;

pub fn get_pool() -> Arc<ThreadPool> {
    unsafe {
        INIT.call_once(|| {
            THREAD_POOL = Some(Arc::new(ThreadPool::new(4)));
        });
        THREAD_POOL.clone().expect("Thread pool is not initialized")
    }
}

pub struct App {
    current: Instant,
    elapsed: Duration,
    inputs: Inputs,
    lua: LuaState,
    not_on_top: bool,
    proxy: EventLoopProxy<UserEvent>,
    render_state: Option<RenderState>,
    scene: Scene,
    window: Option<Arc<Window>>,
}

impl App {
    pub fn new(proxy: EventLoopProxy<UserEvent>, not_on_top: bool) -> Self {
        Self {
            not_on_top,
            current: Instant::now(),
            elapsed: Duration::default(),
            inputs: Inputs::default(),
            lua: LuaState::new("main"),
            proxy,
            render_state: None,
            scene: Scene::new(),
            window: None,
        }
    }

    fn window(&self) -> Arc<Window> {
        self.window.clone().expect("Window not created")
    }

    pub fn init(&mut self) -> Result<()> {
        self.render_state =
            Some(pollster::block_on(RenderState::new(self.window())));
        self.lua
            .init(self.render_state.as_mut().unwrap(), &mut self.scene)?;
        Ok(())
    }

    pub fn update(&mut self) -> Result<()> {
        let delta = self.current.elapsed();
        self.elapsed += delta;
        self.current = Instant::now();
        let delta_sec = delta.as_secs_f32();
        let elapsed_sec = self.elapsed.as_secs_f32();

        let lua = &mut self.lua;
        let render_state = self.render_state.as_mut().unwrap();

        self.scene.begin_frame();
        lua.update(render_state, &mut self.scene, delta_sec)?;

        self.inputs.update();
        if self.inputs.key_pressed(KeyCode::Escape) {
            self.proxy.send_event(UserEvent::ExitApp)?;
        }
        if self.inputs.key_pressed(KeyCode::KeyR) {
            lua.init(render_state, &mut self.scene)?;
        }

        render_state.hot_reload();
        render_state.render(elapsed_sec, &mut self.scene);

        Ok(())
    }
}

impl ApplicationHandler<UserEvent> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_level = match self.not_on_top {
            true => WindowLevel::Normal,
            false => WindowLevel::AlwaysOnTop,
        };
        self.window = Some(Arc::new(
            event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title("bloup")
                        .with_inner_size(LogicalSize::new(720, 550))
                        .with_position(LogicalPosition::new(880, 0))
                        .with_window_level(window_level),
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
                self.render_state.as_mut().unwrap().resize(size);
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
