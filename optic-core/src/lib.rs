//! Shared types, geometry, errors, constants, and logging for the Optic engine.
//!
//! This crate is the foundation of the engine. Every other crate depends on it.
//! It re-exports [`optic_color`] and [`cgmath`], so downstream crates get math
//! and color types through a single dependency.
//!
//! # Organization
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`geometry`] | [`Size2D`], [`Size3D`], [`ClipDist`], [`CamProj`], [`Components`] trait |
//! | [`coord`] | [`Coord2D`] (point), [`CoordOffset`] (vector/displacement) |
//! | [`enums`] | [`PolyMode`], [`Cull`], [`DrawMode`], [`ImgFormat`], [`ImgFilter`], [`ImgWrap`], [`ATTRType`] |
//! | [`error`] | [`OpticError`], [`OpticErrorKind`], [`OpticResult`] |
//! | [`ansi`] | [`ansi::ANSI`] color codes for terminal output |
//! | [`consts`] | Asset paths, cache magic, version constants |
//! | [`network`] | [`PeerId`], [`NetworkMode`], [`NetworkConfig`], [`NetworkEvents`] |
//! | [`proc`] | [`end`], [`end_success`], [`end_error`] process helpers |
//!
//! # Logging macros
//!
//! The crate provides color-coded logging via macro:
//!
//! ```
//! use optic_core::*;
//!
//! log_info!("hello world");
//! log_warn!("value is {}", 42);
//! log_error!("something broke");
//! log_color!("custom format", RED, "arg {}", 1);
//! ```

pub mod ansi;
pub mod color;
pub mod consts;
pub mod coord;
pub mod enums;
pub mod error;
pub mod geometry;
mod log;
pub mod network;
pub mod proc;

pub use color::*;
pub use coord::*;
pub use enums::*;
pub use error::*;
pub use geometry::*;
pub use network::*;
pub use proc::*;

pub use cgmath;
