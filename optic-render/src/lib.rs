//! GPU-accelerated rendering with EGL/OpenGL 4.6.
//!
//! `optic-render` manages the full graphics pipeline: context creation (headless or
//! windowed), asset loading (meshes, textures, shaders), instanced drawing, off-screen
//! canvas (framebuffer objects), and 2D/3D camera transforms.
//!
//! # Architecture
//!
//! | Layer | Module | Role |
//! |-------|--------|------|
//! | Context | [`RenderContext`] | EGL display, surfaces, vsync |
//! | Backend | [`GL`] | Thin wrappers around raw OpenGL calls |
//! | Device | [`GPU`] | Stateful renderer with fallback assets |
//! | Assets | [`asset`] | Load/save/cache meshes, textures, shaders from disk |
//! | Handles | [`handles`] | Runtime GPU handles: [`MeshHandle`], [`Shader`], [`Texture2D`], [`Canvas`], [`InstanceBuffer`] ... |
//! | Camera | [`Camera`] | Perspective/orthographic camera with fly-through controls |
//! | Transforms | [`Transform2D`], [`Transform3D`], [`CamTransform`] | Position / rotation / scale helpers |
//!
//! # Getting started
//!
//! ```ignore
//! use optic_render::GPU;
//!
//! let gpu = GPU::new_headless()?;
//! println!("{}", gpu.version());
//! ```
//!
//! # Feature flags
//!
//! This crate is always compiled with all features. The parent `optic` crate controls
//! which sub-crates are included via its own feature flags.

mod camera;
mod context;
mod glraw;
pub mod handles;
mod renderer;
mod util;

pub mod asset;

pub use camera::*;
pub use context::*;
pub use glraw::*;
pub use handles::*;
pub use renderer::*;
pub use util::*;
