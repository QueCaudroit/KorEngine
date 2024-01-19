use winit::event::{self, DeviceEvent, ElementState};

pub struct Input {
    pub mouse: MouseInput,
    pub keyboard: KeyboardInput,
    pub gamepad: GamepadInput,
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    pub fn new() -> Self {
        Self {
            mouse: MouseInput::new(),
            keyboard: KeyboardInput::new(),
            gamepad: GamepadInput {},
        }
    }

    pub fn update(&mut self, event: DeviceEvent) {
        match event {
            DeviceEvent::MouseMotion { delta: (x, y) } => self.mouse.update_delta(x, y),
            DeviceEvent::Key(event::KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => {
                let key_state = &mut self.keyboard.keys[key as usize];
                match state {
                    ElementState::Pressed => {
                        key_state.pressed = true;
                        key_state.state = true;
                    }
                    ElementState::Released => {
                        key_state.released = true;
                        key_state.state = false;
                    }
                }
            }
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.mouse.reset();
        self.keyboard.reset();
    }
}

pub struct MouseInput {
    pub raw_x: f64,
    pub raw_y: f64,
}

impl Default for MouseInput {
    fn default() -> Self {
        Self::new()
    }
}

impl MouseInput {
    pub fn new() -> Self {
        Self {
            raw_x: 0.0,
            raw_y: 0.0,
        }
    }

    pub fn update_delta(&mut self, x: f64, y: f64) {
        self.raw_x += x;
        self.raw_y += y;
    }

    pub fn reset(&mut self) {
        self.raw_x = 0.0;
        self.raw_y = 0.0;
    }
}
pub struct KeyboardInput {
    pub keys: [KeyState; 163],
}

impl Default for KeyboardInput {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyboardInput {
    pub fn new() -> Self {
        Self {
            keys: [KeyState::new(); 163],
        }
    }

    pub fn reset(&mut self) {
        for key in &mut self.keys {
            key.reset();
        }
    }
}

#[derive(Copy, Clone)]
pub struct KeyState {
    pub state: bool,
    pub released: bool,
    pub pressed: bool,
}

impl Default for KeyState {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyState {
    pub fn new() -> Self {
        Self {
            state: false,
            released: false,
            pressed: false,
        }
    }

    pub fn reset(&mut self) {
        self.released = false;
        self.pressed = false;
    }
}

pub struct GamepadInput {
    // TODO
}
