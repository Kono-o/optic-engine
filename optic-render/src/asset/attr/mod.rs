//! Vertex and instance attribute types.
//!
//! This module defines the [`DataType`] trait (implemented for all primitive
//! numeric types and their fixed-size arrays), the [`ATTRInfo`] / [`ATTRName`]
//! descriptors, and a set of concrete attribute containers:
//!
//! * [`Pos3DATTR`], [`Pos2DATTR`] — position
//! * [`ColATTR`] — colour (RGBA)
//! * [`UVMATTR`] — UV / texture coordinates
//! * [`NrmATTR`] — normal vectors
//! * [`IndATTR`] — index buffer
//! * [`Rot3DATTR`], [`Rot2DATTR`] — rotation (quaternion / angle)
//! * [`Scale3DATTR`], [`Scale2DATTR`] — scale
//! * [`CustomATTR`] — user-defined arbitrary attribute data

mod attr;
mod typ;

pub use attr::*;
pub use typ::*;
