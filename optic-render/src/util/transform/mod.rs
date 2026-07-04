//! 2D, 3D, and camera transforms.
//!
//! Each transform stores position, rotation, and scale, and can produce a
//! 4×4 transformation matrix via [`calc_matrix`](Transform3D::calc_matrix) /
//! [`calc_matrices`](CamTransform::calc_matrices).

mod trans2d;
mod trans3d;
mod transcam;

pub use trans2d::*;
pub use trans3d::*;
pub use transcam::*;
