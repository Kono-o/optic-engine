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
//!
//! # How the Rendering Pipeline Works
//!
//! The Optic rendering pipeline is a **forward renderer** built on OpenGL 4.6. Every
//! frame, the engine clears the screen, iterates over meshes, and issues draw calls
//! that go through a shader-programmed vertex-to-fragment pipeline.
//!
//! ## Pipeline Layers
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │  User code (Runtime::render)                                │
//! │    │                                                        │
//! │    ▼                                                        │
//! │  GPU (stateful renderer)                                    │
//! │    ├─ clear()          → glClear                            │
//! │    ├─ render3d(mesh)   → MVP uniforms + draw call           │
//! │    ├─ render2d(mesh)   → ortho projection + draw call       │
//! │    └─ render_text2d()  → MSDF text instanced draw           │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  Mesh3D / Mesh2D (high-level handles)                       │
//! │    ├─ visibility check                                      │
//! │    ├─ shader.bind()     → glUseProgram                      │
//! │    ├─ set uniforms      → uView, uProj, uTfm               │
//! │    ├─ bind textures    → glActiveTexture + glBindTexture    │
//! │    ├─ bind storages    → glBindBufferBase (SSBO)            │
//! │    └─ handle.draw_as()  → glDrawElements / glDrawArrays     │
//! │         │                                                   │
//! │         ▼                                                   │
//! │  MeshHandle (low-level GPU buffers)                         │
//! │    ├─ bind VAO          → glBindVertexArray                 │
//! │    ├─ bind EBO          → glBindBuffer(ELEMENT_ARRAY)       │
//! │    └─ draw call         → glDrawElements / glDrawArrays     │
//! │         │              or *Instanced variants               │
//! │         ▼                                                   │
//! │  GL (raw OpenGL wrappers)                                   │
//! │    └─ thin wrappers around gl::* calls                      │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Draw Call Lifecycle
//!
//! When you call `gpu.render3d(&mesh, &camera)`, the following happens:
//!
//! 1. **Visibility check** — if the mesh is hidden or empty, the call returns
//!    immediately.
//!
//! 2. **Shader bind** — the mesh's shader program is activated via
//!    `glUseProgram`. If the mesh has no shader, the draw call is skipped.
//!
//! 3. **Uniform upload** — the engine sets three core uniforms:
//!    - `uView` — the camera's view matrix (`Matrix4<f32>`)
//!    - `uProj` — the camera's projection matrix (perspective or orthographic)
//!    - `uTfm` — the mesh's model transform matrix (computed from
//!      position/rotation/scale)
//!
//!    For 2D meshes, `uLayer` (a `u32`) is also set for z-ordering.
//!
//! 4. **Texture bind** — all textures attached to the shader are bound to their
//!    respective slots (S0–S15). Pipeline shaders use `glActiveTexture` +
//!    `glBindTexture(TEXTURE_2D)`. Compute shaders use `glBindImageTexture`.
//!
//!    SSBO binding (for compute or vertex-pulling) uses
//!    `glBindBufferBase(SHADER_STORAGE_BUFFER, slot, id)`.
//!
//! 5. **Draw call** — `MeshHandle::draw_as(mode)` issues the OpenGL draw call:
//!    - Binds the VAO via `glBindVertexArray`
//!    - Binds the IBO (if indexed) via `glBindBuffer(ELEMENT_ARRAY_BUFFER)`
//!    - Calls `glDrawElements`, `glDrawArrays`, or their `*Instanced` variants
//!
//! ## Mesh Buffer Lifecycle (VAO/VBO/IBO)
//!
//! When a mesh is uploaded from an asset type to the GPU:
//!
//! 1. **Create VAO** — `glGenVertexArrays` produces a vertex array object
//! 2. **Create VBO** — `glGenBuffers` produces a vertex buffer object
//! 3. **Upload data** — `glBufferData` fills the VBO with interleaved vertex
//!    data (positions, normals, UVs, colours, indices)
//! 4. **Configure attributes** — `glVertexAttribPointer` + `glEnableVertexAttribArray`
//!    for each attribute in the layout (position at location 0, colour at 1, etc.)
//! 5. **Create IBO** (if indexed) — `glGenBuffers` + `glBufferData` for the
//!    element array buffer
//!
//! ```text
//! Mesh3DFile (CPU)          GPU Buffers
//! ┌──────────────┐         ┌──────────────────────────────┐
//! │ positions[]  │ ──────▶ │ VBO (interleaved)            │
//! │ normals[]    │         │  [pos|nrm|uv|col|...] × N    │
//! │ uvs[]        │         │                              │
//! │ colours[]    │         │  VAO references VBO + layout │
//! │ indices[]    │ ──────▶ │ IBO (element array)          │
//! └──────────────┘         └──────────────────────────────┘
//! ```
//!
//! Cloning a `Mesh3D` or `Mesh2D` is cheap — they share the same GPU buffer
//! handles. Only the transform, shader, and visibility are per-instance.
//!
//! ## Shader Compilation
//!
//! Shaders are compiled from GLSL source via [`compile_shader`] and linked
//! via [`link_program`] (vertex + fragment) or [`link_compute_program`].
//! The compiled program is wrapped in a [`Shader`] handle.
//!
//! ```text
//! .glsl source file
//!   │
//!   ├─ "// V" marker → vertex source
//!   └─ "// F" marker → fragment source
//!         │
//!         ▼
//!   compile_shader(vert) → vertex GL object
//!   compile_shader(frag) → fragment GL object
//!         │
//!         ▼
//!   link_program(v, f) → GL program ID
//!         │
//!         ▼
//!   Shader { id, bound_textures[], bound_storages[] }
//! ```
//!
//! Each shader maintains 16 texture slots and 16 SSBO slots. Textures are
//! auto-assigned to the first empty slot via `attach_texture`, or explicitly
//! placed via `bind_texture(tex, Slot::S3)` to match a `layout(binding = 3)`
//! in GLSL.
//!
//! ## Instanced Rendering
//!
//! For drawing many copies of the same mesh efficiently, Optic uses GPU
//! instancing. An [`InstanceBuffer`] holds per-instance data (position,
//! rotation, scale, colour, custom attributes) interleaved in a single VBO.
//!
//! ```text
//! Single draw call renders N instances:
//!
//!   MeshHandle
//!     ├─ vertex VBO (shared geometry)
//!     ├─ IBO (shared indices)
//!     └─ instance VBO (per-instance transforms)
//!          │
//!          ▼
//!   glDrawElementsInstanced(TRIANGLES, index_count, UNSIGNED_INT,
//!                          null, instance_count)
//! ```
//!
//! Instance attributes use `glVertexAttribDivisor(1)` so they advance once
//! per instance (not per vertex). The attribute layout order is fixed:
//!
//! 1. `pos` — 3 × f32 (12 bytes)
//! 2. `rot` — 4 × f32 quaternion (16 bytes)
//! 3. `scale` — 3 × f32 (12 bytes)
//! 4. `col` — 4 × f32 RGBA (16 bytes)
//! 5. custom attributes (in insertion order)
//!
//! The `InstanceBuffer` maintains a **CPU mirror** — a complete copy of all
//! instance data in system memory. This enables instant reads and partial
//! writes without GPU round-trips. Every mutating method writes through to
//! both the CPU mirror and the GPU buffer.
//!
//! ## Canvas (Render-to-Texture)
//!
//! The [`Canvas`] type wraps one or more OpenGL FBOs for off-screen rendering.
//! Use it for post-processing, shadow maps, UI layers, or multi-pass rendering.
//!
//! ```text
//! Screen rendering:
//!   gpu.set_render_target(&RenderTarget::Screen);
//!   gpu.clear();
//!   gpu.render3d(&mesh, &camera);
//!
//! Canvas rendering:
//!   gpu.set_render_target(&RenderTarget::Canvas(&canvas))?;
//!   gpu.clear();
//!   gpu.render3d(&mesh, &camera);
//!
//!   // Present to screen
//!   canvas.blit_to_screen(window_size);
//! ```
//!
//! A canvas supports:
//! - **Multiple colour attachments** (MRT) — render to several textures at once
//! - **Depth/stencil** — as textures (for sampling in shaders) or renderbuffers
//! - **MSAA** — multisampled renderbuffers resolved via `glBlitFramebuffer`
//! - **Pixel readback** — `read_pixels()` returns CPU-side pixel data
//! - **Disk export** — `save_to_disk()` writes an image file
//!
//! ## GL State Management
//!
//! The engine manages the following OpenGL pipeline state:
//!
//! | State | Default | Controlled by |
//! |-------|---------|---------------|
//! | Depth testing | Enabled | `gpu.set_msaa()` / `gpu.toggle_msaa()` |
//! | Alpha blending | Enabled (SRC_ALPHA, ONE_MINUS_SRC_ALPHA) | Set during GPU init |
//! | Back-face culling | Enabled (counter-clockwise) | `gpu.set_culling()` |
//! | MSAA | Enabled (4 samples) | `gpu.set_msaa()` |
//! | Polygon mode | Filled | `gpu.set_poly_mode()` / `gpu.toggle_wireframe()` |
//! | Clear colour | Grey (0.5) | `gpu.set_bg_color()` |
//!
//! ## Rendering Targets
//!
//! The engine supports switching between the screen (default framebuffer) and
//! off-screen canvases at any point during the frame:
//!
//! ```text
//! gpu.set_render_target(&RenderTarget::Screen)?;      // back to screen
//! gpu.set_render_target(&RenderTarget::Canvas(&fbo))?; // render to FBO
//! ```
//!
//! The viewport is automatically resized to match the current target's
//! dimensions when switching.

mod camera;
mod context;
mod glraw;
pub mod handles;
mod renderer;
pub mod text;
mod util;

pub mod asset;

pub use camera::*;
pub use context::*;
pub use glraw::*;
pub use handles::*;
pub use renderer::*;
pub use util::*;
