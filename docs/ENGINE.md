# Optic Engine — Full Architecture Reference

> Written for an AI (Claude) to understand the engine holistically.
> Every struct, enum, trait, function, macro, and flow pattern is documented.
> No source browsing required — this is the single source of truth.

---

## Table of Contents

1. [Repository Structure](#1-repository-structure)
2. [optic-core — Foundation Types](#2-optic-core--foundation-types)
3. [optic-file — File Utilities](#3-optic-file--file-utilities)
4. [optic-window — Windowing & Input](#4-optic-window--windowing--input)
5. [optic-render — OpenGL Rendering](#5-optic-render--opengl-rendering)
6. [optic-loop — Game Loop](#6-optic-loop--game-loop)
7. [optic — Crate Facade & Prelude](#7-optic--crate-facade--prelude)
8. [Initialization Flow](#8-initialization-flow)
9. [Runtime Loop Flow](#9-runtime-loop-flow)
10. [Shutdown Flow](#10-shutdown-flow)
11. [Rendering Pipeline](#11-rendering-pipeline)
12. [GPU Resource Lifecycle & Stats](#12-gpu-resource-lifecycle--stats)
13. [Canvas / RenderTarget System](#13-canvas--rendertarget-system)
14. [Transform System](#14-transform-system)
15. [Complete Type Index](#15-complete-type-index)

---

## 1. Repository Structure

```
optic/                          # Workspace root
├── optic-core/                 # Foundation: types, enums, errors, colors, logging, consts
│   └── src/
│       ├── lib.rs              # Module declarations + pub re-exports + pub use cgmath
│       ├── ansi.rs             # ANSI terminal escape constants (48)
│       ├── color.rs            # RGBA, RGB structs + 65 named color constants
│       ├── consts.rs           # Path/file-extension string constants (13)
│       ├── coord.rs            # Coord2D, CoordOffset
│       ├── enums.rs            # PolyMode, Cull, DrawMode, ImgFormat, ImgFilter, ImgWrap, ATTRType
│       ├── error.rs            # OpticError, OpticErrorKind, OpticResult<T>
│       ├── geometry.rs         # Size2D, Size3D, ClipDist, Rect, CamProj
│       ├── log.rs              # log_color!, log_event!, log_info!, log_warn!, log_fatal! macros
│       └── proc.rs             # end(), end_success(), end_error(), SUCCESS, ERROR
│
├── optic-file/                 # Standalone file I/O: read/write/cache helpers
│   └── src/lib.rs              # 8 pub free functions
│
├── optic-window/               # Windowing (winit wrapper) + input state tracking
│   └── src/
│       ├── lib.rs              # Re-exports everything + pub use winit
│       ├── window.rs           # Window struct (inner Arc<WinitWindow> + state)
│       ├── events.rs           # Events, KeyBitMap, MouseBitMap, ButtonState, Is, Mouse, KeyCode
│       └── keys.rs             # (empty? KeyCode re-exported from winit)
│
├── optic-render/               # OpenGL 4.6 renderer (EGL/Khronos)
│   └── src/
│       ├── lib.rs              # Modules + pub use stats::GpuStats + pub use * from each
│       ├── glraw.rs            # GL unit struct — binding/state wrappers (22 methods)
│       ├── context.rs          # RenderContext, WindowSurface — EGL display/surface/context mgmt
│       ├── renderer.rs         # GPU struct — the main renderer API (30+ methods)
│       ├── stats.rs            # GpuStats struct + atomic counters (12 atomics)
│       ├── camera/
│       │   ├── mod.rs
│       │   └── camera.rs       # Camera, CamTransform (view/proj matrices, fly/spin controls)
│       ├── util/
│       │   ├── mod.rs
│       │   └── transform/
│       │       ├── mod.rs
│       │       ├── trans2d.rs  # Transform2D (pos, rot, scale, layer, aspect)
│       │       ├── trans3d.rs  # Transform3D (pos, rot(xyz), scale(xyz))
│       │       └── transcam.rs # CamTransform (view_matrix, persp/ortho matrices, front vector)
│       ├── handles/
│       │   ├── mod.rs          # pub mod + pub use for canvas/mesh/shader/texture
│       │   ├── texture.rs      # Texture2D, create_texture(), delete_texture()
│       │   ├── shader.rs       # Shader, Slot, Workers, compile/link/delete, uniform setters
│       │   ├── mesh.rs         # MeshHandle, Mesh3D, Mesh2D, StorageBuffer, buffer create/fill
│       │   └── canvas.rs       # Canvas, CanvasDesc, RenderTarget — FBO system
│       └── asset/
│           ├── mod.rs
│           ├── img.rs          # TextureFile (load PNG, cache, ship to GPU)
│           ├── msh.rs          # Mesh3DFile, Mesh2DFile, Center (OBJ parser, quad gen)
│           ├── shdr.rs         # ShaderFile, ShaderType (load GLSL, compile)
│           └── attr/
│               ├── mod.rs
│               ├── attr.rs     # ATTRInfo, ATTRName, Pos3DATTR, ColATTR, UVMATTR, etc.
│               └── typ.rs      # DataType trait (f32, u8, u32, etc.)
│
├── optic-loop/                 # Game loop orchestration
│   └── src/
│       ├── lib.rs              # Module decls + WindowState, FrameState, GameLoop, run()
│       ├── game.rs             # GameBuilder, Game (winit ApplicationHandler)
│       ├── runtime.rs          # Runtime trait (start/update/end)
│       ├── scene.rs            # Scene (owns Camera)
│       └── time.rs             # Time (delta, fps, elapsed, tick_count)
│
├── optic/                      # Facade crate — re-exports everything
│   └── src/
│       ├── lib.rs              # Feature-gated pub use of sub-crates
│       └── prelude.rs          # ~120+ items imported via optic::prelude::*
│
└── docs/
    └── ENGINE.md               # ← This file
```

**Dependency graph:**

```
vcb3d  ───►  optic  ───┬──►  optic-core
                        ├──►  optic-file
                        ├──►  optic-render  ──┬──►  optic-core
                        │                      ├──►  optic-file
                        │                      ├──►  cgmath
                        │                      ├──►  khronos-egl
                        │                      ├──►  gl
                        │                      ├──►  image
                        │                      └──►  raw-window-handle
                        ├──►  optic-window  ──┬──►  optic-core
                        │                      └──►  winit
                        └──►  optic-loop  ────┬──►  optic-core
                                                ├──►  optic-render
                                                └──►  optic-window
```

---

## 2. optic-core — Foundation Types

### 2.1 RGBA / RGB

```rust
// Tuple structs with 0.0–1.0 f32 fields
RGBA(pub f32, pub f32, pub f32, pub f32);
RGB(pub f32, pub f32, pub f32);

// Methods
RGBA::grey(lum: f32) -> RGBA        // R=G=B=lum, A=1.0
RGBA::from_rgb(rgb: RGB, a: f32) -> RGBA
RGBA::to_rgb(&self) -> RGB
RGB::grey(lum: f32) -> RGB
RGB::from_rgba(rgba: RGBA) -> RGB
RGB::to_rgba(&self, a: f32) -> RGBA
```

**65 named color constants** (all `pub const RGBA`):
RED, CRIMSON, PINK, BLUSH, CORAL, ORANGE, AMBER, GOLD, YELLOW, LIME, SPRING, SEA, FOREST, GREEN, TEAL, AQUA, SKY, CYAN, BLUE, MIDNIGHT, INDIGO, PURPLE, PLUM, DUSK, MAGENTA, FERN, SALMON, BROWN, GRAY, SILVER, WHITE, BLACK, OBSIDIAN, MAROON, BURGUNDY, SCARLET, PEACH, APRICOT, TANGERINE, MANGO, MUSTARD, OLIVE, CELADON, MINT, TURQUOISE, COBALT, NAVY, LAPIS, LAVENDER, VIOLET, WISTERIA, MULBERRY, ROSEWOOD, MAHOGANY, KHAKI, BEIGE, SAND, COPPER, BRONZE, SLATE, CHARCOAL, IVORY, ALABASTER, SNOW

### 2.2 Coordinate & Geometry Types

| Type | Fields | Key Methods |
|------|--------|-------------|
| `Coord2D` | `x: f64, y: f64` | `empty()`, `from(x,y)`, `from_tup((x,y))`, `is_inside(size)` |
| `CoordOffset` | `x: f64, y: f64` | `empty()`, `from(x,y)`, `from_tup((x,y))`, `is_zero()` |
| `Size2D` | `w: u32, h: u32` | `empty()`, `from(w,h)`, `shave(n)`, `aspect_ratio()` |
| `Size3D` | `w: u32, h: u32, d: u32` | `empty()`, `from(w,h,d)`, `shave(n)` |
| `ClipDist` | `near: f32, far: f32` | `from(near, far)`, `Default` → `{0.01, 1000.0}` |
| `Rect` | `x: i32, y: i32, w: i32, h: i32` | `from(x,y,w,h)` |

### 2.3 Enums

```rust
PolyMode  { Points, WireFrame, Filled }
Cull      { Clock, AntiClock }
DrawMode  { Points, Lines, Triangles(Default), Strip }
CamProj   { Ortho, Persp }

ImgFormat { R(u8), RG(u8), RGB(u8), RGBA(u8) }
  // channels() -> u8 (1..4)
  // bit_depth() -> u8 (inner)
  // pixel_size() -> u8 (channels * bit_depth)
  // from(channels: u8, bit_depth: u8) -> ImgFormat

ImgFilter { Closest, Linear }
ImgWrap   { Repeat, Extend, Clip }

ATTRType  { U8, I8, U16, I16, U32, I32, F32, F64 }
```

### 2.4 OpticError / OpticErrorKind / OpticResult

```rust
OpticErrorKind { Init, OpenGL, Shader, Asset, File, Framebuffer, Custom }

OpticError {
    kind: OpticErrorKind,
    msg: String,
}
// impl Display -> "optic error: {msg}"

// Constructors:
OpticError::new(kind, msg: &str) -> Self
OpticError::custom(msg: &str) -> Self  // kind = Custom

type OpticResult<T> = Result<T, OpticError>;
```

This is the unified error type used across all crates. Every fallible function returns `OpticResult<T>`.

### 2.5 ANSI Constants

48 terminal escape constants, type `ANSI { prefix: &'static str, suffix: &'static str }`.
Organized as: foreground (6), bold foreground (6), dark foreground (6), bold dark (6), background (6), bold background (6), dark background (6), bold dark background (6).
Each color: RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN.
Accessed as `ansi::BOLD_GREEN`, `ansi::BG_RED`, etc.

### 2.6 Path Constants (in `consts` module)

```
ASSET="opt/", TEMP="opt/temp/", SHDR_ASSET="opt/shdr/",
MESH_ASSET="opt/mesh/", TXTR_ASSET="opt/txtr/",
VERT="vert", FRAG="frag", GLSL="glsl", OBJ="obj", PNG="png",
OSHDR="oshdr", OMESH="omesh", OTXTR="otxtr"
```

### 2.7 Logging Macros (all #[macro_export])

```rust
log_color!("fmt", ansi::CONSTANT);          // Print with arbitrary color
log_color!("fmt", ansi::CONSTANT, args...); // With format args
log_event!("fmt");     // [EVENT] in BOLD_BLUE
log_info!("fmt");      // [INFO] in BOLD_GREEN
log_warn!("fmt");      // [WARN] in BOLD_YELLOW
log_fatal!("fmt");     // [FATAL] in BOLD_RED
```

All macros print to stdout with ANSI color prefix/suffix.

### 2.8 Process Control

```rust
SUCCESS: i32 = 0
ERROR:   i32 = 1

end(code: i32)           // dispatches to end_success or end_error
end_success()            // log_info!("~ end ~") + process::exit(0)
end_error()              // log_fatal!("~ end error ~") + process::exit(1)
```

### 2.9 cgmath Re-export

`optic_core` re-exports the entire `cgmath` crate as `pub use cgmath`. Key types used throughout:
`Matrix4<f32>`, `Vector3<f32>`, `Vector2<f32>`, `Rad<f32>`, `Point3<f32>`, `InnerSpace`.

---

## 3. optic-file — File Utilities

8 free functions, all returning `OpticResult<T>`:

```rust
name(path: &str) -> Option<String>            // file stem (no ext)
extension(path: &str) -> Option<String>       // extension
exists(path: &str) -> bool                    // path exists
read_bytes(path: &str) -> OpticResult<Vec<u8>>
read_string(path: &str) -> OpticResult<String>
write_bytes(path: &str, data: &[u8]) -> OpticResult<()>   // auto-creates parent dirs
write_string(path: &str, data: &str) -> OpticResult<()>
cached_path(source: &str, ext: &str) -> String  // "path/file.png" + "otxtr" → "path/optc/file.otxtr"
create_dir(path: &str) -> OpticResult<()>
```

---

## 4. optic-window — Windowing & Input

### 4.1 Window Struct

```rust
Window {
    inner: Option<Arc<WinitWindow>>,   // None = closed
    size: Size2D,
    title: String,
    fullscreen: bool,
    resizable: bool,
    cursor_hidden: bool,
    cursor_grabbed: bool,
    cursor_inside: bool,
    cursor_pos: (f64, f64),
    prev_cursor_pos: (f64, f64),
    cursor_delta: (f64, f64),
    coord: (f64, f64),             // window position on screen
    prev_coord: (f64, f64),
    prev_size: Size2D,
}
```

**Key methods:** `new(el, title, size)`, `close()`, `is_closed()`, `raw_handle() -> Option<RawWindowHandle>`, `size()`, `actual_size()`, `set_title()`, `set_size()`, `set_fullscreen()`, `toggle_fullscreen()`, `set_cursor_visibility()`, `set_cursor_grab()`, `toggle_cursor_usage()`, `cursor_offset()`, `cursor_coord()`, `cursor_coord_normalized()`, `request_redraw()`.

Construction: `Window::new(&el, title, size)` creates a winit window via `el.create_window(attrs)` and wraps it in `Arc`. The `inner` field is `Some(arc)` when open, `None` when closed.

### 4.2 Events & Input System

```rust
// Three-state per-button tracking
ButtonState { pressed: bool, held: bool, released: bool }
// pressed/released are one-frame flags, held persists

// Fixed-size arrays (not bitflags)
KeyBitMap(pub [ButtonState; 256])       // indexed by key_index()
MouseBitMap(pub [ButtonState; 8])       // indexed by mouse_index()

// Query action enum
Is { Pressed, Released, Held }

// Mouse button enum
Mouse { Left, Right, Middle, Back, Forward, Other(u16) }

// KeyCode: re-exported from winit::keyboard::KeyCode (all standard keys)
```

**Key mapping:** ~117 physical keys mapped to indices 0-116 (A-Z, 0-9, F1-F24, arrows, modifiers, numpad, etc.). Unmapped keys → index 255 (a sink).

**Events struct:**
```rust
Events {
    pub keys: KeyBitMap,
    pub mouse_buttons: MouseBitMap,
    pub resize_event: Option<Size2D>,
    pub close_requested: bool,
    keys_to_reset: Vec<KeyCode>,    // private
    mouse_to_reset: Vec<Mouse>,     // private
}
```

**Methods:**
- `process_window_event(&mut self, event: &WindowEvent, window: &Window)` — Updates key/mouse/resize/close state from winit events.
- `end_frame(&mut self)` — Clears `pressed`/`released` flags (one-frame reset). Must be called every frame.
- `key(&self, kc: KeyCode, action: Is) -> bool`
- `mouse(&self, m: Mouse, action: Is) -> bool`
- `key_combo(&self, primary: KeyCode, modifier: KeyCode, action: Is) -> bool` — e.g., `key_combo(KeyC, ControlLeft, Pressed)` for Ctrl+C.
- `key_combo_n(&self, keys: &[(KeyCode, Is)]) -> bool` — ALL must match.

### 4.3 winit Re-export

`optic_window` re-exports `winit` as `pub use winit`, making the entire winit crate available to consumers.

---

## 5. optic-render — OpenGL Rendering

This is the largest crate (~5,000 lines across 25 files). It manages OpenGL 4.6 via EGL (Khronos, not GLFW).

### 5.1 GL Unit Struct (`glraw.rs`)

A stateless unit struct with 22 static methods wrapping raw GL calls:

| Method | GL Call |
|--------|---------|
| `clear()` | `glClear(COLOR + DEPTH)` |
| `set_clear(rgba)` | `glClearColor` |
| `resize(size)` | `glViewport` |
| `poly_mode(mode)` | `glPolygonMode` |
| `enable_msaa(bool)` | `glEnable/Disable(GL_MULTISAMPLE)` |
| `enable_depth(bool)` | `glEnable/Disable(GL_DEPTH_TEST)` |
| `enable_alpha(bool)` | `glEnable/Disable(GL_BLEND)` + `glBlendFunc(SRC_ALPHA, ONE_MINUS_SRC_ALPHA)` |
| `enable_cull(bool)` | `glEnable/Disable(GL_CULL_FACE)` + `glCullFace(BACK)` |
| `set_cull_face(Cull)` | `glFrontFace(CW/CCW)` |
| `set_point_size(f32)` | `glPointSize` |
| `set_wire_width(f32)` | `glLineWidth` |
| `bind_shader(id)` | `glUseProgram` |
| `unbind_shader()` | `glUseProgram(0)` |
| `bind_texture_at(id, slot)` | `glActiveTexture(TEXTURE0+slot)` + `glBindTexture` |
| `unbind_texture()` | `glBindTexture(TEXTURE_2D, 0)` |
| `bind_vao(id)` | `glBindVertexArray` |
| `unbind_vao()` | `glBindVertexArray(0)` |
| `bind_buffer(id)` | `glBindBuffer(ARRAY_BUFFER, id)` |
| `unbind_buffer()` | `glBindBuffer(ARRAY_BUFFER, 0)` |
| `bind_ebo(id)` | `glBindBuffer(ELEMENT_ARRAY_BUFFER, id)` |
| `unbind_ebo()` | `glBindBuffer(ELEMENT_ARRAY_BUFFER, 0)` |
| `bind_ssbo(id)` | `glBindBuffer(SHADER_STORAGE_BUFFER, id)` |
| `unbind_ssbo()` | `glBindBuffer(SHADER_STORAGE_BUFFER, 0)` |

### 5.2 RenderContext (`context.rs`)

Manages the EGL display, context, and window surfaces.

```rust
WindowSurface { pub surface: egl::Surface, pub size: Size2D }

RenderContext {
    pub display: egl::Display,
    pub context: egl::Context,
    config: egl::Config,                     // private
    pub surfaces: Vec<WindowSurface>,
    pub active_index: Option<usize>,
    pub gl_ver: String,                       // e.g., "4.6 (core)"
    pub glsl_ver: String,                     // e.g., "4.50"
    pub device: String,                       // e.g., "AMD Radeon ..."
}
```

**Methods:** `new_headless()` (offscreen via pbuffer), `new_windowed(raw_handle, size)` (creates headless + attaches window), `attach_window(raw_handle, size) -> usize` (returns surface index), `resize_window(index, size)`, `make_current(index)`, `swap_buffers(index)`, `clear()`, `set_vsync(bool)`, `set_clear_color(RGBA)`.

**Constants:** Requests OpenGL 4.6 Core Profile. Config: RGBA8, Depth24, double-buffered.

**Drop:** Makes no context current, destroys all EGL surfaces, destroys context, terminates display. Does NOT clean up GL objects (textures, buffers, shaders) — those must be deleted manually.

### 5.3 GPU Struct (`renderer.rs`) — The Central API

```rust
GPU {
    pub ctx: RenderContext,
    pub poly_mode: PolyMode,
    pub cull_face: Cull,
    pub bg_color: RGBA,
    pub msaa: bool,
    pub msaa_samples: u32,
    pub culling: bool,
    pub fallback_shader2d: Shader,
    pub fallback_shader3d: Shader,
    pub fallback_texture: Texture2D,
    pub canvas_size: Size2D,
    pub(crate) current_target_size: Size2D,
    pub(crate) max_color_attachments: i32,   // queried via glGetIntegerv
    pub(crate) max_draw_buffers: i32,
    pub(crate) max_samples: i32,
}
```

**Construction** — `from_ctx(ctx)` (private, called by `new_headless()` and `new_windowed()`):
1. Enables depth test.
2. Queries GL caps: `MAX_COLOR_ATTACHMENTS`, `MAX_DRAW_BUFFERS`, `MAX_SAMPLES`.
3. Creates default state (bg=0.5 grey, msaa=4x, culling=anticlockwise, filled polys).
4. Loads fallback assets (texture: `optic/assets/txtr/fallback.png`, shaders: `fallback3d.glsl`, `fallback2d.glsl`).
5. Configures MSAA, culling, wire width, bg color, alpha blending.

**Full method index (all `pub`):**

| Method | Signature | Notes |
|--------|-----------|-------|
| `new_headless()` | `-> OpticResult<Self>` | Offscreen only |
| `new_windowed(handle, size)` | `-> OpticResult<Self>` | Windowed |
| `version()` | `-> &str` | GL version |
| `lang_version()` | `-> &str` | GLSL version |
| `name()` | `-> &str` | Device name |
| `clear()` | | Clears color+depth |
| `set_msaa_samples(u32)` | | |
| `set_bg_color(RGBA)` | | Also calls `ctx.set_clear_color` |
| `set_poly_mode(PolyMode)` | | |
| `toggle_wireframe()` | | |
| `set_msaa(bool)` | | |
| `toggle_msaa()` | | |
| `set_culling(bool)` | | |
| `toggle_culling()` | | |
| `set_cull_face(Cull)` | | |
| `flip_cull_face()` | | |
| `set_canvas_size(Size2D)` | | For 2D aspect math |
| `set_wire_width(f32)` | | |
| `set_point_size(f32)` | | |
| `log_backend_info()` | | |
| `log_info()` | | Current state |
| `fallback_shader3d()` | `-> Shader` | Returns clone |
| `fallback_shader2d()` | `-> Shader` | Returns clone |
| `ship_mesh3d(&Mesh3DFile)` | `-> Mesh3D` | With fallback shader |
| `ship_mesh2d(&Mesh2DFile)` | `-> Mesh2D` | With fallback shader |
| `ship_shader(&ShaderFile)` | `-> Option<Shader>` | None on compile failure |
| `ship_texture(&TextureFile)` | `-> Texture2D` | |
| `ship_canvas(&CanvasDesc)` | `-> OpticResult<Canvas>` | Validates GL caps |
| `set_render_target(&RenderTarget)` | `-> OpticResult<()>` | Screen or Canvas |
| `clear_target(Option<RGBA>, depth)` | | Clears color+depth |
| `current_render_target_size()` | `-> Size2D` | |
| `render3d(&Mesh3D, &Camera)` | | Calls `mesh.render(view, proj)` |
| `render2d(&Mesh2D)` | | Computes ortho from aspect |
| `stats()` | `-> GpuStats` | Snapshot of counters |
| `log_stats()` | | Pretty-print all counters |

### 5.4 GpuStats (`stats.rs`)

```rust
GpuStats {
    pub textures: u32,
    pub meshes_3d: u32,
    pub meshes_2d: u32,
    pub shaders: u32,
    pub framebuffers: u32,
    pub renderbuffers: u32,
    pub storage_buffers: u32,
    pub vertex_buffers: u32,
    pub index_buffers: u32,
    pub estimated_vram_bytes: u64,
}
```

Backed by 12 module-level `AtomicU32`/`AtomicU64` counters in `stats.rs`. `snapshot_stats()` loads all atomics with `Relaxed` ordering. `texture_vram_bytes(w, h, channels, bit_depth) -> u64` computes `w * h * channels * (bit_depth/8).max(1)`.

**Where each counter is incremented/decremented:**

| Counter | Incremented at | Decremented at |
|---------|---------------|----------------|
| `TEXTURES` | `create_texture()`, `create_empty_tex()`, `create_depth_tex()` | `Texture2D::delete()`, Canvas error paths |
| `TEXTURE_VRAM` | Same create sites | `Texture2D::delete()` |
| `MESHES_3D` | `GPU::ship_mesh3d()` (via macro) | `Mesh3D::delete()` (via macro) |
| `MESHES_2D` | `GPU::ship_mesh2d()` (via macro) | `Mesh2D::delete()` (via macro) |
| `SHADERS` | `link_program()`, `link_compute_program()` | `Shader::delete()`, error paths |
| `FRAMEBUFFERS` | `Canvas::new()` (FBO + resolve FBO) | `Canvas::delete()`, error paths |
| `RENDERBUFFERS` | `create_rbo_storage_msaa()`, inline in `Canvas::new()` | `Canvas::delete()`, error paths |
| `STORAGE_BUFFERS` | `create_storage_buffer()` | `StorageBuffer::delete()` |
| `VERTEX_BUFFERS` | `create_mesh_buffer()` | `MeshHandle::delete()` |
| `INDEX_BUFFERS` | `create_index_buffer()` | `MeshHandle::delete()` (conditional) |
| `BUFFER_VRAM` | `fill_buffer()`, `fill_index_buffer()` | (not decremented — approximate) |
| `FRAMEBUFFER_VRAM` | `Canvas::new()` | `Canvas::delete()` |

### 5.5 Handle Types

#### 5.5.1 Texture2D (`handles/texture.rs`)

```rust
Texture2D { pub id: u32, pub size: Size2D, pub fmt: ImgFormat, pub filter: ImgFilter, pub wrap: ImgWrap }
// Derives Clone, Debug

// Methods: new(), size(), wrap(), set_wrap(), filter(), set_filter(), delete(self)
```

**Free functions:**
- `create_texture(bytes, size, fmt, filter, wrap) -> u32` — Generates texture, sets params, uploads via `glTexImage2D`, generates mipmaps. Increments `TEXTURES` + `TEXTURE_VRAM`.
- `delete_texture(id: u32)` — Pure `glDeleteTextures`, no counter logic (counters handled in `Texture2D::delete()`).

#### 5.5.2 Shader (`handles/shader.rs`)

```rust
Shader {
    pub workers: Workers,
    pub id: u32,
    pub is_compute: bool,
    pub tex_ids: Vec<Option<u32>>,     // len = 16 (per SLOT)
    pub sbo_ids: Vec<Option<u32>>,     // len = 16
}
// Derives Clone, Debug

Slot { S0..S15 }  // as_index() -> 0..15, total_slots() -> 16
Workers { pub group_x: u32, pub group_y: u32, pub group_z: u32 }  // compute dispatch size
```

**Shader methods:** `new(id, is_compute)`, `attach_tex(&Texture2D)` (first free slot), `attach_sbo(&StorageBuffer)`, `set_tex_at_slot(&Texture2D, Slot)`, `set_sbo_at_slot(&StorageBuffer, Slot)`, `delete(self)`, `bind()`, `unbind()`, `compute()` (dispatch + barrier), `bind_textures()`, `bind_storages()`.

**Uniform setters** (all take `&self`, `name: &str`, value):
`set_i32`, `set_u32`, `set_f32`, `set_vec2_f32`/`vec3`/`vec4`, `set_vec2_i32`/`vec3`/`vec4`, `set_vec2_u32`/`vec3`/`vec4`, `set_m2_f32`/`m3`/`m4`.

**Free functions:**
- `compile_shader(src, type) -> OpticResult<u32>` — Compiles GLSL shader object.
- `link_program(vert, frag) -> OpticResult<u32>` — Compiles V+F, links program. Increments `SHADERS`.
- `link_compute_program(src) -> OpticResult<u32>` — Same for compute. Increments `SHADERS`.
- `delete_program(id: u32)` — Pure `glDeleteProgram`.

#### 5.5.3 MeshHandle / Mesh3D / Mesh2D (`handles/mesh.rs`)

```rust
MeshHandle {
    pub layouts: Vec<(ATTRInfo, u32)>,   // (attr_info, buffer_offset)
    pub draw_mode: DrawMode,
    pub has_indices: bool,
    pub vert_count: u32,
    pub ind_count: u32,
    pub vao_id: u32,
    pub buf_id: u32,                     // VBO
    pub ind_id: u32,                     // EBO (0 if no indices)
}
// Derives Clone, Debug

// Methods: draw(), delete(self) — decrements VERTEX_BUFFERS (+ INDEX_BUFFERS if indexed)
```

**Macro-generated mesh types:**
```rust
mesh_struct!(Mesh3D, Transform3D, MESHES_3D);
mesh_struct!(Mesh2D, Transform2D, MESHES_2D);

// Each has:
pub struct Mesh3D/Mesh2D {
    pub visibility: bool,
    pub handle: MeshHandle,
    pub shader: Option<Shader>,
    pub transform: Transform3D/Transform2D,
    pub draw_mode: DrawMode,
}
// Derives Clone, Debug

// Generated methods: set_shader(), remove_shader(), get/set_draw_mode(),
// index_count(), vertex_count(), has_indices(), is_empty(), is_visible(),
// set_visibility(), toggle_visibility(), update(), delete(self)
```

**Mesh3D-extra methods:** `log_info()`, `render(&self, view: &Matrix4, proj: &Matrix4)` — Binds shader, sets `uView`, `uProj`, `uTfm` uniforms, binds textures+storages, calls `handle.draw()`.

**Mesh2D-extra methods:** `log_info()`, `render(&self, proj: &Matrix4)` — Binds shader, sets `uProj`, `uTfm`, `uLayer`, binds textures+storages, calls `handle.draw()`.

**Buffer free functions:**
- `create_mesh_buffer() -> (vao_id, buf_id)` — Increments `VERTEX_BUFFERS`.
- `set_attr_layout(attr, attr_id, stride, offset)` — `glVertexAttribPointer` + `glEnableVertexAttribArray`.
- `fill_buffer(id, data: &[u8])` — `glBufferData`. Increments `BUFFER_VRAM`.
- `subfill_buffer(id, offset, data)` — `glBufferSubData`.
- `resize_buffer(id, size)` — `glBufferData` with null data.
- `create_index_buffer() -> u32` — Increments `INDEX_BUFFERS`.
- `fill_index_buffer(id, data: &[u32])` — `glBufferData`. Increments `BUFFER_VRAM`.

#### 5.5.4 StorageBuffer (`handles/mesh.rs`)

```rust
StorageBuffer { pub id: u32, pub size: usize }
// Methods: new(size), resize(size), fill(data), subfill(offset, data), fetch() -> Vec<u8>, delete(self)
// delete decrements STORAGE_BUFFERS
```

#### 5.5.5 Canvas & CanvasDesc (`handles/canvas.rs`)

```rust
CanvasDesc {
    pub size: Size2D,
    pub color_formats: Vec<ImgFormat>,     // color attachments
    pub depth: bool,
    pub depth_as_texture: bool,
    pub depth_compare: bool,               // shadow map mode
    pub stencil: bool,
    pub samples: u32,                      // 0 = no MSAA
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}
// Default: 512x512, RGBA(8), depth=true (as texture), samples=0, filter=Linear, wrap=Extend

Canvas {
    pub(crate) fbo_id: u32,
    pub(crate) resolve_fbo_id: u32,        // 0 if no MSAA
    pub(crate) msaa_rbos: Vec<u32>,        // per-color-attachment MSAA RBOs
    pub(crate) depth_stencil_rbo: u32,     // 0 if depth_as_texture
    pub(crate) color_texs: Vec<Texture2D>,
    pub(crate) depth_tex: Option<Texture2D>,
    pub(crate) size: Size2D,
    pub(crate) samples: u32,
    pub(crate) has_stencil: bool,
    pub(crate) has_depth: bool,
    pub(crate) depth_as_texture: bool,
    pub(crate) desc: CanvasDesc,
}

// Methods:
//   new(desc) -> OpticResult<Self> — Validates, creates FBO + attachments, checks completeness
//   size() -> Size2D
//   color_tex(index) -> OpticResult<&Texture2D>
//   depth_tex() -> Option<&Texture2D>
//   set_size(new_size) -> OpticResult<()>  — Re-creates canvas at new size
//   resolve() — Blits MSAA FBO → resolve FBO
//   blit_to_screen(window_size) — Resolve + blit to default FBO
//   set_renderable_area(Rect) — Viewport for rendering into canvas
//   read_pixels(index) -> OpticResult<Vec<u8>>
//   save_to_file(index, path) — Reads pixels, saves via image crate (supports L8..RGBA32F)
//   delete(&mut self) — Deletes FBOs, RBOs, textures. Decrements counters.

RenderTarget<'a> { Screen, Canvas(&'a Canvas) }
```

**Canvas::new() flow:**
1. Validates: at least one color format or depth, stencil requires depth, depth_compare requires depth_as_texture.
2. Generates main FBO (`glGenFramebuffers`). Increments `FRAMEBUFFERS` + `FRAMEBUFFER_VRAM`.
3. For each color format:
   - If MSAA: creates RBO via `create_rbo_storage_msaa()` (increments `RENDERBUFFERS`), attaches.
   - If no MSAA: creates empty texture via `create_empty_tex()` (increments `TEXTURES` + `TEXTURE_VRAM`), attaches.
4. Configures `glDrawBuffers`.
5. Depth:
   - MSAA: creates depth RBO (increments `RENDERBUFFERS`).
   - depth_as_texture: creates depth texture via `create_depth_tex()` (increments `TEXTURES` + `TEXTURE_VRAM`).
   - Neither: creates depth RBO (increments `RENDERBUFFERS`).
6. If MSAA: generates resolve FBO (increments `FRAMEBUFFERS`), attaches textures for each color + depth.
7. Checks `glCheckFramebufferStatus`. On failure: deletes all created objects (decrements counters), returns Err.

**Canvas::delete():** Decrements `FRAMEBUFFERS` (×1 or ×2), `RENDERBUFFERS` (×n), then calls `tex.delete()` on each Texture2D (which decrements textures + VRAM).

#### 5.5.6 Important Handle Lifecycle Rules

- All handle types are `Clone`. Clones share the same underlying GL object.
- No `Drop` impls on any handle — cleanup is manual via `.delete()`.
- `Canvas::delete(&mut self)` — mutable borrow (unique).
- `Texture2D::delete(self)`, `Shader::delete(self)`, `Mesh3D::delete(self)`, `Mesh2D::delete(self)`, `MeshHandle::delete(self)`, `StorageBuffer::delete(self)` — consuming.
- Double-delete is a bug (both in GL and in counters).

### 5.6 Asset Loaders

#### 5.6.1 TextureFile (`asset/img.rs`)

```rust
TextureFile {
    pub bytes: Vec<u8>,
    pub size: Size2D,
    pub fmt: ImgFormat,
    pub filter: ImgFilter,    // default Closest
    pub wrap: ImgWrap,        // default Clip
}

// Methods:
//   from_path(path) -> OpticResult<Self>  — Uses image crate to load PNG etc.
//   from_path_cached(path) -> OpticResult<Self>  — Loads from cache or from path + saves cache
//   ship(&self) -> Texture2D  — Calls create_texture(), returns Texture2D
//   save_cached(&self, path)
//   from_cached(path) -> OpticResult<Self>
//   fallback() -> OpticResult<Self>  — Loads "optic/assets/txtr/fallback.png"
```

Cached format: binary file with `[channels:u8, bit_depth:u8, w:u32le, h:u32le, filter:u8, wrap:u8, bytes...]`.

`ship()` calls `create_texture()` which is the central GL texture creation function.

#### 5.6.2 Mesh3DFile / Mesh2DFile (`asset/msh.rs`)

```rust
Mesh3DFile {
    pub pos_attr: Pos3DATTR,
    pub col_attr: ColATTR,
    pub uvm_attr: UVMATTR,
    pub nrm_attr: NrmATTR,
    pub ind_attr: IndATTR,
    pub cus_attrs: Vec<CustomATTR>,
}
// Methods: empty(), from_obj_src(src), from_obj(path), from_obj_cached(path), ship() -> MeshHandle

Mesh2DFile {
    pub pos_attr: Pos2DATTR,
    pub layer: u8,
    pub aspect: f32,
    pub col_attr: ColATTR,
    pub uvm_attr: UVMATTR,
    pub ind_attr: IndATTR,
    pub cus_attrs: Vec<CustomATTR>,
}
// Methods: empty(), quad(size) -> centered quad with UVs, fullscreen_quad() -> NDC quad (-1..1),
//          ship() -> MeshHandle, set_pos_attr, set_center(Center), etc.

Center { TopLeft, TopRight, BottomLeft, BottomRight, Middle, Custom(f32, f32) }
```

**Mesh processing logic (`create_mesh3d_handle` / `create_mesh2d_handle`):**
1. Calls `create_mesh_buffer()` → generates VAO + VBO.
2. Interleaves all non-empty attribute data into a single vertex buffer (POS, COL, UVM, NRM).
3. For each attribute: calls `set_attr_layout()` with stride and offset.
4. Uploads vertex data via `fill_buffer()`.
5. If has indices: `create_index_buffer()` + `fill_index_buffer()`.
6. Returns `MeshHandle` with the VAO/VBO/EBO IDs, layout info, and counts.

**OBJ parser** (`OBJ::parse`): Handles `v`, `vt`, `vn`, `f` lines. Only triangles. Deduplicates vertices by position+uv+normal key. Returns `OBJ::Parsed { pos_attr, col_attr, uvm_attr, nrm_attr, ind_attr }` or `NonTriangle`.

#### 5.6.3 ShaderFile (`asset/shdr.rs`)

```rust
ShaderType { Pipeline, Compute }

ShaderFile {
    pub v_src: String,
    pub f_src: String,
    pub is_compute: bool,
}

// Methods:
//   from_src(src, typ) -> OpticResult<Self>  — Parses GLSL source for //VERTEX/FRAGMENT markers
//   from_path(path, typ) -> OpticResult<Self>
//   from_vert_frag(v_src, f_src) -> Self  — Direct construction
//   from_path_cached(path, typ) -> OpticResult<Self>
//   compile(&self) -> OpticResult<Shader>  — Links program, returns Shader
//   default_3d() / default_2d() -> OpticResult<Self>  — Loads from optic/assets/shdr/
```

**GLSL parsing:** For `Pipeline` type, splits source at `//v` / `//vert` / `//VERTEX` markers (and various `//` comment style variants like `#VERTEX`, `//VERT`, etc.) into vertex and fragment sections.

#### 5.6.4 Attribute System (`asset/attr/`)

```rust
ATTRInfo { pub name: ATTRName, pub elem_count: u32, pub typ: ATTRType }
ATTRName { Custom(String), Pos2D, Pos3D, Col, UVM, Nrm, Ind }

// Concrete attribute types (all generic over DataType):
Pos3DATTR  // pos: Vec<[T; 3]>
Pos2DATTR  // pos: Vec<[T; 2]>
ColATTR    // col: Vec<[T; 4]>
UVMATTR    // uvm: Vec<[T; 2]>
NrmATTR    // nrm: Vec<[T; 3]>
IndATTR    // ind: Vec<u32>  (always u32 indices)
CustomATTR // name: String, data: Vec<u8>, elem_count: u32, typ: ATTRType

// Each has: empty(), is_empty(), count(), push(), data_ptr(), data_size_bytes()

DataType trait — implemented for f32, u8, u16, u32, i8, i16, i32, f64
```

### 5.7 Camera System (`camera/`)

```rust
CamTransform {
    pub pos: Vector3<f32>,
    pub rot: Vector3<f32>,           // pitch, yaw, roll (degrees)
    pub fov: f32,                     // default 75
    pub clip: ClipDist,               // default {0.01, 1000.0}
    pub size: Size2D,                 // viewport size
    pub proj: CamProj,                // Persp or Ortho
    pub view_matrix: Matrix4<f32>,
    pub ortho_scale: f32,             // default 2.0
    pub front: Vector3<f32>,          // computed from pitch/yaw
    pub persp_matrix: Matrix4<f32>,
    pub ortho_matrix: Matrix4<f32>,
}
// Derives Clone, Debug

// Methods:
//   calc_matrices(&mut self) — Computes front vector, view matrix (look_at_rh), persp/ortho matrices
//   view_matrix() / proj_matrix()

Camera {
    pub transform: CamTransform,
}
// Methods: new(size, proj), match_canvas_size(canvas, proj), pre_update(),
//          fov/set_fov/add_fov, ortho_scale/set/add, set_size, set_proj,
//          set_clip/set_clip_near/set_clip_far,
//          fly_forw/back/left/right/up/down(speed), spin_x/y/z(speed)
```

`Camera::new(size, proj)` defaults: pos=(0,0,5), rot=(0,-90,0), fov=75, ortho_scale=2.0.

### 5.8 Transform System (`util/transform/`)

```rust
Transform2D {
    matrix: Matrix4<f32>,    // private — computed by calc_matrix()
    pos: Vector2<f32>,       // private — position in screen coords
    rot: f32,                // private — rotation in degrees
    layer: u8,               // private — Z layer
    aspect: f32,             // private — width/height ratio
    scale: Vector2<f32>,     // private
}
// Default: identity, (0,0), rot 0, layer 0, aspect 1.0, scale (1,1)
// Methods: calc_matrix, get/set pos/rot/layer/aspect/scale/matrix, move_all/x/y, rotate, etc.

Transform3D {
    matrix: Matrix4<f32>,    // private
    pos: Vector3<f32>,       // private
    rot: Vector3<f32>,       // private — XYZ rotation in degrees
    scale: Vector3<f32>,     // private
}
// Default: identity, (0,0,0), rot (0,0,0), scale (1,1,1)
// Methods: calc_matrix, get/set pos/rot/scale/matrix, move/rotate/scale per-axis

// Matrix computation: matrix = position_matrix * rotation_matrix * scale_matrix
```

---

## 6. optic-loop — Game Loop

### 6.1 Two Loop Architectures

The engine provides two ways to run a game loop:

#### 6.1.1 Game + Runtime (trait-based, single-window, feature-rich)

```rust
// User implements this trait:
trait Runtime {
    fn start(&mut self, game: &mut Game);   // called once before first frame
    fn update(&mut self, game: &mut Game);  // called every frame
    fn end(&mut self, game: &mut Game) {}   // called on shutdown (default no-op)
}
```

**Game struct (public fields):**
```rust
Game {
    pub renderer: GPU,      // the GL renderer
    pub scene: Scene,       // owns Camera
    pub events: Events,     // input state
    pub time: Time,         // frame timing
    // private: window, surface_index, runtime, running, started, requested_size, resized_once
}
```

**GameBuilder:**
```rust
GameBuilder::new()
    .with_title("title")
    .with_size(Size2D::from(w, h))
    .logic(my_runtime)          // -> OpticResult<Game>
    .unwrap()
    .run();
```

**Usage pattern (from vcb3d):**
```rust
struct MyScene { meshes3d: Vec<Mesh3D>, meshes2d: Vec<Mesh2D> }

impl Runtime for MyScene {
    fn start(&mut self, game: &mut Game) {
        // Load assets: TextureFile, Mesh3DFile, ShaderFile
        // Upload:   game.renderer.ship_texture(), ship_mesh3d(), ship_shader()
        // Wire up:  shader.attach_tex(), mesh.set_shader()
        // Configure: transforms, renderer settings
    }
    fn update(&mut self, game: &mut Game) {
        // Input:    game.events.key(KeyCode, Is::Held)
        // Camera:   game.scene.camera.fly_forw(), spin_x()
        // Update:   mesh.update()
        // Draw:     game.renderer.render3d(&mesh, &game.scene.camera)
        //           game.renderer.render2d(&mesh)
    }
    fn end(&mut self, game: &mut Game) {
        // Cleanup:  mesh.delete(), texture.delete(), shader.delete()
    }
}
```

#### 6.1.2 GameLoop + Closure (simpler, multi-window)

```rust
// Single-window convenience:
optic_loop::run("title", Size2D::from(800, 600), |frame: &mut FrameState| {
    // frame.time, frame.windows, frame.gpu, frame.camera
});

// Multi-window (manual):
let el = EventLoop::new().unwrap();
let ws = vec![WindowState::new(&el, "win1", size1), WindowState::new(&el, "win2", size2)];
let gpu = GPU::new_headless().unwrap();
// attach windows to gpu...
let camera = Camera::new(size1, CamProj::Persp);
GameLoop::new(el, gpu, camera, ws, |frame: &mut FrameState| {
    // per-frame logic
}).run();
```

**FrameState:**
```rust
FrameState<'a> {
    pub time: &'a Time,
    pub windows: &'a mut [WindowState],
    pub gpu: &'a mut GPU,
    pub camera: &'a mut Camera,
}
```

**WindowState:**
```rust
WindowState {
    pub window: Window,
    pub events: Events,
    pub surface_index: usize,
}
```

### 6.2 Time Struct

```rust
Time {
    pub fps: f64,              // rolling-average FPS
    pub delta: f64,            // seconds since last frame
    pub tick_count: u64,       // total frames
    pub elapsed: f64,          // seconds since start
    pub start_time: Instant,
    pub prev_time: Instant,
    pub prev_sec: Instant,
    pub local_tick: u32,       // ticks since last second boundary   // private
    prev_deltas: Vec<f64>,     // max 32 entries                      // private
    prev_deltas_size: usize,                                          // private
}
// Methods: new(), update(), fps(), delta(), elapsed(), now(), now_ms(), sleep(secs), sleep_ms(), sleep_ns()

// update() logic:
//   tick_count++, local_tick++
//   delta = now - prev_time
//   elapsed = now - start_time
//   Push delta into rolling window (max 32)
//   fps = 1.0 / average_delta
//   If 1 sec since prev_sec: local_tick = 0
```

### 6.3 Scene Struct

```rust
Scene {
    pub camera: Camera,
}
// new(size, proj) -> creates Camera
```

---

## 7. optic — Crate Facade & Prelude

### 7.1 lib.rs

```rust
pub mod prelude;

#[cfg(feature = "core")]    pub use optic_core::*;
#[cfg(feature = "file")]    pub use optic_file::*;
#[cfg(feature = "render")]  pub use optic_render::*;
#[cfg(feature = "window")]  pub use optic_window::*;
#[cfg(feature = "minimal")] pub use optic_loop::*;
```

### 7.2 Prelude (~120+ items)

```rust
// cgmath
pub use cgmath;
pub use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector2, Vector3, vec3};

// optic_core modules
pub use optic_core::{ansi, consts};
// optic_core types (20+)
pub use optic_core::{CamProj, ClipDist, Coord2D, CoordOffset, Cull, ATTRType, ...};
// 65 named color constants
pub use optic_core::{RED, GREEN, ... SNOW};
// 5 logging macros
pub use optic_core::{log_color, log_event, log_fatal, log_info, log_warn};
// 4 exit functions/consts
pub use optic_core::{end, end_error, end_success, ERROR, SUCCESS};

pub use optic_file;

// optic_loop (9 items)
pub use optic_loop::{FrameState, Game, GameBuilder, GameLoop, Runtime, Scene, Time, WindowState, run};

// optic_render attrs (10)
pub use optic_render::asset::attr::{ATTRInfo, ATTRName, ColATTR, ..., DataType};
// optic_render assets (5)
pub use optic_render::asset::{Center, Mesh2DFile, Mesh3DFile, ShaderFile, ShaderType, TextureFile};
// optic_render handles (17)
pub use optic_render::{Camera, Canvas, CanvasDesc, GL, GPU, GpuStats, Mesh2D, Mesh3D, ...};

// optic_window (8)
pub use optic_window::{Events, Is, KeyBitMap, KeyCode, Mouse, MouseBitMap, Window};
```

---

## 8. Initialization Flow

### 8.1 Full Flow (GameBuilder + Runtime path)

```
1. User calls GameBuilder::new()
   └─ Creates builder with defaults:
      title="Optic Game", size=800x600, bg=RGBA::grey(0.5)

2. User chains .with_title("vcb3d"), .with_size(Size2D::from(1280, 720))

3. User calls .logic(MyRuntime)
   └─ GameBuilder::logic() does:
      a. Creates winit EventLoop::builder().build()
      b. Creates Window::new(&el, &title, size)
         ├─ Creates winit WindowAttributes with title + size
         ├─ Calls el.create_window(attrs) -> Arc<WinitWindow>
         └─ Returns Window { inner: Some(arc), size, title, ... }
      c. Gets actual window size (may differ from requested)
      d. Gets raw window handle via window.raw_handle()
         └─ Calls HasWindowHandle on WinitWindow -> RawWindowHandle
      e. Creates GPU::new_headless()
         ├─ RenderContext::new_headless()
         │   ├─ EGL: get_display, initialize, choose_config (RGBA8, Depth24, pbuffer)
         │   ├─ Create pbuffer surface (1x1)
         │   ├─ Create context (OpenGL 4.6 Core Profile)
         │   ├─ Make current (pbuffer)
         │   ├─ Load GL functions via eglGetProcAddress
         │   ├─ Query GL/GLSL version, device name
         │   └─ Return RenderContext { display, context, config, surfaces: [], ... }
         ├─ GPU::from_ctx(ctx):
         │   ├─ Enable depth test
         │   ├─ Query GL caps: MAX_COLOR_ATTACHMENTS, MAX_DRAW_BUFFERS, MAX_SAMPLES
         │   ├─ Create fallback Texture2D (id=0, placeholder)
         │   ├─ Load fallback assets:
         │   │   ├─ TextureFile::fallback() -> ship_texture() -> Texture2D
         │   │   ├─ ShaderFile::default_3d() -> ship_shader() -> Shader
         │   │   ├─ ShaderFile::default_2d() -> ship_shader() -> Shader
         │   └─ Configure: MSAA on, culling anticlockwise, wire width 2.0, bg color, alpha on
         └─ Return GPU { ctx, bg_color, msaa, ... }
      f. GPU::ctx.attach_window(raw_handle, actual_size) -> surface_index
         ├─ Creates EGL window surface from RawWindowHandle
         ├─ Pushes to surfaces vec
         └─ Returns index
      g. make_current(surface_index), VSync on
      h. Set gpu.canvas_size = actual_size, set_bg_color(bg)
      i. Create Scene::new(actual_size, CamProj::Persp)
         └─ Camera::new(size, Persp):
            pos=(0,0,5), rot=(0,-90,0), fov=75, clip=(0.01,1000), ortho_scale=2.0
            calc_matrices() -> front, view, persp, ortho
      j. Return Game { renderer: gpu, scene, events, time, window, runtime: Some(Box::new(user_runtime)), ... }

4. User calls game.run()
   └─ Calls el.run_app(&mut game) -> enters winit event loop
      ├─ resumed: time.start_time = prev_time = prev_sec = now()
      └─ about_to_wait: (first iteration)
           ├─ !started -> runtime.start(self)
           └─ runtime.update(self) -> user's per-frame logic
```

### 8.2 Quick-Start Flow (GameLoop + closure path)

```
optic_loop::run("title", Size2D::from(800,600), |frame| { ... })
├─ Creates EventLoop, Window, GPU: same as steps 3a-3f above
├─ Creates Camera::new(size, Persp): same as 3i
└─ Creates GameLoop::new(el, gpu, camera, windows, closure):
   ├─ For each window: ctx.attach_window(handle, size) -> surface_index
   └─ game.run() -> el.run_app(&mut game)
```

---

## 9. Runtime Loop Flow

### 9.1 Game + Runtime Path

The winit `ApplicationHandler::about_to_wait` is the core loop body. It fires every time the OS says "prepare a frame."

```
about_to_wait(self, el):
│
├─ [SHUTDOWN CHECK]
│  if !self.running || self.window.is_closed():
│  │  ├─ runtime.take() → Option::take() removes runtime from Game
│  │  ├─ runtime.end(self) → user cleanup
│  │  ├─ Stats leak check:
│  │  │  if any counter > 0: log_warn!("GPU resource leak detected...")
│  │  ├─ end_success() → log_info!("~ end ~") + process::exit(0)
│  │  └─ el.exit()
│  └─ return
│
├─ [FRAME SETUP]
│  ├─ ctx.make_current(surface_index)
│  ├─ renderer.clear() → glClear(COLOR | DEPTH)
│  ├─ time.update() → computes delta, fps, elapsed, tick_count
│  ├─ Compute cursor delta: cursor_delta = cursor_pos - prev_cursor_pos
│  └─ camera.pre_update() → recalc view/proj matrices
│
├─ [USER CODE]
│  ├─ runtime = runtime.take().unwrap()   ─── borrow moved out of Game
│  ├─ if !started:
│  │  ├─ runtime.start(self)              ─── user asset loading, mesh creation, etc.
│  │  └─ started = true
│  ├─ runtime.update(self)                ─── user per-frame logic
│  └─ self.runtime = Some(runtime)        ─── borrow returned to Game
│
├─ [FRAME END]
│  ├─ ctx.swap_buffers(surface_index)     ─── present to screen
│  ├─ events.end_frame()                  ─── clear one-frame key/mouse flags
│  └─ window.request_redraw()             ─── request next about_to_wait
```

### 9.2 GameLoop + Closure Path

Same structure, but:
- No `start`/`end` method — only the closure runs every frame.
- Multi-window: iterates all windows, removes closed ones, ends frames on all.
- `FrameState` is built with `&self.time`, `&mut self.windows`, `&mut gpu`, `&mut self.camera`.
- The closure receives `&mut FrameState`.

---

## 10. Shutdown Flow

### 10.1 Triggers

- User calls `game.exit()` → sets `self.running = false`.
- Window close button → `WindowEvent::CloseRequested` → `events.close_requested = true`.
- Window is programmatically closed via `window.close()` → `inner = None` → `is_closed() = true`.

### 10.2 Shutdown Sequence

```
about_to_wait detects shutdown condition:
1. runtime.end(self)  — user can manually delete GPU resources here
2. GPU leak check     — log_warn! if any GpuStats counter > 0
     "GPU resource leak detected at shutdown: N textures, M shaders, ..."
3. end_success()      — [INFO] ~ end ~ (green) + process::exit(0)
```

**Note:** The engine does NOT automatically clean up GL objects on shutdown. It relies on the OS/driver to reclaim GPU memory when the EGL context is destroyed (via `RenderContext::Drop`). The leak check is a warning, not a hard error — non-zero counts are expected for resources meant to live the entire program lifetime.

---

## 11. Rendering Pipeline

### 11.1 Per-Frame Render Order (from vcb3d pattern)

```
1. GPU::clear()                                           — glClear(COLOR | DEPTH)
   (called automatically by Game before user update)
2. User sets uniforms on currently-bound shader
3. For each 3D mesh:
   a. GPU::render3d(&mesh, &camera)
      └─ Mesh3D::render(&view, &proj):
         1. Check visibility
         2. shader.bind()                                — glUseProgram
         3. Set uniform: uView = camera.view_matrix()
         4. Set uniform: uProj = camera.proj_matrix()
         5. Set uniform: uTfm = mesh.transform.matrix()
         6. shader.bind_textures()                       — glActiveTexture + glBindTexture
         7. shader.bind_storages()                       — glBindBufferBase
         8. handle.draw()
            ├─ GL::bind_vao(vao_id)
            ├─ if has_indices: GL::bind_ebo(ind_id)
            │  └─ glDrawElements(GL_TRIANGLES, ind_count, UNSIGNED_INT, null)
            └─ else: glDrawArrays(mode, 0, vert_count)
4. For each 2D mesh:
   a. GPU::render2d(&mesh)
      └─ Mesh2D::render(&proj):
         Same as above but:
         - uProj = ortho(-aspect, aspect, -1, 1, -1, 1)
         - uTfm = transform.matrix()
         - uLayer = mesh.transform.layer()
         - Z position = uLayer * 0.001 (in vertex shader)
5. swap_buffers                                         — eglSwapBuffers
```

### 11.2 Rendering with Canvas (FBO) Pattern

```
1. gpu.ship_canvas(&desc)                                — create offscreen FBO
2. gpu.set_render_target(&RenderTarget::Canvas(&canvas)) — bind FBO
3. gpu.clear_target(Some(color), true)                   — clear FBO
4. // render meshes onto canvas...
5. gpu.set_render_target(&RenderTarget::Screen)           — bind default FBO
6. canvas.blit_to_screen(window_size)                     — blit canvas to screen
   OR
   // sample canvas.color_tex(0) in a shader
```

### 11.3 Canvas MSAA Resolution

```
Canvas with samples>1:
  Main FBO (fbo_id):           all attachments are MSAA RBOs/textures
  Resolve FBO (resolve_fbo_id): all attachments are non-MSAA textures
  
  After rendering to canvas:
  - canvas.resolve()            — glBlitFramebuffer(MSAA → resolve)
  - canvas.color_tex(i)         — returns resolve FBO's texture (only valid after resolve)
  - canvas.read_pixels(i)       — calls resolve_if_needed() internally
  - canvas.save_to_file(i,path) — calls resolve_if_needed() internally
  - canvas.blit_to_screen()     — calls resolve_if_needed() internally
```

---

## 12. GPU Resource Lifecycle & Stats

### 12.1 Resource Creation Pattern

All GL resources are created through `pub` free functions in the `handles/` modules, which are called either directly or via `GPU::ship_*` methods:

```
Texture2D:
  TextureFile::ship() → create_texture() → glGenTextures + glTexImage2D
  │                     └─ incr: TEXTURES, TEXTURE_VRAM
  └─ Called by GPU::ship_texture()

Mesh3D/Mesh2D:
  Mesh3DFile::ship() → create_mesh3d_handle()
  │                     ├─ create_mesh_buffer() → glGenVertexArrays + glGenBuffers
  │                     │   └─ incr: VERTEX_BUFFERS
  │                     ├─ fill_buffer() → glBufferData
  │                     │   └─ incr: BUFFER_VRAM
  │                     ├─ create_index_buffer() → glGenBuffers
  │                     │   └─ incr: INDEX_BUFFERS
  │                     └─ fill_index_buffer() → glBufferData
  │                         └─ incr: BUFFER_VRAM
  └─ Called by GPU::ship_mesh3d()/ship_mesh2d()

Shader:
  ShaderFile::compile() → link_program() → glCreateProgram + glLinkProgram
  │                        └─ incr: SHADERS
  └─ Called by GPU::ship_shader()

Canvas:
  GPU::ship_canvas(desc) → Canvas::new(desc)
    ├─ glGenFramebuffers (main FBO)           ── incr: FRAMEBUFFERS, FRAMEBUFFER_VRAM
    ├─ for each color: create_empty_tex()     ── incr: TEXTURES, TEXTURE_VRAM
    │                 OR create_rbo_storage_msaa() ── incr: RENDERBUFFERS
    ├─ depth: create_depth_tex()              ── incr: TEXTURES, TEXTURE_VRAM
    │       OR inline glGenRenderbuffers      ── incr: RENDERBUFFERS
    ├─ if MSAA: glGenFramebuffers (resolve)   ── incr: FRAMEBUFFERS
    └─ if MSAA: create_empty_tex() per color  ── incr: TEXTURES, TEXTURE_VRAM

StorageBuffer:
  StorageBuffer::new(size) → create_storage_buffer() → glGenBuffers
                              └─ incr: STORAGE_BUFFERS
```

### 12.2 Resource Deletion Pattern

```
Texture2D::delete(self):
  ├─ decr: TEXTURES, TEXTURE_VRAM
  └─ delete_texture(self.id) → glDeleteTextures

Shader::delete(self):
  ├─ decr: SHADERS
  └─ delete_program(self.id) → glDeleteProgram

Mesh3D::delete(self) / Mesh2D::delete(self) (via macro):
  ├─ decr: MESHES_3D or MESHES_2D
  └─ MeshHandle::delete(self):
       ├─ decr: VERTEX_BUFFERS
       ├─ if has_indices: decr: INDEX_BUFFERS
       └─ glDeleteVertexArrays + glDeleteBuffers

StorageBuffer::delete(self):
  ├─ decr: STORAGE_BUFFERS
  └─ glDeleteBuffers

Canvas::delete(&mut self):
  ├─ decr: FRAMEBUFFERS (×1 or ×2)
  ├─ decr: RENDERBUFFERS (×N)
  ├─ glDeleteFramebuffers + glDeleteRenderbuffers
  └─ tex.delete() on each color/depth texture  — decr: TEXTURES, TEXTURE_VRAM
```

### 12.3 VRAM Estimation

`estimated_vram_bytes` = sum of:
- `TEXTURE_VRAM`: `w * h * channels * (bit_depth / 8)` per texture, at creation time. Subtracted on `Texture2D::delete()`.
- `BUFFER_VRAM`: sum of all `glBufferData` sizes for vertex/index data. Added at `fill_buffer()`/`fill_index_buffer()`. **Not subtracted** (would require storing per-buffer sizes).
- `FRAMEBUFFER_VRAM`: `w * h * 4` per FBO, estimated. Subtracted on `Canvas::delete()`.

**Documented caveats:** Does not include driver overhead, alignment/padding, internal mipmaps, or renderbuffer storage. Not comparable to profiler-reported VRAM.

---

## 13. Canvas / RenderTarget System

### 13.1 CanvasDesc — Configuration

| Field | Type | Default | Notes |
|-------|------|---------|-------|
| `size` | `Size2D` | (512,512) | Pixel dimensions |
| `color_formats` | `Vec<ImgFormat>` | [RGBA(8)] | One Texture2D per entry |
| `depth` | `bool` | true | Enable depth attachment |
| `depth_as_texture` | `bool` | true | Sampleable depth texture vs RBO |
| `depth_compare` | `bool` | false | GL_TEXTURE_COMPARE_MODE for shadow maps |
| `stencil` | `bool` | false | Requires depth=true |
| `samples` | `u32` | 0 | 0 = no MSAA, >1 = MSAA with N samples |
| `filter` | `ImgFilter` | Linear | Texture filter mode |
| `wrap` | `ImgWrap` | Extend | Texture wrap mode |

### 13.2 Canvas Internal Structure

```
Canvas (no MSAA):
  fbo_id → [COLOR_ATTACHMENT0: Texture2D, ...]
           [DEPTH_ATTACHMENT:   Texture2D or RBO]

Canvas (MSAA, samples > 1):
  fbo_id         → [COLOR_ATTACHMENT0: RBO (multisampled), ...]
                   [DEPTH_ATTACHMENT:   RBO (multisampled)]
  resolve_fbo_id → [COLOR_ATTACHMENT0: Texture2D, ...]
                   [DEPTH_ATTACHMENT:   Texture2D (if depth_as_texture)]
```

### 13.3 Y-Flip Convention

Canvas textures are in GL's native bottom-left origin. `color_tex()`/`depth_tex()` docs warn users to flip UVs in shaders when sampling. `fullscreen_quad()` does NOT auto-flip.

### 13.4 sRGB Note

Canvases use linear internal formats. Users compositing onto screen must do linear→sRGB in shader.

### 13.5 Canvas Methods

| Method | Purpose |
|--------|---------|
| `new(desc)` | Create FBO with all attachments |
| `size()` | Get pixel dimensions |
| `color_tex(i)` | Get color attachment as Texture2D |
| `depth_tex()` | Get depth texture (if depth_as_texture) |
| `set_size(new)` | Recreate at new size (same desc) |
| `resolve()` | Blit MSAA FBO → resolve FBO |
| `blit_to_screen(win)` | Blit to default FBO |
| `set_renderable_area(Rect)` | Viewport for rendering into canvas |
| `read_pixels(i)` | CPU readback of color attachment |
| `save_to_file(i, path)` | Save color attachment as image file |
| `delete(&mut self)` | Delete all GL objects |

---

## 14. Transform System

### 14.1 Transform2D

For 2D sprites. The model matrix is computed as `pos_matrix * rot_matrix * scale_matrix` where:

- `pos`: `Vector2<f32>` — screen position (normalized to NDC, ~ -1..1 range adjusted by aspect)
- `rot`: `f32` — rotation around Z axis (degrees)
- `scale`: `Vector2<f32>` — uniform or per-axis scale
- `layer`: `u8` — Z-order, passed as `uLayer` uniform, used as `uLayer * 0.001` in vertex shader for depth
- `aspect`: `f32` — width/height ratio for non-square correction

Position and scale use the aspect ratio internally: `pos_matrix` multiplies x by aspect to maintain screen-space proportions.

### 14.2 Transform3D

For 3D objects. Matrix = `pos_matrix * rot_matrix * scale_matrix` where:

- `pos`: `Vector3<f32>` — world position
- `rot`: `Vector3<f32>` — Euler angles (pitch, yaw, roll) in degrees, applied as ZYX
- `scale`: `Vector3<f32>` — per-axis scale

### 14.3 CamTransform

Camera transform. Computed in `calc_matrices()`:
- `front`: `(cos(pitch)*cos(yaw), sin(pitch), cos(pitch)*sin(yaw))` — look direction
- `view_matrix`: `look_at_rh(pos, pos + front, up)`
- `persp_matrix`: `perspective(Rad(fov), aspect, near, far)`
- `ortho_matrix`: `ortho(-scale*aspect, scale*aspect, -scale, scale, near, far)`

---

## 15. Complete Type Index

### optic-core (26 types + 65 colors + 48 ANSI + 13 consts)
| Category | Items |
|----------|-------|
| Structs | `RGBA`, `RGB`, `Coord2D`, `CoordOffset`, `Size2D`, `Size3D`, `ClipDist`, `Rect`, `ANSI`, `OpticError` |
| Enums | `PolyMode`, `Cull`, `DrawMode`, `ImgFormat`, `ImgFilter`, `ImgWrap`, `ATTRType`, `CamProj`, `OpticErrorKind` |
| Type Alias | `OpticResult<T>` |
| Macros | `log_color!`, `log_event!`, `log_info!`, `log_warn!`, `log_fatal!` |
| Functions | `end()`, `end_success()`, `end_error()` |

### optic-file (8 functions)
`name`, `extension`, `exists`, `read_bytes`, `read_string`, `write_bytes`, `write_string`, `cached_path`, `create_dir`

### optic-window (8 types + winit)
`Window`, `Events`, `KeyBitMap`, `MouseBitMap`, `ButtonState`, `Is`, `Mouse`, `KeyCode` (from winit)

### optic-render (30+ types)
| Category | Items |
|----------|-------|
| Core | `GPU`, `GpuStats`, `GL`, `RenderContext`, `WindowSurface` |
| Handles | `Texture2D`, `Shader`, `Workers`, `Slot`, `MeshHandle`, `Mesh3D`, `Mesh2D`, `StorageBuffer`, `Canvas`, `CanvasDesc`, `RenderTarget` |
| Camera | `Camera`, `CamTransform` |
| Transforms | `Transform2D`, `Transform3D` |
| Assets | `TextureFile`, `Mesh3DFile`, `Mesh2DFile`, `ShaderFile`, `ShaderType`, `Center` |
| Attributes | `ATTRInfo`, `ATTRName`, `Pos2DATTR`, `Pos3DATTR`, `ColATTR`, `UVMATTR`, `NrmATTR`, `IndATTR`, `CustomATTR`, `DataType` (trait) |

### optic-loop (9 types)
`Game`, `GameBuilder`, `Runtime` (trait), `Scene`, `Time`, `WindowState`, `FrameState`, `GameLoop`, `run` (fn)

### Key Traits
| Trait | Defined In | Methods |
|-------|-----------|---------|
| `Runtime` | optic-loop | `start(&mut self, &mut Game)`, `update(&mut self, &mut Game)`, `end(&mut self, &mut Game)` |
| `DataType` | optic-render/attr | (implemented for f32, u8, u16, u32, i8, i16, i32, f64) |

---

## Appendix: Shader Uniform Conventions

### 3D Pipeline Shader

```glsl
// Vertex shader inputs (location 0-3)
layout(location = 0) in vec3 vPos;
layout(location = 1) in vec4 vCol;
layout(location = 2) in vec2 vUVM;
layout(location = 3) in vec3 vNrm;

// Uniforms set by engine
layout(location = 0) uniform mat4 uView;     // Camera view matrix
layout(location = 1) uniform mat4 uProj;     // Camera projection matrix
layout(location = 2) uniform mat4 uTfm;      // Mesh model matrix

// Optional uniforms
layout(location = 4) uniform vec3 uLight;    // Light direction (default: 0.5, 1.0, 0.3)
uniform sampler2D Tex0;                      // First bound texture
```

### 2D Pipeline Shader

```glsl
// Vertex shader inputs (location 0-2)
layout(location = 0) in vec3 vPos;           // Actually vec2 in Pos2DATTR, padded
layout(location = 1) in vec4 vCol;
layout(location = 2) in vec2 vUVM;

// Uniforms set by engine
layout(location = 0) uniform mat4 uProj;     // Orthographic projection
layout(location = 1) uniform mat4 uTfm;      // Sprite model matrix
layout(location = 2) uniform uint uLayer;    // Z-order (used as uLayer * 0.001)

uniform sampler2D Tex0;                      // First bound texture
```

### Compute Shader

Compute shaders are linked via `ShaderFile::from_src(src, ShaderType::Compute)` → `link_compute_program(src)`. Dispatching is done via `Shader::compute()`, which calls `glDispatchCompute(group_x, group_y, group_z)` and `glMemoryBarrier`. The `Workers` struct holds the group counts. Textures are bound via `glBindImageTexture` (not `glBindTexture`). Storage buffers are bound via `glBindBufferBase`.

---

*End of ENGINE.md — this document captures the complete state of the Optic engine as of July 2026.*
