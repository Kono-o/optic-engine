//! Asset loading, caching, and GPU upload.
//!
//! Each asset type provides a `from_disk(path)` constructor that in debug
//! builds loads the source file and overwrites the binary cache, and in
//! release builds loads the cache for faster startup.
//!
//! # Types
//!
//! | Type | File extension | Cache extension |
//! |------|----------------|-----------------|
//! | [`TextureFile`] | `.png`, `.jpg`, ... | `.otxtr` |
//! | [`Mesh3DFile`] | `.obj`, `.stl` | `.omesh` |
//! | [`ShaderFile`] | `.glsl` | `.oshdr` |
//!
//! Also provides the [`attr`] sub-module for vertex and instance attribute
//! descriptors used by meshes and instance buffers.

pub mod attr;
pub mod font;
mod img;
mod msh;
mod msdf;
mod shdr;

pub use font::*;
pub use img::*;
pub use msh::*;
pub use msdf::*;
pub use shdr::*;
