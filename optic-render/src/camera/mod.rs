//! Perspective and orthographic cameras with fly-through controls.
//!
//! The camera system provides high-level wrappers around view and projection
//! matrix computation for both 3D perspective and 2D orthographic rendering.
//!
//! # Key types
//!
//! | Type | Role |
//! |------|------|
//! | [`Camera`] | Bundles a [`CamTransform`] with projection parameters (FOV, near/far planes). |
//! | [`CamTransform`] | Position + rotation (yaw / pitch) with fly-through movement helpers. |
//!
//! # Typical usage
//!
//! ```ignore
//! use optic_render::Camera;
//!
//! let mut cam = Camera::perspective(800.0 / 600.0, 45.0, 0.1, 100.0);
//! cam.transform.forward(1.0);   // move forward
//! cam.transform.yaw(0.5);       // rotate
//!
//! gpu.render3d(&mesh, &cam);
//! ```

mod camera;
pub use camera::*;
