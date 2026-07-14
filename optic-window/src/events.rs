//! Input handling and event processing.
//!
//! This module provides [`Events`], a frame-based input snapshot that collects
//! keyboard, mouse, gamepad, window, and custom signal events. Feed it raw
//! [`winit::event::WindowEvent`]s and [`gilrs::Event`]s each frame, then query
//! the snapshot with frame-aware predicates ([`Is::Pressed`], [`Is::Released`],
//! [`Is::Held`]).
//!
//! The frame counter advances when [`Events::end_frame`] is called at the end of
//! each frame, which also resets transient state (scroll deltas, resize events,
//! signals).

use optic_core::{NetworkEvents, Size2D};
use std::collections::HashMap;

use crate::window::Window;
use gilrs;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase, WindowEvent};
use winit::keyboard::{ModifiersState, PhysicalKey};
pub use winit::keyboard::KeyCode;

/// Per-button state machine.
///
/// Tracks press/release frame numbers so callers can distinguish the exact
/// frame a button was pressed or released, independent of polling order.
///
/// Used internally by [`Events`] for keyboard, mouse, and gamepad buttons.
#[derive(Copy, Clone)]
pub struct ButtonState {
    pub held: bool,
    pub press_frame: u64,
    pub release_frame: u64,
}

/// Input action filter for frame-based queries.
///
/// Passed to query methods on [`Events`] to specify which edge or level
/// of the button state should match.
///
/// | Variant | Behaviour |
/// |---------|-----------|
/// | `Pressed`  | True only on the frame the button transitions up→down |
/// | `Released` | True only on the frame the button transitions down→up |
/// | `Held`     | True every frame while the button is down |
#[derive(Debug, Clone, Copy)]
pub enum Is {
    Pressed,
    Released,
    Held,
}

/// Mouse button identifier.
///
/// Used with [`Events::mouse`] to query button state. The standard five
/// buttons (left, right, middle, back, forward) are named variants; any
/// additional buttons are represented by [`Mouse::Other`] with the
/// platform-specific button index.
///
/// # Examples
///
/// ```ignore
/// if events.mouse(Mouse::Left, Is::Pressed) { /* click */ }
/// if events.mouse(Mouse::Other(5), Is::Held) { /* extra side button */ }
/// ```
#[derive(Debug, Clone, Copy)]
pub enum Mouse {
    /// Primary (left) mouse button.
    Left,
    /// Secondary (right) mouse button.
    Right,
    /// Middle / tertiary mouse button.
    Middle,
    /// Back side button (thumb).
    Back,
    /// Forward side button (thumb).
    Forward,
    /// Additional mouse button identified by a platform-specific index.
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

/// Maximum number of supported gamepads.
pub const MAX_GAMEPADS: usize = 4;

/// Gamepad button identifier.
///
/// Follows the Xbox / Standard Gamepad layout. Used with
/// [`Events::gamepad_button`] to query per-gamepad button state.
/// Unknown or unmapped buttons are captured as [`GamepadButton::Other`].
///
/// # Examples
///
/// ```ignore
/// if events.gamepad_button(0, GamepadButton::A, Is::Pressed) { /* jump */ }
/// if events.gamepad_button(0, GamepadButton::LB, Is::Held) { /* aim */ }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadButton {
    /// Face south (A on Xbox, × on PlayStation).
    A,
    /// Face east (B on Xbox, ○ on PlayStation).
    B,
    /// Face west (X on Xbox, □ on PlayStation).
    X,
    /// Face north (Y on Xbox, △ on PlayStation).
    Y,
    /// Left bumper / shoulder.
    LB,
    /// Right bumper / shoulder.
    RB,
    /// Left trigger (analog digital press).
    LT,
    /// Right trigger (analog digital press).
    RT,
    /// Back / Select / View button.
    Back,
    /// Start / Menu button.
    Start,
    /// Guide / Home / PS button.
    Guide,
    /// Left stick click.
    LeftStick,
    /// Right stick click.
    RightStick,
    /// D-pad up.
    DPadUp,
    /// D-pad down.
    DPadDown,
    /// D-pad left.
    DPadLeft,
    /// D-pad right.
    DPadRight,
    /// Unrecognized or vendor-specific button.
    Other(u8),
}

/// Gamepad analog axis identifier.
///
/// Each axis reports a continuous `f32` value in the range `-1.0..=1.0`
/// (sticks) or `0.0..=1.0` (triggers). Query with
/// [`Events::gamepad_axis`] which applies a default deadzone, or
/// [`Events::gamepad_axis_raw`] / [`Events::gamepad_axis_deadzoned`] for
/// custom thresholds.
///
/// # Examples
///
/// ```ignore
/// let horizontal = events.gamepad_axis(0, GamepadAxis::LeftX);
/// if horizontal > 0.5 { /* move right */ }
/// let trigger = events.gamepad_axis(0, GamepadAxis::RightTrigger);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadAxis {
    /// Left stick horizontal (left = -1.0, right = 1.0).
    LeftX,
    /// Left stick vertical (up = -1.0, down = 1.0, winit convention).
    LeftY,
    /// Right stick horizontal (left = -1.0, right = 1.0).
    RightX,
    /// Right stick vertical (up = -1.0, down = 1.0, winit convention).
    RightY,
    /// Left analog trigger (0.0 = released, 1.0 = fully pressed).
    LeftTrigger,
    /// Right analog trigger (0.0 = released, 1.0 = fully pressed).
    RightTrigger,
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

/// A signal payload — either empty or containing raw byte data.
///
/// Use [`emit`](Events::emit) for empty signals and
/// [`emit_with`](Events::emit_with) to attach a payload.
#[derive(Debug, Clone)]
pub enum SignalPayload {
    /// Signal emitted with no data.
    None,
    /// Signal emitted with raw byte payload.
    Bytes(Vec<u8>),
}

/// Per-frame input state snapshot.
///
/// `Events` holds all input state for a single frame. It is populated by calling
/// [`process_window_event`](Events::process_window_event) and
/// [`process_gilrs_event`](Events::process_gilrs_event) as events arrive, then
/// frozen for the frame. At the end of the frame, [`end_frame`](Events::end_frame)
/// advances the frame counter and clears transient state.
///
/// # Keyboard
///
/// ```
/// use optic_window::*;
///
/// # let events = Events::new();
/// if events.key(KeyCode::Space, Is::Pressed) {
///     // space was just pressed this frame
/// }
/// if events.key(KeyCode::ControlLeft, Is::Held) {
///     // ctrl is held down
/// }
/// ```
///
/// # Mouse
///
/// ```
/// use optic_window::*;
///
/// # let mut events = Events::new();
/// if events.mouse(Mouse::Left, Is::Pressed) {
///     // left click this frame
/// }
/// ```
///
/// # Gamepad
///
/// ```
/// use optic_window::*;
///
/// # let mut events = Events::new();
/// if events.gamepad_button(0, GamepadButton::A, Is::Pressed) {
///     // gamepad 0 pressed A this frame
/// }
/// let axis = events.gamepad_axis(0, GamepadAxis::LeftX);
/// ```
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
    pub network: NetworkEvents,
    pub signals: HashMap<String, Vec<SignalPayload>>,
}

fn empty_buttons<const N: usize>() -> [ButtonState; N] {
    [ButtonState { held: false, press_frame: 0, release_frame: 0 }; N]
}

impl Events {
    /// Create a new, empty event state (frame = 1, focused = true).
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
            network: NetworkEvents::default(),
            signals: HashMap::new(),
        }
    }

    /// Reset all state to defaults. Does not advance the frame counter.
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
        self.network = NetworkEvents::default();
        self.signals = HashMap::new();
    }

    // ── Event processing ──────────────────────────────────────────────────

    /// Process a single winit [`WindowEvent`], updating internal state.
    ///
    /// Called by the game loop for each event in the winit event queue.
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
                self.resize_event = Some(Size2D::new(size.width, size.height));
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

    /// Process a single gilrs gamepad event.
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

    /// Advance the frame counter and clear per-frame transient state.
    ///
    /// Must be called at the end of every frame. Resets scroll deltas,
    /// resize events, and network events. The `close_requested` flag
    /// persists across frames (the user must manually clear it).
    pub fn end_frame(&mut self) {
        self.frame += 1;
        self.mouse_scroll_line = None;
        self.mouse_scroll_pixel = None;
        self.resize_event = None;
        self.network.packets.clear();
        self.network.peers_connected.clear();
        self.network.peers_disconnected.clear();
        self.signals.clear();
    }

    // ── Keyboard queries ──────────────────────────────────────────────────

    /// Query a single key by [`KeyCode`] and [`Is`] action.
    pub fn key(&self, kc: KeyCode, action: Is) -> bool {
        let idx = key_index(&kc);
        if idx >= 256 { return false; }
        check_state(&self.keys[idx], action, self.frame)
    }

    /// Query a key combo: `primary` must match `action` while `modifier` is held.
    ///
    /// ```
    /// use optic_window::*;
    ///
    /// # let events = Events::new();
    /// if events.key_combo(KeyCode::KeyC, KeyCode::ControlLeft, Is::Pressed) {
    ///     // Ctrl+C was pressed
    /// }
    /// ```
    pub fn key_combo(&self, primary: KeyCode, modifier: KeyCode, action: Is) -> bool {
        self.key(primary, action) && self.key(modifier, Is::Held)
    }

    /// Query multiple keys simultaneously — all must match their respective actions.
    pub fn key_combo_n(&self, keys: &[(KeyCode, Is)]) -> bool {
        keys.iter().all(|(kc, action)| self.key(*kc, *action))
    }

    /// True if any key matches the given action.
    pub fn any_key(&self, action: Is) -> bool {
        self.keys.iter().any(|s| check_state(s, action, self.frame))
    }

    // ── Mouse queries ─────────────────────────────────────────────────────

    /// Query a mouse button by [`Mouse`] and [`Is`] action.
    pub fn mouse(&self, m: Mouse, action: Is) -> bool {
        let idx = mouse_index(&m);
        if idx >= 8 { return false; }
        check_state(&self.mouse_buttons[idx], action, self.frame)
    }

    /// True if any mouse button matches the given action.
    pub fn any_mouse(&self, action: Is) -> bool {
        self.mouse_buttons.iter().any(|s| check_state(s, action, self.frame))
    }

    // ── Gamepad queries ───────────────────────────────────────────────────

    /// Default deadzone for gamepad analog axes.
    pub const GAMEPAD_AXIS_DEADZONE: f32 = 0.15;

    /// True if a gamepad with the given `id` is currently connected.
    pub fn gamepad_connected(&self, id: usize) -> bool {
        if id >= MAX_GAMEPADS { return false; }
        self.gamepad_connected[id]
    }

    /// Number of currently connected gamepads.
    pub fn gamepad_count(&self) -> usize {
        self.gamepad_connected.iter().filter(|c| **c).count()
    }

    /// Query a gamepad button by [`GamepadButton`] and [`Is`] action.
    pub fn gamepad_button(&self, id: usize, button: GamepadButton, action: Is) -> bool {
        if id >= MAX_GAMEPADS { return false; }
        let idx = gamepad_button_index(&button);
        if idx >= 20 { return false; }
        check_state(&self.gamepad_buttons[id][idx], action, self.frame)
    }

    /// True if any button on the given gamepad matches the action.
    pub fn any_gamepad_button(&self, id: usize, action: Is) -> bool {
        if id >= MAX_GAMEPADS { return false; }
        self.gamepad_buttons[id].iter().any(|s| check_state(s, action, self.frame))
    }

    /// True if any button on any connected gamepad matches the action.
    pub fn any_gamepad(&self, action: Is) -> bool {
        for id in 0..MAX_GAMEPADS {
            if self.gamepad_connected[id] && self.any_gamepad_button(id, action) {
                return true;
            }
        }
        false
    }

    /// Raw gamepad axis value (no deadzone applied).
    pub fn gamepad_axis_raw(&self, id: usize, axis: GamepadAxis) -> f32 {
        if id >= MAX_GAMEPADS { return 0.0; }
        let idx = gamepad_axis_index(&axis);
        if idx >= 6 { return 0.0; }
        self.gamepad_axes[id][idx]
    }

    /// Gamepad axis value with the default deadzone (`GAMEPAD_AXIS_DEADZONE`).
    ///
    /// Values below the deadzone are snapped to 0.0.
    pub fn gamepad_axis(&self, id: usize, axis: GamepadAxis) -> f32 {
        let v = self.gamepad_axis_raw(id, axis);
        if v.abs() < Self::GAMEPAD_AXIS_DEADZONE { 0.0 } else { v }
    }

    /// Gamepad axis value with a custom deadzone.
    pub fn gamepad_axis_deadzoned(&self, id: usize, axis: GamepadAxis, deadzone: f32) -> f32 {
        let v = self.gamepad_axis_raw(id, axis);
        if v.abs() < deadzone { 0.0 } else { v }
    }

    // ── Custom signals ────────────────────────────────────────────────────

    /// Emit a named signal (no payload).
    ///
    /// The signal is available for the remainder of the frame and cleared
    /// at [`end_frame`](Self::end_frame).
    ///
    /// ```ignore
    /// game.events.emit("boss_defeated");
    /// ```
    pub fn emit(&mut self, name: &str) {
        self.signals
            .entry(name.to_string())
            .or_default()
            .push(SignalPayload::None);
    }

    /// Emit a named signal with a byte payload.
    ///
    /// ```ignore
    /// game.events.emit_with("score", 100u32.to_ne_bytes().to_vec());
    /// ```
    pub fn emit_with(&mut self, name: &str, payload: Vec<u8>) {
        self.signals
            .entry(name.to_string())
            .or_default()
            .push(SignalPayload::Bytes(payload));
    }

    /// Returns `true` if the named signal was emitted this frame.
    pub fn was_emitted(&self, name: &str) -> bool {
        self.signals.get(name).map_or(false, |v| !v.is_empty())
    }

    /// Returns how many times the named signal was emitted this frame.
    pub fn emitted_count(&self, name: &str) -> u32 {
        self.signals.get(name).map_or(0, |v| v.len() as u32)
    }

    /// Returns the first payload for the named signal, if any.
    ///
    /// Returns `None` if the signal wasn't emitted or carries no payload.
    pub fn payload(&self, name: &str) -> Option<&SignalPayload> {
        self.signals.get(name)?.first()
    }

    /// Returns all payloads for the named signal.
    ///
    /// Returns an empty slice if the signal wasn't emitted.
    pub fn payloads(&self, name: &str) -> &[SignalPayload] {
        self.signals.get(name).map_or(&[], |v| v.as_slice())
    }
}
