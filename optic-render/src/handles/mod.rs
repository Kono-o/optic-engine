//! Runtime GPU handles for rendering.
//!
//! These types wrap OpenGL objects (textures, shaders, meshes, framebuffers,
//! instance buffers) and provide safe Rust APIs for creating, updating, and
//! destroying them.
//!
//! # Modules
//!
//! | Module | Key types |
//! |--------|-----------|
//! | [`texture`] | [`Texture2D`], [`create_texture`], [`delete_texture`] |
//! | [`shader`] | [`Shader`], [`Slot`], [`Workers`], [`compile_shader`], [`link_program`], [`link_compute_program`] |
//! | [`mesh`] | [`MeshHandle`], [`Mesh3D`], [`Mesh2D`], [`StorageBuffer`] |
//! | [`canvas`] | [`Canvas`], [`CanvasDesc`], [`RenderTarget`] |
//! | [`instance`] | [`InstanceBuffer`], [`InstanceDesc3D`], [`InstanceDesc2D`] |

pub mod canvas;
pub mod instance;
pub mod mesh;
pub mod shader;
pub mod texture;

pub use canvas::*;
pub use instance::*;
pub use mesh::*;
pub use shader::*;
pub use texture::*;
