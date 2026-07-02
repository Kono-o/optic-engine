use optic_core::Size2D;

use crate::window::Window;
use gilrs;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::keyboard::{ModifiersState, PhysicalKey};
pub use winit::keyboard::KeyCode;

// ── Core types ──────────────────────────────────────────────────────────

#[derive(Copy, Clone)]
pub struct ButtonState {
    pub held: bool,
    pub press_frame: u64,
    pub release_frame: u64,
}

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

fn check_state(s: &ButtonState, action: Is, frame: u64) -> bool {
    match action {
        Is::Pressed => s.press_frame == frame,
        Is::Released => s.release_frame == frame,
        Is::Held => s.held,
    }
}

// ── Gamepad types ────────────────────────────────────────────────────────

pub const MAX_GAMEPADS: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadButton {
    A, B, X, Y,
    LB, RB, LT, RT,
    Back, Start, Guide,
    LeftStick, RightStick,
    DPadUp, DPadDown, DPadLeft, DPadRight,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadAxis {
    LeftX, LeftY,
    RightX, RightY,
    LeftTrigger, RightTrigger,
}

fn gamepad_button_index(b: &GamepadButton) -> usize {
    match b {
        GamepadButton::A => 0,
        GamepadButton::B => 1,
        GamepadButton::X => 2,
        GamepadButton::Y => 3,
        GamepadButton::LB => 4,
        GamepadButton::RB => 5,
        GamepadButton::LT => 6,
        GamepadButton::RT => 7,
        GamepadButton::Back => 8,
        GamepadButton::Start => 9,
        GamepadButton::Guide => 10,
        GamepadButton::LeftStick => 11,
        GamepadButton::RightStick => 12,
        GamepadButton::DPadUp => 13,
        GamepadButton::DPadDown => 14,
        GamepadButton::DPadLeft => 15,
        GamepadButton::DPadRight => 16,
        GamepadButton::Other(n) => 17 + (*n as usize).min(2),
    }
}

fn gamepad_axis_index(a: &GamepadAxis) -> usize {
    match a {
        GamepadAxis::LeftX => 0,
        GamepadAxis::LeftY => 1,
        GamepadAxis::RightX => 2,
        GamepadAxis::RightY => 3,
        GamepadAxis::LeftTrigger => 4,
        GamepadAxis::RightTrigger => 5,
    }
}

fn gamepad_button_from_gilrs(b: gilrs::Button) -> GamepadButton {
    use gilrs::Button;
    match b {
        Button::South => GamepadButton::A,
        Button::East => GamepadButton::B,
        Button::West => GamepadButton::X,
        Button::North => GamepadButton::Y,
        Button::LeftTrigger => GamepadButton::LB,
        Button::RightTrigger => GamepadButton::RB,
        Button::LeftTrigger2 => GamepadButton::LT,
        Button::RightTrigger2 => GamepadButton::RT,
        Button::Select => GamepadButton::Back,
        Button::Start => GamepadButton::Start,
        Button::Mode => GamepadButton::Guide,
        Button::LeftThumb => GamepadButton::LeftStick,
        Button::RightThumb => GamepadButton::RightStick,
        Button::DPadUp => GamepadButton::DPadUp,
        Button::DPadDown => GamepadButton::DPadDown,
        Button::DPadLeft => GamepadButton::DPadLeft,
        Button::DPadRight => GamepadButton::DPadRight,
        Button::Unknown => GamepadButton::Other(0),
        _ => GamepadButton::Other(1),
    }
}

fn gamepad_axis_from_gilrs(a: gilrs::Axis) -> GamepadAxis {
    use gilrs::Axis;
    match a {
        Axis::LeftStickX => GamepadAxis::LeftX,
        Axis::LeftStickY => GamepadAxis::LeftY,
        Axis::RightStickX => GamepadAxis::RightX,
        Axis::RightStickY => GamepadAxis::RightY,
        Axis::LeftZ => GamepadAxis::LeftTrigger,
        Axis::RightZ => GamepadAxis::RightTrigger,
        _ => GamepadAxis::LeftX,
    }
}

// ── Events ───────────────────────────────────────────────────────────────

pub struct Events {
    pub keys: [ButtonState; 256],
    pub mouse_buttons: [ButtonState; 8],
    pub mouse_scroll_line: Option<(f32, f32)>,
    pub mouse_scroll_pixel: Option<(f64, f64)>,
    pub modifiers: ModifiersState,
    pub gamepad_connected: [bool; MAX_GAMEPADS],
    pub gamepad_buttons: [[ButtonState; 20]; MAX_GAMEPADS],
    pub gamepad_axes: [[f32; 6]; MAX_GAMEPADS],
    pub resize_event: Option<Size2D>,
    pub close_requested: bool,
    pub focused: bool,
    pub frame: u64,
}

fn empty_buttons<const N: usize>() -> [ButtonState; N] {
    [ButtonState { held: false, press_frame: 0, release_frame: 0 }; N]
}

impl Events {
    pub fn new() -> Self {
        Self {
            keys: empty_buttons::<256>(),
            mouse_buttons: empty_buttons::<8>(),
            mouse_scroll_line: None,
            mouse_scroll_pixel: None,
            modifiers: ModifiersState::default(),
            gamepad_connected: [false; MAX_GAMEPADS],
            gamepad_buttons: [empty_buttons::<20>(); MAX_GAMEPADS],
            gamepad_axes: [[0.0f32; 6]; MAX_GAMEPADS],
            resize_event: None,
            close_requested: false,
            focused: true,
            frame: 1,
        }
    }

    pub fn clear(&mut self) {
        self.keys = empty_buttons::<256>();
        self.mouse_buttons = empty_buttons::<8>();
        self.mouse_scroll_line = None;
        self.mouse_scroll_pixel = None;
        self.gamepad_buttons = [empty_buttons::<20>(); MAX_GAMEPADS];
        self.gamepad_axes = [[0.0f32; 6]; MAX_GAMEPADS];
        self.resize_event = None;
        self.close_requested = false;
        self.focused = true;
        self.frame = 1;
    }

    // ── Event processing ──────────────────────────────────────────────────

    pub fn process_window_event(&mut self, event: &WindowEvent, _window: &Window) {
        match event {
            WindowEvent::KeyboardInput { event, .. } => {
                if let PhysicalKey::Code(kc) = event.physical_key {
                    let idx = key_index(&kc);
                    if idx < 256 {
                        let s = &mut self.keys[idx];
                        match event.state {
                            ElementState::Pressed => {
                                if !s.held {
                                    s.press_frame = self.frame;
                                }
                                s.held = true;
                            }
                            ElementState::Released => {
                                if s.held {
                                    s.release_frame = self.frame;
                                }
                                s.held = false;
                            }
                        }
                    }
                }
            }
            WindowEvent::MouseInput { button, state, .. } => {
                let m = mouse_from_winit(*button);
                let idx = mouse_index(&m);
                if idx < 8 {
                    let s = &mut self.mouse_buttons[idx];
                    match state {
                        ElementState::Pressed => {
                            if !s.held {
                                s.press_frame = self.frame;
                            }
                            s.held = true;
                        }
                        ElementState::Released => {
                            if s.held {
                                s.release_frame = self.frame;
                            }
                            s.held = false;
                        }
                    }
                }
            }
            WindowEvent::MouseWheel { delta, phase: TouchPhase::Moved, .. }
            | WindowEvent::MouseWheel { delta, phase: TouchPhase::Started, .. } => {
                match delta {
                    MouseScrollDelta::LineDelta(x, y) => {
                        self.mouse_scroll_line = Some((*x, *y));
                    }
                    MouseScrollDelta::PixelDelta(p) => {
                        self.mouse_scroll_pixel = Some((p.x, p.y));
                    }
                }
            }
            WindowEvent::Resized(size) => {
                self.resize_event = Some(Size2D::from(size.width, size.height));
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
            }
            WindowEvent::Focused(yes) => {
                self.focused = *yes;
            }
            WindowEvent::ModifiersChanged(mods) => {
                self.modifiers = mods.state();
            }
            _ => {}
        }
    }

    pub fn process_gilrs_event(&mut self, event: &gilrs::Event) {
        let idx: usize = event.id.into();
        if idx >= MAX_GAMEPADS { return; }

        match &event.event {
            gilrs::EventType::Connected => {
                self.gamepad_connected[idx] = true;
            }
            gilrs::EventType::Disconnected => {
                self.gamepad_connected[idx] = false;
                self.gamepad_buttons[idx] = empty_buttons::<20>();
                self.gamepad_axes[idx] = [0.0f32; 6];
            }
            gilrs::EventType::ButtonPressed(button, _) => {
                let gp = gamepad_button_from_gilrs(*button);
                let bi = gamepad_button_index(&gp);
                let pad_btns: &mut [ButtonState; 20] = &mut self.gamepad_buttons[idx];
                if bi < 20 {
                    if !pad_btns[bi].held {
                        pad_btns[bi].press_frame = self.frame;
                    }
                    pad_btns[bi].held = true;
                }
            }
            gilrs::EventType::ButtonRepeated(button, _) => {
                let gp = gamepad_button_from_gilrs(*button);
                let bi = gamepad_button_index(&gp);
                let pad_btns: &mut [ButtonState; 20] = &mut self.gamepad_buttons[idx];
                if bi < 20 {
                    pad_btns[bi].press_frame = self.frame;
                }
            }
            gilrs::EventType::ButtonReleased(button, _) => {
                let gp = gamepad_button_from_gilrs(*button);
                let bi = gamepad_button_index(&gp);
                let pad_btns: &mut [ButtonState; 20] = &mut self.gamepad_buttons[idx];
                if bi < 20 {
                    if pad_btns[bi].held {
                        pad_btns[bi].release_frame = self.frame;
                    }
                    pad_btns[bi].held = false;
                }
            }
            gilrs::EventType::AxisChanged(axis, val, _) => {
                let ga = gamepad_axis_from_gilrs(*axis);
                let ai = gamepad_axis_index(&ga);
                let axes: &mut [f32; 6] = &mut self.gamepad_axes[idx];
                if ai < 6 {
                    axes[ai] = *val;
                }
            }
            _ => {}
        }
    }

    /// Advances the frame counter and resets per-frame transient state.
    pub fn end_frame(&mut self) {
        self.frame += 1;
        self.mouse_scroll_line = None;
        self.mouse_scroll_pixel = None;
        self.resize_event = None;
    }

    // ── Keyboard queries ──────────────────────────────────────────────────

    pub fn key(&self, kc: KeyCode, action: Is) -> bool {
        let idx = key_index(&kc);
        if idx >= 256 { return false; }
        check_state(&self.keys[idx], action, self.frame)
    }

    pub fn key_combo(&self, primary: KeyCode, modifier: KeyCode, action: Is) -> bool {
        self.key(primary, action) && self.key(modifier, Is::Held)
    }

    pub fn key_combo_n(&self, keys: &[(KeyCode, Is)]) -> bool {
        keys.iter().all(|(kc, action)| self.key(*kc, *action))
    }

    pub fn any_key(&self, action: Is) -> bool {
        self.keys.iter().any(|s| check_state(s, action, self.frame))
    }

    // ── Mouse queries ─────────────────────────────────────────────────────

    pub fn mouse(&self, m: Mouse, action: Is) -> bool {
        let idx = mouse_index(&m);
        if idx >= 8 { return false; }
        check_state(&self.mouse_buttons[idx], action, self.frame)
    }

    pub fn any_mouse(&self, action: Is) -> bool {
        self.mouse_buttons.iter().any(|s| check_state(s, action, self.frame))
    }

    // ── Gamepad queries ───────────────────────────────────────────────────

    pub const GAMEPAD_AXIS_DEADZONE: f32 = 0.15;

    pub fn gamepad_connected(&self, id: usize) -> bool {
        if id >= MAX_GAMEPADS { return false; }
        self.gamepad_connected[id]
    }

    pub fn gamepad_count(&self) -> usize {
        self.gamepad_connected.iter().filter(|c| **c).count()
    }

    pub fn gamepad_button(&self, id: usize, button: GamepadButton, action: Is) -> bool {
        if id >= MAX_GAMEPADS { return false; }
        let idx = gamepad_button_index(&button);
        if idx >= 20 { return false; }
        check_state(&self.gamepad_buttons[id][idx], action, self.frame)
    }

    /// Check any button on a specific gamepad.
    pub fn any_gamepad_button(&self, id: usize, action: Is) -> bool {
        if id >= MAX_GAMEPADS { return false; }
        self.gamepad_buttons[id].iter().any(|s| check_state(s, action, self.frame))
    }

    /// Check any button on any connected gamepad.
    pub fn any_gamepad(&self, action: Is) -> bool {
        for id in 0..MAX_GAMEPADS {
            if self.gamepad_connected[id] && self.any_gamepad_button(id, action) {
                return true;
            }
        }
        false
    }

    pub fn gamepad_axis_raw(&self, id: usize, axis: GamepadAxis) -> f32 {
        if id >= MAX_GAMEPADS { return 0.0; }
        let idx = gamepad_axis_index(&axis);
        if idx >= 6 { return 0.0; }
        self.gamepad_axes[id][idx]
    }

    pub fn gamepad_axis(&self, id: usize, axis: GamepadAxis) -> f32 {
        let v = self.gamepad_axis_raw(id, axis);
        if v.abs() < Self::GAMEPAD_AXIS_DEADZONE { 0.0 } else { v }
    }

    pub fn gamepad_axis_deadzoned(&self, id: usize, axis: GamepadAxis, deadzone: f32) -> f32 {
        let v = self.gamepad_axis_raw(id, axis);
        if v.abs() < deadzone { 0.0 } else { v }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Index mapping ─────────────────────────────────────────────────

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

    // ── Frame-based lifecycle ────────────────────────────────────────

    #[test]
    fn press_release_lifecycle() {
        let mut ev = Events::new();
        assert_eq!(ev.frame, 1);
        let kc = KeyCode::KeyA;
        let idx = key_index(&kc);

        // Initially nothing
        assert!(!ev.key(kc, Is::Pressed));
        assert!(!ev.key(kc, Is::Held));
        assert!(!ev.key(kc, Is::Released));

        // Press on frame 1
        ev.keys[idx].held = true;
        ev.keys[idx].press_frame = ev.frame;
        assert!(ev.key(kc, Is::Pressed));
        assert!(ev.key(kc, Is::Held));
        assert!(!ev.key(kc, Is::Released));

        // Still held frame 2, no longer "pressed"
        ev.end_frame();
        assert_eq!(ev.frame, 2);
        assert!(!ev.key(kc, Is::Pressed));
        assert!(ev.key(kc, Is::Held));
        assert!(!ev.key(kc, Is::Released));

        // Release on frame 2
        ev.keys[idx].held = false;
        ev.keys[idx].release_frame = ev.frame;
        assert!(!ev.key(kc, Is::Pressed));
        assert!(!ev.key(kc, Is::Held));
        assert!(ev.key(kc, Is::Released));

        // Released flag clears after end_frame (frame 3)
        ev.end_frame();
        assert!(!ev.key(kc, Is::Released));
    }

    #[test]
    fn pressed_only_on_current_frame() {
        let mut ev = Events::new();
        assert_eq!(ev.frame, 1);
        let kc = KeyCode::KeyA;
        let idx = key_index(&kc);

        // Press on frame 1
        ev.keys[idx].held = true;
        ev.keys[idx].press_frame = ev.frame;
        assert!(ev.key(kc, Is::Pressed));

        // End frame — press_frame (1) != frame (2), so not pressed
        ev.end_frame();
        assert_eq!(ev.frame, 2);
        assert!(!ev.key(kc, Is::Pressed));
        assert!(ev.key(kc, Is::Held));

        // Press again on frame 2
        ev.keys[idx].press_frame = ev.frame;
        assert!(ev.key(kc, Is::Pressed));
    }

    #[test]
    fn key_unmapped() {
        let ev = Events::new();
        assert!(!ev.key(KeyCode::AudioVolumeUp, Is::Held));
    }

    #[test]
    fn mouse_unmapped() {
        let ev = Events::new();
        assert!(!ev.mouse(Mouse::Other(10), Is::Held));
    }

    #[test]
    fn key_combo_works() {
        let mut ev = Events::new();
        let ctrl = KeyCode::ControlLeft;
        let c = KeyCode::KeyC;

        assert!(!ev.key_combo(c, ctrl, Is::Pressed));

        ev.keys[key_index(&ctrl)].held = true;
        assert!(!ev.key_combo(c, ctrl, Is::Pressed));

        ev.keys[key_index(&c)].press_frame = ev.frame;
        ev.keys[key_index(&c)].held = true;
        assert!(ev.key_combo(c, ctrl, Is::Pressed));
    }

    #[test]
    fn key_combo_n_works() {
        let mut ev = Events::new();
        ev.keys[key_index(&KeyCode::ControlLeft)].held = true;
        ev.keys[key_index(&KeyCode::KeyA)].press_frame = ev.frame;
        ev.keys[key_index(&KeyCode::KeyA)].held = true;
        ev.keys[key_index(&KeyCode::KeyB)].held = true;

        assert!(ev.key_combo_n(&[
            (KeyCode::KeyA, Is::Pressed),
            (KeyCode::KeyB, Is::Held),
            (KeyCode::ControlLeft, Is::Held),
        ]));
    }

    #[test]
    fn initial_state() {
        let ev = Events::new();
        assert!(ev.close_requested == false);
        assert!(ev.focused);
        assert!(ev.resize_event.is_none());
        assert!(ev.mouse_scroll_line.is_none());
        assert!(ev.mouse_scroll_pixel.is_none());
        assert_eq!(ev.frame, 1);
        assert!(!ev.any_key(Is::Held));
        assert!(!ev.any_mouse(Is::Held));
        assert_eq!(ev.gamepad_count(), 0);
    }

    // ── Mouse ─────────────────────────────────────────────────────────

    #[test]
    fn mouse_press_release() {
        let mut ev = Events::new();
        assert!(!ev.mouse(Mouse::Left, Is::Pressed));

        ev.mouse_buttons[0].held = true;
        ev.mouse_buttons[0].press_frame = ev.frame;
        assert!(ev.mouse(Mouse::Left, Is::Pressed));
        assert!(ev.mouse(Mouse::Left, Is::Held));

        ev.mouse_buttons[0].held = false;
        ev.mouse_buttons[0].release_frame = ev.frame;
        assert!(ev.mouse(Mouse::Left, Is::Released));
    }

    #[test]
    fn any_mouse_works() {
        let mut ev = Events::new();
        assert!(!ev.any_mouse(Is::Pressed));

        ev.mouse_buttons[0].held = true;
        ev.mouse_buttons[0].press_frame = ev.frame;
        assert!(ev.any_mouse(Is::Pressed));
    }

    // ── Mouse wheel ───────────────────────────────────────────────────

    #[test]
    fn scroll_resets_on_end_frame() {
        let mut ev = Events::new();
        ev.mouse_scroll_line = Some((1.0, 2.0));
        ev.mouse_scroll_pixel = Some((10.0, 20.0));
        ev.end_frame();
        assert!(ev.mouse_scroll_line.is_none());
        assert!(ev.mouse_scroll_pixel.is_none());
    }

    // ── Modifiers ─────────────────────────────────────────────────────

    #[test]
    fn modifiers_default() {
        let ev = Events::new();
        assert!(!ev.modifiers.shift_key());
        assert!(!ev.modifiers.control_key());
        assert!(!ev.modifiers.alt_key());
        assert!(!ev.modifiers.super_key());
    }

    // ── Focus ─────────────────────────────────────────────────────────

    #[test]
    fn focus_starts_true() {
        let ev = Events::new();
        assert!(ev.focused);
    }

    // ── Gamepad ───────────────────────────────────────────────────────

    #[test]
    fn gamepad_button_indices() {
        assert_eq!(gamepad_button_index(&GamepadButton::A), 0);
        assert_eq!(gamepad_button_index(&GamepadButton::B), 1);
        assert_eq!(gamepad_button_index(&GamepadButton::X), 2);
        assert_eq!(gamepad_button_index(&GamepadButton::Y), 3);
        assert_eq!(gamepad_button_index(&GamepadButton::LB), 4);
        assert_eq!(gamepad_button_index(&GamepadButton::RB), 5);
        assert_eq!(gamepad_button_index(&GamepadButton::LT), 6);
        assert_eq!(gamepad_button_index(&GamepadButton::RT), 7);
        assert_eq!(gamepad_button_index(&GamepadButton::Back), 8);
        assert_eq!(gamepad_button_index(&GamepadButton::Start), 9);
        assert_eq!(gamepad_button_index(&GamepadButton::Guide), 10);
        assert_eq!(gamepad_button_index(&GamepadButton::LeftStick), 11);
        assert_eq!(gamepad_button_index(&GamepadButton::RightStick), 12);
        assert_eq!(gamepad_button_index(&GamepadButton::DPadUp), 13);
        assert_eq!(gamepad_button_index(&GamepadButton::DPadDown), 14);
        assert_eq!(gamepad_button_index(&GamepadButton::DPadLeft), 15);
        assert_eq!(gamepad_button_index(&GamepadButton::DPadRight), 16);
        assert_eq!(gamepad_button_index(&GamepadButton::Other(0)), 17);
        assert_eq!(gamepad_button_index(&GamepadButton::Other(2)), 19);
    }

    #[test]
    fn gamepad_axis_indices() {
        assert_eq!(gamepad_axis_index(&GamepadAxis::LeftX), 0);
        assert_eq!(gamepad_axis_index(&GamepadAxis::LeftY), 1);
        assert_eq!(gamepad_axis_index(&GamepadAxis::RightX), 2);
        assert_eq!(gamepad_axis_index(&GamepadAxis::RightY), 3);
        assert_eq!(gamepad_axis_index(&GamepadAxis::LeftTrigger), 4);
        assert_eq!(gamepad_axis_index(&GamepadAxis::RightTrigger), 5);
    }

    #[test]
    fn gamepad_press_release() {
        let mut ev = Events::new();
        assert!(!ev.gamepad_button(0, GamepadButton::A, Is::Pressed));

        ev.gamepad_buttons[0][0].held = true;
        ev.gamepad_buttons[0][0].press_frame = ev.frame;
        assert!(ev.gamepad_button(0, GamepadButton::A, Is::Pressed));
        assert!(ev.gamepad_button(0, GamepadButton::A, Is::Held));
        assert!(!ev.gamepad_button(0, GamepadButton::A, Is::Released));

        ev.end_frame();
        assert!(!ev.gamepad_button(0, GamepadButton::A, Is::Pressed));
        assert!(ev.gamepad_button(0, GamepadButton::A, Is::Held));

        ev.gamepad_buttons[0][0].held = false;
        ev.gamepad_buttons[0][0].release_frame = ev.frame;
        assert!(ev.gamepad_button(0, GamepadButton::A, Is::Released));
    }

    #[test]
    fn gamepad_out_of_range_id() {
        let ev = Events::new();
        assert!(!ev.gamepad_connected(99));
        assert!(!ev.gamepad_button(99, GamepadButton::A, Is::Held));
        assert_eq!(ev.gamepad_axis_raw(99, GamepadAxis::LeftX), 0.0);
    }

    #[test]
    fn gamepad_axis_deadzone() {
        let mut ev = Events::new();
        ev.gamepad_axes[0][0] = 0.05;
        assert_eq!(ev.gamepad_axis(0, GamepadAxis::LeftX), 0.0);

        ev.gamepad_axes[0][0] = 0.5;
        assert!((ev.gamepad_axis(0, GamepadAxis::LeftX) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn gamepad_axis_custom_deadzone() {
        let mut ev = Events::new();
        ev.gamepad_axes[0][0] = 0.05;
        assert_eq!(ev.gamepad_axis_deadzoned(0, GamepadAxis::LeftX, 0.1), 0.0);
        assert!((ev.gamepad_axis_deadzoned(0, GamepadAxis::LeftX, 0.01) - 0.05).abs() < 1e-6);
    }

    #[test]
    fn gamepad_initial_state() {
        let ev = Events::new();
        for g in 0..MAX_GAMEPADS {
            assert!(!ev.gamepad_connected[g]);
            for b in 0..20 {
                assert!(!ev.gamepad_buttons[g][b].held);
                assert_eq!(ev.gamepad_buttons[g][b].press_frame, 0);
                assert_eq!(ev.gamepad_buttons[g][b].release_frame, 0);
            }
            for a in 0..6 {
                assert_eq!(ev.gamepad_axes[g][a], 0.0);
            }
        }
        assert_eq!(ev.gamepad_count(), 0);
    }

    #[test]
    fn gamepad_connected_count() {
        let mut ev = Events::new();
        assert_eq!(ev.gamepad_count(), 0);
        ev.gamepad_connected[0] = true;
        assert_eq!(ev.gamepad_count(), 1);
        ev.gamepad_connected[2] = true;
        assert_eq!(ev.gamepad_count(), 2);
    }

    #[test]
    fn gamepad_any_button_specific() {
        let mut ev = Events::new();
        assert!(!ev.any_gamepad_button(0, Is::Pressed));

        ev.gamepad_buttons[0][3].held = true;
        ev.gamepad_buttons[0][3].press_frame = ev.frame;
        assert!(ev.any_gamepad_button(0, Is::Pressed));
        assert!(!ev.any_gamepad_button(1, Is::Pressed));
    }

    #[test]
    fn gamepad_any_connected() {
        let mut ev = Events::new();
        assert!(!ev.any_gamepad(Is::Pressed));

        ev.gamepad_connected[0] = true;
        ev.gamepad_buttons[0][5].held = true;
        ev.gamepad_buttons[0][5].press_frame = ev.frame;
        assert!(ev.any_gamepad(Is::Pressed));

        ev.end_frame();
        assert!(!ev.any_gamepad(Is::Pressed));
        assert!(ev.any_gamepad(Is::Held));
    }

    #[test]
    fn gamepad_any_skips_disconnected() {
        let mut ev = Events::new();
        // Set button on a disconnected gamepad — should not be detected
        ev.gamepad_buttons[0][0].held = true;
        ev.gamepad_buttons[0][0].press_frame = ev.frame;
        assert!(!ev.any_gamepad(Is::Pressed),
            "disconnected gamepad should not count");

        ev.gamepad_connected[0] = true;
        assert!(ev.any_gamepad(Is::Pressed),
            "connecting makes it visible");
    }

    #[test]
    fn gamepad_end_frame_resets_press() {
        let mut ev = Events::new();
        ev.gamepad_buttons[0][0].held = true;
        ev.gamepad_buttons[0][0].press_frame = ev.frame;
        assert!(ev.gamepad_button(0, GamepadButton::A, Is::Pressed));

        ev.end_frame();
        assert!(!ev.gamepad_button(0, GamepadButton::A, Is::Pressed));
        assert!(ev.gamepad_button(0, GamepadButton::A, Is::Held));
    }

    #[test]
    fn gamepad_axis_named_constants() {
        match GamepadAxis::LeftX { _ => {} }
        match GamepadAxis::LeftY { _ => {} }
        match GamepadAxis::RightX { _ => {} }
        match GamepadAxis::RightY { _ => {} }
        match GamepadAxis::LeftTrigger { _ => {} }
        match GamepadAxis::RightTrigger { _ => {} }
    }

    // ── end_frame ─────────────────────────────────────────────────────

    #[test]
    fn end_frame_increments_counter() {
        let mut ev = Events::new();
        assert_eq!(ev.frame, 1);
        ev.end_frame();
        assert_eq!(ev.frame, 2);
        ev.end_frame();
        assert_eq!(ev.frame, 3);
    }

    #[test]
    fn end_frame_clears_resize_event() {
        let mut ev = Events::new();
        ev.resize_event = Some(Size2D::from(800, 600));
        ev.end_frame();
        assert!(ev.resize_event.is_none());
    }

    #[test]
    fn close_requested_sticks() {
        let mut ev = Events::new();
        ev.close_requested = true;
        ev.end_frame();
        assert!(ev.close_requested, "close_requested persists after end_frame");
        ev.close_requested = false;
        assert!(!ev.close_requested, "user clears it manually");
    }

    #[test]
    fn clear_resets_all() {
        let mut ev = Events::new();
        let kc = KeyCode::KeyW;
        let idx = key_index(&kc);
        ev.keys[idx].held = true;
        ev.keys[idx].press_frame = 0;
        ev.mouse_scroll_line = Some((1.0, 0.0));
        ev.close_requested = true;

        ev.clear();
        assert!(ev.close_requested == false);
        assert!(!ev.keys[idx].held);
        assert!(ev.mouse_scroll_line.is_none());
    }

    // ── any_key / any_mouse ───────────────────────────────────────────

    #[test]
    fn any_key_detects_press() {
        let mut ev = Events::new();
        assert!(!ev.any_key(Is::Pressed));

        ev.keys[key_index(&KeyCode::Space)].held = true;
        ev.keys[key_index(&KeyCode::Space)].press_frame = ev.frame;
        assert!(ev.any_key(Is::Pressed));
        assert!(ev.any_key(Is::Held));
    }

    #[test]
    fn any_key_returns_false_when_none_held() {
        let ev = Events::new();
        assert!(!ev.any_key(Is::Held));
        assert!(!ev.any_key(Is::Pressed));
        // Released on frame 0 is never possible since frame starts at 1
        assert!(!ev.any_key(Is::Released));
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
