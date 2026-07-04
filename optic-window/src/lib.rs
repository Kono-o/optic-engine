//! Windowing and input subsystem for the Optic engine.
//!
//! Wraps [`winit`] (window creation, event loop) and [`gilrs`] (gamepad support)
//! into a frame-based input model. Input state is captured once per frame as a
//! snapshot, avoiding callback-driven event handling in game code.
//!
//! # Frame-based input
//!
//! Every input method accepts an [`Is`] action to query:
//!
//! | Action | Meaning |
//! |--------|---------|
//! | [`Is::Pressed`] | True only on the exact frame the button went down |
//! | [`Is::Released`] | True only on the exact frame the button came up |
//! | [`Is::Held`] | True every frame while the button is held |
//!
//! This is the same pattern used by frame-based state machines: `Pressed`
//! is the leading edge, `Released` is the trailing edge, `Held` is the level.
//!
//! # Architecture
//!
//! The [`Window`] owns the winit window handle. The [`Events`] struct holds
//! per-frame input state (keys, mouse, gamepad). The game loop (`optic_loop`)
//! drives both: it processes winit events into `Events` via
//! [`process_window_event`](Events::process_window_event) and calls
//! [`end_frame`](Events::end_frame) at the end of each frame.

mod events;
mod screen;
mod window;

pub use events::*;
pub use screen::*;
pub use window::*;

pub use winit;
pub use gilrs;
