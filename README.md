# Optic

A modular OpenGL 4.6 Core Profile rendering engine for Rust. EGL everywhere —
windowed and headless. One shared GL context, multiple windows, compute shaders.

## Crates

| Crate | Description |
|---|---|
| `optic-core` | Shared types: RGBA, Size2D, enums, errors |
| `optic-file` | Sanitary file I/O (read/write bytes/strings, `cached_path`) |
| `optic-render` | EGL context, GL state, shaders, meshes, textures, cameras, assets |
| `optic-window` | Winit window wrapper, key/mouse event state machine |
| `optic-loop` | Game loop (winit `ApplicationHandler`), auto-window-cleanup |
| `optic` | Meta-crate with feature gates (`core`, `file`, `render`, `window`, `minimal`, `full`) |

## Prerequisites

- OpenGL 4.6 capable GPU
- EGL 1.5 (provided by your GPU driver)
- Rust 1.70+

## Usage

```toml
[dependencies]
optic = { git = "https://github.com/Kono-o/optic-engine" }
```

Or cherry-pick crates:

```toml
[dependencies]
optic-core = { git = "https://github.com/Kono-o/optic-engine" }
optic-render = { git = "https://github.com/Kono-o/optic-engine" }
# no windowing, no loop — headless compute only
```

## Hello Window

```rust
use optic::prelude::*;

fn main() {
    optic::run("hello", Size2D::from(800, 600), |frame| {
        frame.gpu.clear();
        // ... render here
    });
}
```

## Loading and Rendering a 3D Mesh

```rust
use optic::prelude::*;

fn main() {
    optic::run("cube", Size2D::from(1024, 768), |frame| {
        frame.gpu.clear();

        for mesh in &meshes {
            mesh.render(&frame.camera.transform.view_matrix, &frame.camera.transform.persp_matrix);
        }
    });
}
```

## Asset Caching

Optic caches decoded assets in `optc/` subdirectories for faster subsequent loads:

| Format | Cache file |
|---|---|
| `.png` → `.otxtr` | `<dir>/optc/<name>.otxtr` |
| `.glsl` → `.oshdr` | `<dir>/optc/<name>.oshdr` |
| `.obj` → `.omesh` | `<dir>/optc/<name>.omesh` |

Use `from_path_cached` on `Image`, `ShaderAsset`, and `Mesh3DFile` to opt in.

## Headless / Compute Only

```toml
optic = { git = "https://github.com/Kono-o/optic-engine", default-features = false, features = ["core", "render"] }
```

```rust
let gpu = GPU::new_headless().unwrap();
// dispatch compute shaders, read back SSBOs
```

## Project Structure

```
optic/
  Cargo.toml              # workspace root
  optic-core/src/         # types, enums, errors, geometry
  optic-file/src/         # file I/O
  optic-render/src/       # EGL context, GL wrappers, handles, assets, camera
    context.rs            # EGL display/surface/context
    renderer.rs           # GPU high-level wrapper
    glraw.rs              # static GL state helpers
    handles/              # Shader, MeshHandle, Mesh3D/Mesh2D, Texture2D, StorageBuffer
    asset/                # OBJ parser, GLSL parser, Image loader
    camera/               # Camera with fly controls, ortho/persp
  optic-window/src/       # Window (winit), Events (key/mouse state machine)
  optic-loop/src/         # GameLoop, Time, run() convenience
  optic/src/              # feature-gated re-exports
```

## License

MIT
