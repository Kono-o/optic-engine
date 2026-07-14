//! Rendering utility types — transforms, colour helpers, and common re-exports.
//!
//! This module re-exports the transform primitives used throughout
//! `optic-render` as well as colour types from `optic-core`:
//!
//! | Type | Source | Purpose |
//! |------|--------|---------|
//! | [`Transform2D`] | `util::transform` | 2-D position, rotation, and scale |
//! | [`Transform3D`] | `util::transform` | 3-D position, rotation (Euler), and scale |
//! | [`CamTransform`] | `util::transform` | First-person camera position + yaw/pitch |
//! | [`RGBA`], [`RGB`] | `optic_core` | sRGBA / sRGB colour types |

pub mod transform;
pub use transform::*;

pub use optic_core::{RGBA, RGB};
