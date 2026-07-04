//! Perspective and orthographic cameras with fly-through controls.
//!
//! The [`Camera`] wraps a [`CamTransform`] and provides high-level movement
//! and rotation methods suitable for first-person and orbital cameras.

mod camera;
pub use camera::*;
