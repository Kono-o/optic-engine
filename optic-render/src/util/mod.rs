//! Utility types for rendering.
//!
//! Currently provides transformation helpers ([`Transform2D`], [`Transform3D`],
//! [`CamTransform`]) and re-exports [`RGBA`] / [`RGB`].

pub mod transform;
pub use transform::*;

pub use optic_core::{RGBA, RGB};
