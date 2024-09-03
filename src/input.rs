use std::collections::HashMap;

use glam::Vec2;
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug)]
pub enum UserEvent {
    ExitApp,
}

#[derive(Debug)]
pub struct Inputs {
    actions: HashMap<String, Vec<KeyCode>>,
    pub cursor_in_window: bool,
    pub focused: bool,
    frame_device_events: Vec<DeviceEvent>,
    frame_window_events: Vec<WindowEvent>,
    keys_state: [bool; 256],
    last_keys_state: [bool; 256],
    last_mouse_state: [bool; 32],
    pub mouse_delta: Vec2,
    mouse_state: [bool; 32],
}

impl Default for Inputs {
    fn default() -> Self {
        Self {
            actions: HashMap::new(),
            cursor_in_window: false,
            focused: false,
            frame_device_events: vec![],
            frame_window_events: vec![],
            keys_state: [false; 256],
            last_keys_state: [false; 256],
            last_mouse_state: [false; 32],
            mouse_delta: Vec2::default(),
            mouse_state: [false; 32],
        }
    }
}

impl Inputs {
    fn handle_window_event(&mut self, event: WindowEvent) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                self.handle_key(event);
            }
            WindowEvent::Focused(focused) => self.focused = focused,
            WindowEvent::CursorEntered { .. } => self.cursor_in_window = true,
            WindowEvent::CursorLeft { .. } => self.cursor_in_window = false,
            _ => (),
        };
    }

    fn handle_device_event(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::Button { button, state } => {
                self.handle_mouse_input(button, state);
            }
            DeviceEvent::MouseMotion { delta } => {
                if !self.focused {
                    return;
                }
                self.mouse_delta = Vec2::new(delta.0 as f32, delta.1 as f32);
            }
            _ => (),
        }
    }

    fn handle_key(&mut self, event: KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        if self.last_keys_state[code as usize] && event.repeat {
            return;
        }
        match event.state {
            ElementState::Pressed => self.keys_state[code as usize] = true,
            ElementState::Released => self.keys_state[code as usize] = false,
        };
    }

    fn handle_mouse_input(&mut self, button: u32, state: ElementState) {
        match state {
            ElementState::Pressed => self.mouse_state[button as usize] = true,
            ElementState::Released => self.mouse_state[button as usize] = false,
        }
    }

    pub fn key_pressed(&self, code: KeyCode) -> bool {
        self.keys_state[code as usize]
    }

    pub fn key_just_pressed(&self, code: KeyCode) -> bool {
        self.keys_state[code as usize] && !self.last_keys_state[code as usize]
    }

    pub fn mouse_pressed(&self, button: usize) -> bool {
        self.mouse_state[button]
    }

    pub fn mouse_just_pressed(&self, button: usize) -> bool {
        self.mouse_state[button] && !self.last_mouse_state[button]
    }

    pub fn register_action(&mut self, name: &str, codes: Vec<KeyCode>) {
        self.actions.insert(name.to_string(), codes);
    }

    pub fn action_pressed(&self, action: &str) -> bool {
        self.actions
            .get(action)
            .unwrap_or(&Vec::new())
            .iter()
            .any(|code| self.key_pressed(*code))
    }

    pub fn action_just_pressed(&self, action: &str) -> bool {
        self.actions
            .get(action)
            .unwrap_or(&Vec::new())
            .iter()
            .any(|code| self.key_just_pressed(*code))
    }

    pub fn on_window_event(&mut self, event: WindowEvent) {
        self.frame_window_events.push(event);
    }

    pub fn on_device_event(&mut self, event: DeviceEvent) {
        self.frame_device_events.push(event);
    }

    pub fn update(&mut self) {
        self.last_keys_state.copy_from_slice(&self.keys_state);
        self.last_mouse_state.copy_from_slice(&self.mouse_state);
        self.mouse_delta = Vec2::default();
        while let Some(event) = self.frame_window_events.pop() {
            self.handle_window_event(event);
        }
        while let Some(event) = self.frame_device_events.pop() {
            self.handle_device_event(event);
        }
    }

    pub fn focus_out(&mut self) {
        self.keys_state = [false; 256];
        self.last_keys_state = [false; 256];
        self.mouse_state = [false; 32];
        self.last_mouse_state = [false; 32];
    }
}
