//! # Optic Engine
//!
//! A modular Rust game engine composed of independently-compilable sub-crates.
//!
//! This is the top-level **facade crate**. It re-exports all public items from
//! the engine's sub-crates, gated behind Cargo feature flags. Add `optic` to
//! your `Cargo.toml` and enable the features you need:
//!
//! ```toml
//! [dependencies]
//! optic = { git = "...", features = ["core", "render", "window", "minimal"] }
//! ```
//!
//! # Feature flags
//!
//! | Feature | Re-exports | Description |
//! |---|---|---|
//! | `core` | [`optic_core`] | Core types, math, enums, colour, logging |
//! | `file` | [`optic_file`] | File I/O and asset path utilities |
//! | `render` | [`optic_render`] | OpenGL rendering pipeline |
//! | `window` | [`optic_window`] | Windowing, input, events |
//! | `minimal` | [`optic_loop`] | Game loop and runtime |
//! | `online` | [`optic_online`] | Networking |
//!
//! # Architecture
//!
//! The engine is split into focused crates so you can depend on only what you
//! need. The `optic` crate itself is a zero-cost re-export layer — add it for
//! convenience, or depend on individual sub-crates for finer control.
//!
//! ```text
//!             ┌──────────┐
//!             │  optic   │  facade (re-exports)
//!             ├──────────┤
//!             │  optic_core   │  math, colour, enums, logging
//!             │  optic_file   │  asset I/O
//!             │  optic_render │  GPU / GL rendering
//!             │  optic_window │  winit, input
//!             │  optic_loop   │  game loop
//!             │  optic_online │  networking
//!             └──────────┘
//! ```

#[cfg(feature = "core")]
pub use optic_core::*;
#[cfg(feature = "file")]
pub use optic_file::*;
#[cfg(feature = "render")]
pub use optic_render::*;
#[cfg(feature = "window")]
pub use optic_window::*;
#[cfg(feature = "minimal")]
pub use optic_loop::*;
#[cfg(feature = "online")]
pub use optic_online::*;
