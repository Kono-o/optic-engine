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
| `optic-loop` | Game loop, `Runtime` trait, `Game`/`GameBuilder`, `Time` |
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
use optic::*;

struct App;

impl Runtime for App {
    fn start(&mut self, _game: &mut Game) {}
    fn update(&mut self, _game: &mut Game) {}
}

fn main() {
    GameBuilder::new()
        .with_title("hello")
        .with_size(Size2D::from(800, 600))
        .build(App)
        .unwrap()
        .run();
}
```

## Loading and Rendering a 3D Mesh

```rust
use optic::*;

struct App {
    mesh: Option<Mesh3D>,
    done: bool,
}

impl Runtime for App {
    fn start(&mut self, game: &mut Game) {
        let file = Mesh3DFile::from_obj_cached("assets/mesh/cube.obj").unwrap();
        let mut mesh = game.renderer.add_mesh3d(&file);

        let asset = ShaderFile::from_path_cached(
            "assets/shader/3d.glsl", ShaderType::Pipeline,
        ).unwrap();
        let shader = game.renderer.add_shader(&asset).unwrap();
        mesh.set_shader(shader);

        self.mesh = Some(mesh);
        self.done = true;
    }

    fn update(&mut self, game: &mut Game) {
        if !self.done { return; }
        let mesh = self.mesh.as_mut().unwrap();
        mesh.transform.rotate_y(30.0 * game.time.delta());
        mesh.update();
        game.renderer.render3d(mesh, &game.scene.camera);
    }
}

fn main() {
    GameBuilder::new()
        .with_title("3D Viewer")
        .with_size(Size2D::from(1024, 768))
        .build(App { mesh: None, done: false })
        .unwrap()
        .run();
}
```

## Asset Caching

Optic caches decoded assets in `optc/` subdirectories for faster subsequent loads:

| Format | Cache file |
|---|---|
| `.png` → `.otxtr` | `<dir>/optc/<name>.otxtr` |
| `.glsl` → `.oshdr` | `<dir>/optc/<name>.oshdr` |
| `.obj` → `.omesh` | `<dir>/optc/<name>.omesh` |

Use `from_path_cached` on `TextureFile`, `ShaderFile`, and `Mesh3DFile` to opt in.

## Headless / Compute Only

```toml
optic = { git = "https://github.com/Kono-o/optic-engine", default-features = false, features = ["core", "render"] }
```

```rust
use optic::*;

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
    asset/                # OBJ parser, GLSL parser, TextureFile loader
    camera/               # Camera with fly controls, ortho/persp
  optic-window/src/       # Window (winit), Events (key/mouse state machine)
  optic-loop/src/         # GameLoop, Runtime trait, Game/GameBuilder, Scene, Time
  optic/src/              # feature-gated re-exports, prelude
```

## License

MIT
