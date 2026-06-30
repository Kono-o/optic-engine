use optic_core::Size2D;

use crate::window::Window;
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::keyboard::{KeyCode, PhysicalKey};

#[derive(Copy, Clone)]
pub struct ButtonState {
    pub pressed: bool,
    pub held: bool,
    pub released: bool,
}

pub struct KeyBitMap(pub [ButtonState; 256]);
pub struct MouseBitMap(pub [ButtonState; 8]);

#[derive(Debug, Clone, Copy)]
pub enum Is {
    Pressed,
    Released,
    Held,
}

#[derive(Debug, Clone, Copy)]
pub enum Mouse {
    Left,
    Right,
    Middle,
    Back,
    Forward,
    Other(u16),
}

fn mouse_index(m: &Mouse) -> usize {
    match m {
        Mouse::Left => 0,
        Mouse::Right => 1,
        Mouse::Middle => 2,
        Mouse::Back => 3,
        Mouse::Forward => 4,
        Mouse::Other(n) => (5 + *n as usize).min(7),
    }
}

fn key_index(kc: &KeyCode) -> usize {
    match kc {
        KeyCode::KeyA => 0, KeyCode::KeyB => 1, KeyCode::KeyC => 2, KeyCode::KeyD => 3,
        KeyCode::KeyE => 4, KeyCode::KeyF => 5, KeyCode::KeyG => 6, KeyCode::KeyH => 7,
        KeyCode::KeyI => 8, KeyCode::KeyJ => 9, KeyCode::KeyK => 10, KeyCode::KeyL => 11,
        KeyCode::KeyM => 12, KeyCode::KeyN => 13, KeyCode::KeyO => 14, KeyCode::KeyP => 15,
        KeyCode::KeyQ => 16, KeyCode::KeyR => 17, KeyCode::KeyS => 18, KeyCode::KeyT => 19,
        KeyCode::KeyU => 20, KeyCode::KeyV => 21, KeyCode::KeyW => 22, KeyCode::KeyX => 23,
        KeyCode::KeyY => 24, KeyCode::KeyZ => 25,
        KeyCode::Digit0 => 26, KeyCode::Digit1 => 27, KeyCode::Digit2 => 28, KeyCode::Digit3 => 29,
        KeyCode::Digit4 => 30, KeyCode::Digit5 => 31, KeyCode::Digit6 => 32, KeyCode::Digit7 => 33,
        KeyCode::Digit8 => 34, KeyCode::Digit9 => 35,
        KeyCode::F1 => 36, KeyCode::F2 => 37, KeyCode::F3 => 38, KeyCode::F4 => 39,
        KeyCode::F5 => 40, KeyCode::F6 => 41, KeyCode::F7 => 42, KeyCode::F8 => 43,
        KeyCode::F9 => 44, KeyCode::F10 => 45, KeyCode::F11 => 46, KeyCode::F12 => 47,
        KeyCode::F13 => 48, KeyCode::F14 => 49, KeyCode::F15 => 50, KeyCode::F16 => 51,
        KeyCode::F17 => 52, KeyCode::F18 => 53, KeyCode::F19 => 54, KeyCode::F20 => 55,
        KeyCode::F21 => 56, KeyCode::F22 => 57, KeyCode::F23 => 58, KeyCode::F24 => 59,
        KeyCode::Escape => 60,
        KeyCode::Enter => 61, KeyCode::Tab => 62, KeyCode::Space => 63, KeyCode::Backspace => 64,
        KeyCode::Delete => 65, KeyCode::Insert => 66,
        KeyCode::Home => 67, KeyCode::End => 68,
        KeyCode::PageUp => 69, KeyCode::PageDown => 70,
        KeyCode::ArrowUp => 71, KeyCode::ArrowDown => 72, KeyCode::ArrowLeft => 73, KeyCode::ArrowRight => 74,
        KeyCode::ShiftLeft => 75, KeyCode::ShiftRight => 76,
        KeyCode::ControlLeft => 77, KeyCode::ControlRight => 78,
        KeyCode::AltLeft => 79, KeyCode::AltRight => 80,
        KeyCode::SuperLeft => 81, KeyCode::SuperRight => 82,
        KeyCode::CapsLock => 83, KeyCode::ScrollLock => 84, KeyCode::NumLock => 85,
        KeyCode::PrintScreen => 86, KeyCode::Pause => 87,
        KeyCode::Minus => 88, KeyCode::Equal => 89,
        KeyCode::BracketLeft => 90, KeyCode::BracketRight => 91,
        KeyCode::Semicolon => 92, KeyCode::Quote => 93, KeyCode::Comma => 94, KeyCode::Period => 95, KeyCode::Slash => 96,
        KeyCode::Backslash => 97, KeyCode::IntlBackslash => 98,
        KeyCode::Numpad0 => 99, KeyCode::Numpad1 => 100, KeyCode::Numpad2 => 101, KeyCode::Numpad3 => 102,
        KeyCode::Numpad4 => 103, KeyCode::Numpad5 => 104, KeyCode::Numpad6 => 105, KeyCode::Numpad7 => 106,
        KeyCode::Numpad8 => 107, KeyCode::Numpad9 => 108,
        KeyCode::NumpadDecimal => 109, KeyCode::NumpadDivide => 110, KeyCode::NumpadMultiply => 111,
        KeyCode::NumpadSubtract => 112, KeyCode::NumpadAdd => 113, KeyCode::NumpadEnter => 114, KeyCode::NumpadEqual => 115,
        KeyCode::ContextMenu => 116,
        _ => 255,
    }
}

fn mouse_from_winit(b: MouseButton) -> Mouse {
    match b {
        MouseButton::Left => Mouse::Left,
        MouseButton::Right => Mouse::Right,
        MouseButton::Middle => Mouse::Middle,
        MouseButton::Back => Mouse::Back,
        MouseButton::Forward => Mouse::Forward,
        MouseButton::Other(n) => Mouse::Other(n),
    }
}

pub struct Events {
    pub keys: KeyBitMap,
    pub mouse_buttons: MouseBitMap,
    pub resize_event: Option<Size2D>,
    pub close_requested: bool,
    keys_to_reset: Vec<KeyCode>,
    mouse_to_reset: Vec<Mouse>,
}

impl Events {
    pub fn new() -> Self {
        Self {
            keys: KeyBitMap([ButtonState { pressed: false, held: false, released: false }; 256]),
            mouse_buttons: MouseBitMap([ButtonState { pressed: false, held: false, released: false }; 8]),
            resize_event: None,
            close_requested: false,
            keys_to_reset: Vec::new(),
            mouse_to_reset: Vec::new(),
        }
    }

    pub fn process_window_event(&mut self, event: &WindowEvent, _window: &Window) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(kc) = event.physical_key {
                    let idx = key_index(&kc);
                    if idx < 256 {
                        let state = &mut self.keys.0[idx];
                        match event.state {
                            ElementState::Pressed => {
                                state.pressed = true;
                                state.held = true;
                            }
                            ElementState::Released => {
                                state.held = false;
                                state.released = true;
                            }
                        }
                        self.keys_to_reset.push(kc);
                    }
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let m = mouse_from_winit(*button);
                let idx = mouse_index(&m);
                if idx < 8 {
                    let mb = &mut self.mouse_buttons.0[idx];
                    match state {
                        ElementState::Pressed => {
                            mb.pressed = true;
                            mb.held = true;
                        }
                        ElementState::Released => {
                            mb.held = false;
                            mb.released = true;
                        }
                    }
                    self.mouse_to_reset.push(m);
                }
            }
            WindowEvent::Resized(size) => {
                self.resize_event = Some(Size2D::from(size.width, size.height));
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            _ => {}
        }
    }

    pub fn update_cursor(&mut self, _window: &Window) {
        self.keys_to_reset.clear();
        self.mouse_to_reset.clear();
    }

    pub fn end_frame(&mut self) {
        for kc in self.keys_to_reset.drain(..) {
            let idx = key_index(&kc);
            if idx < 256 {
                self.keys.0[idx].pressed = false;
                self.keys.0[idx].released = false;
            }
        }
        for m in self.mouse_to_reset.drain(..) {
            let idx = mouse_index(&m);
            if idx < 8 {
                self.mouse_buttons.0[idx].pressed = false;
                self.mouse_buttons.0[idx].released = false;
            }
        }
    }

    pub fn key_combo(&self, primary: KeyCode, modifier: KeyCode, action: Is) -> bool {
        self.key(primary, action) && self.key(modifier, Is::Held)
    }

    pub fn key(&self, kc: KeyCode, action: Is) -> bool {
        let idx = key_index(&kc);
        if idx >= 256 { return false; }
        let s = &self.keys.0[idx];
        match action {
            Is::Pressed => s.pressed,
            Is::Released => s.released,
            Is::Held => s.held,
        }
    }

    pub fn mouse(&self, m: Mouse, action: Is) -> bool {
        let idx = mouse_index(&m);
        if idx >= 8 { return false; }
        let s = &self.mouse_buttons.0[idx];
        match action {
            Is::Pressed => s.pressed,
            Is::Released => s.released,
            Is::Held => s.held,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_index_mapping() {
        assert_eq!(key_index(&KeyCode::KeyA), 0);
        assert_eq!(key_index(&KeyCode::KeyZ), 25);
        assert_eq!(key_index(&KeyCode::Digit0), 26);
        assert_eq!(key_index(&KeyCode::Digit9), 35);
        assert_eq!(key_index(&KeyCode::Escape), 60);
        assert_eq!(key_index(&KeyCode::Space), 63);
        assert_eq!(key_index(&KeyCode::ShiftLeft), 75);
        assert_eq!(key_index(&KeyCode::ControlRight), 78);
        assert_eq!(key_index(&KeyCode::ArrowUp), 71);
        assert_eq!(key_index(&KeyCode::ContextMenu), 116);
    }

    #[test]
    fn key_index_unmapped() {
        assert_eq!(key_index(&KeyCode::AudioVolumeUp), 255);
    }

    #[test]
    fn mouse_index_mapping() {
        assert_eq!(mouse_index(&Mouse::Left), 0);
        assert_eq!(mouse_index(&Mouse::Right), 1);
        assert_eq!(mouse_index(&Mouse::Middle), 2);
        assert_eq!(mouse_index(&Mouse::Back), 3);
        assert_eq!(mouse_index(&Mouse::Forward), 4);
    }

    #[test]
    fn mouse_index_other() {
        assert_eq!(mouse_index(&Mouse::Other(0)), 5);
        assert_eq!(mouse_index(&Mouse::Other(3)), 7);
    }

    #[test]
    fn events_key_pressed() {
        let mut ev = Events::new();
        assert!(!ev.key(KeyCode::KeyA, Is::Pressed));
        assert!(!ev.key(KeyCode::KeyA, Is::Held));
        assert!(!ev.key(KeyCode::KeyA, Is::Released));

        ev.keys.0[key_index(&KeyCode::KeyA)].pressed = true;
        ev.keys.0[key_index(&KeyCode::KeyA)].held = true;
        assert!(ev.key(KeyCode::KeyA, Is::Pressed));
        assert!(ev.key(KeyCode::KeyA, Is::Held));
        assert!(!ev.key(KeyCode::KeyA, Is::Released));
    }

    #[test]
    fn events_key_released() {
        let mut ev = Events::new();
        ev.keys.0[key_index(&KeyCode::KeyA)].held = false;
        ev.keys.0[key_index(&KeyCode::KeyA)].released = true;
        assert!(!ev.key(KeyCode::KeyA, Is::Pressed));
        assert!(!ev.key(KeyCode::KeyA, Is::Held));
        assert!(ev.key(KeyCode::KeyA, Is::Released));
    }

    #[test]
    fn events_key_unmapped() {
        let ev = Events::new();
        assert!(!ev.key(KeyCode::AudioVolumeUp, Is::Held));
    }

    #[test]
    fn events_mouse_pressed() {
        let mut ev = Events::new();
        assert!(!ev.mouse(Mouse::Left, Is::Pressed));
        ev.mouse_buttons.0[0].pressed = true;
        ev.mouse_buttons.0[0].held = true;
        assert!(ev.mouse(Mouse::Left, Is::Pressed));
        assert!(ev.mouse(Mouse::Left, Is::Held));
    }

    #[test]
    fn events_mouse_unmapped() {
        let ev = Events::new();
        assert!(!ev.mouse(Mouse::Other(10), Is::Held));
    }

    #[test]
    fn events_end_frame_resets_press_release() {
        let mut ev = Events::new();
        let idx_a = key_index(&KeyCode::KeyA);
        ev.keys.0[idx_a].pressed = true;
        ev.keys.0[idx_a].held = true;
        ev.keys_to_reset.push(KeyCode::KeyA);

        ev.end_frame();
        assert!(!ev.keys.0[idx_a].pressed);
        assert!(ev.keys.0[idx_a].held);
    }

    #[test]
    fn events_end_frame_mouse_reset() {
        let mut ev = Events::new();
        ev.mouse_buttons.0[0].released = true;
        ev.mouse_buttons.0[0].held = false;
        ev.mouse_to_reset.push(Mouse::Left);

        ev.end_frame();
        assert!(!ev.mouse_buttons.0[0].released);
        assert!(!ev.mouse_buttons.0[0].held);
    }

    #[test]
    fn events_key_combo() {
        let mut ev = Events::new();
        let primary = KeyCode::KeyC;
        let modifier = KeyCode::ControlLeft;

        assert!(!ev.key_combo(primary, modifier, Is::Pressed));

        ev.keys.0[key_index(&modifier)].held = true;
        assert!(!ev.key_combo(primary, modifier, Is::Pressed));

        ev.keys.0[key_index(&primary)].pressed = true;
        ev.keys.0[key_index(&primary)].held = true;
        assert!(ev.key_combo(primary, modifier, Is::Pressed));
    }

    #[test]
    fn events_initial_state() {
        let ev = Events::new();
        assert!(ev.resize_event.is_none());
        assert!(!ev.close_requested);
        assert!(ev.keys_to_reset.is_empty());
        assert!(ev.mouse_to_reset.is_empty());
    }

    #[test]
    fn is_values() {
        match Is::Pressed { _ => {} }
        match Is::Released { _ => {} }
        match Is::Held { _ => {} }
    }

    #[test]
    fn mouse_enum() {
        match Mouse::Left { _ => {} }
        match Mouse::Right { _ => {} }
        match Mouse::Middle { _ => {} }
        match Mouse::Back { _ => {} }
        match Mouse::Forward { _ => {} }
        match Mouse::Other(5) { _ => {} }
    }
}
