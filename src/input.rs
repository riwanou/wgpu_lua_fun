use std::collections::HashMap;

use winit::{
    event::{ElementState, KeyEvent, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Debug)]
pub enum UserEvent {
    ExitApp,
}

#[derive(Debug)]
pub struct Inputs {
    frame_events: Vec<WindowEvent>,
    keys_state: [bool; 256],
    last_keys_state: [bool; 256],
    actions: HashMap<String, Vec<KeyCode>>,
}

impl Default for Inputs {
    fn default() -> Self {
        Self {
            frame_events: vec![],
            keys_state: [false; 256],
            last_keys_state: [false; 256],
            actions: HashMap::new(),
        }
    }
}

impl Inputs {
    fn handle_event(&mut self, event: WindowEvent) {
        if let WindowEvent::KeyboardInput { event, .. } = event {
            self.handle_key(event)
        }
    }

    fn handle_key(&mut self, event: KeyEvent) {
        let PhysicalKey::Code(code) = event.physical_key else {
            return;
        };
        match event.state {
            ElementState::Pressed => self.keys_state[code as usize] = true,
            ElementState::Released => self.keys_state[code as usize] = false,
        };
    }

    pub fn key_pressed(&self, code: KeyCode) -> bool {
        self.keys_state[code as usize]
    }

    pub fn key_just_pressed(&self, code: KeyCode) -> bool {
        self.keys_state[code as usize] && !self.last_keys_state[code as usize]
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

    pub fn on_event(&mut self, event: WindowEvent) {
        self.frame_events.push(event);
    }

    pub fn update(&mut self) {
        self.last_keys_state.copy_from_slice(&self.keys_state);
        while let Some(event) = self.frame_events.pop() {
            self.handle_event(event);
        }
    }
}
