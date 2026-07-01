# Getting Started with Optic

## Prerequisites

- **GPU**: OpenGL 4.6 capable (for compute shader support)
- **EGL 1.5**: Provided by your GPU driver (`libegl1-mesa-dev` on Debian/Ubuntu, `libegl1` on Fedora)
- **Rust**: Edition 2021, stable toolchain

## Quick Start: Single-Window App

The new high-level API uses a `GameBuilder` + `Runtime` trait — no manual GL
management, no closures, no window handle plumbing:

```rust
use optic::prelude::*;

struct App;

impl Runtime for App {
    fn start(&mut self, _game: &mut Game) {}
    fn update(&mut self, _game: &mut Game) {}
}

fn main() {
    GameBuilder::new()
        .with_title("hello optic")
        .with_size(Size2D::from(800, 600))
        .build(App)
        .unwrap()
        .run();
}
```

`Game` (available inside `Runtime` methods) exposes:
- `game.renderer` — the `GPU` wrapper for rendering
- `game.scene` — holds the active `Camera`
- `game.events` — key/mouse state machine
- `game.time` — delta, FPS, elapsed

Everything else — window creation, EGL surface attachment, `make_current`,
`clear`, `swap_buffers` — happens automatically inside the game loop.

## Step by Step: 3D Mesh Viewer

### 1. Project Setup

```toml
[dependencies]
optic = { git = "https://github.com/Kono-o/optic-engine" }
cgmath = "0.18"
```

### 2. Asset Files

Create `assets/shader/3d.glsl`:

```glsl
// VERTEX
#version 460 core
layout (location = 0) in vec3 vPos;
layout (location = 1) in vec3 vCol;
layout (location = 2) in vec2 vUVM;

uniform mat4 uView;
uniform mat4 uProj;
uniform mat4 uTfm;

out vec3 fCol;
out vec2 fUVM;

void main() {
    gl_Position = uProj * uView * uTfm * vec4(vPos, 1.0);
    fCol = vCol;
    fUVM = vUVM;
}

// FRAGMENT
#version 460 core
in vec3 fCol;
in vec2 fUVM;

out vec4 fragPIXEL;

void main() {
    fragPIXEL = vec4(fCol, 1.0);
}
```

Create `assets/mesh/cube.obj`:

```obj
v  0.5  0.5  0.5
v  0.5 -0.5  0.5
v -0.5 -0.5  0.5
v -0.5  0.5  0.5
v  0.5  0.5 -0.5
v  0.5 -0.5 -0.5
v -0.5 -0.5 -0.5
v -0.5  0.5 -0.5
f 1 2 3
f 3 4 1
f 5 8 7
f 7 6 5
f 1 5 6
f 6 2 1
f 2 6 7
f 7 3 2
f 3 7 8
f 8 4 3
f 5 1 4
f 4 8 5
```

### 3. Full Application

```rust
use optic::prelude::*;

struct App {
    mesh: Option<Mesh3D>,
    shader: Option<Shader>,
}

impl Runtime for App {
    fn start(&mut self, game: &mut Game) {
        let file = Mesh3DFile::from_obj("assets/mesh/cube.obj").unwrap();
        let mut mesh = game.renderer.add_mesh3d(&file);

        let asset = ShaderFile::from_path(
            "assets/shader/3d.glsl", ShaderType::Pipeline,
        ).unwrap();
        let shader = game.renderer.add_shader(&asset).unwrap();
        mesh.set_shader(shader.clone());

        self.mesh = Some(mesh);
        self.shader = Some(shader);
    }

    fn update(&mut self, game: &mut Game) {
        let mesh = match &mut self.mesh { Some(m) => m, None => return };

        mesh.transform.rotate_y(30.0 * game.time.delta());
        mesh.update();

        game.renderer.render3d(mesh, &game.scene.camera);
    }
}

fn main() {
    GameBuilder::new()
        .with_title("3D Viewer")
        .with_size(Size2D::from(1024, 768))
        .build(App { mesh: None, shader: None })
        .unwrap()
        .run();
}
```

Run with:

```bash
cargo run
```

### 4. Add a Texture

Create `assets/tex/crate.png` (any square PNG), then update the shader:

```glsl
// FRAGMENT (updated)
in vec3 fCol;
in vec2 fUVM;

out vec4 fragPIXEL;

uniform sampler2D Tex0;

void main() {
    fragPIXEL = texture(Tex0, fUVM);
}
```

And in Rust, upload the texture and bind it to the shader before setting it on the mesh:

```rust
fn start(&mut self, game: &mut Game) {
    let file = Mesh3DFile::from_obj("assets/mesh/cube.obj").unwrap();
    let mut mesh = game.renderer.add_mesh3d(&file);

    let asset = ShaderFile::from_path(
        "assets/shader/3d.glsl", ShaderType::Pipeline,
    ).unwrap();
    let mut shader = game.renderer.add_shader(&asset).unwrap();

    let img = TextureFile::from_path("assets/tex/crate.png").unwrap();
    let tex = game.renderer.add_texture(&img);
    shader.set_tex_at_slot(&tex, Slot::S0);

    mesh.set_shader(shader.clone());

    self.mesh = Some(mesh);
    self.shader = Some(shader);
}
```

Now the cube will be textured.

## Input & Camera Control

```rust
use optic::prelude::*;

struct App {
    mesh: Option<Mesh3D>,
    shader: Option<Shader>,
    loaded: bool,
}

impl Runtime for App {
    fn start(&mut self, game: &mut Game) {
        // load mesh, shader, texture ...
        self.loaded = true;
    }

    fn update(&mut self, game: &mut Game) {
        if !self.loaded { return; }

        let speed = 3.0 * game.time.delta();

        // Camera movement (WASD + Space/Shift)
        if game.events.key(KeyCode::KeyW, Is::Held) {
            game.scene.camera.fly_forw(speed);
        }
        if game.events.key(KeyCode::KeyS, Is::Held) {
            game.scene.camera.fly_back(speed);
        }
        if game.events.key(KeyCode::KeyA, Is::Held) {
            game.scene.camera.fly_left(speed);
        }
        if game.events.key(KeyCode::KeyD, Is::Held) {
            game.scene.camera.fly_right(speed);
        }
        if game.events.key(KeyCode::Space, Is::Held) {
            game.scene.camera.fly_up(speed);
        }
        if game.events.key(KeyCode::ShiftLeft, Is::Held) {
            game.scene.camera.fly_down(speed);
        }

        // Camera look (right mouse button + drag)
        if game.events.mouse(Mouse::Right, Is::Held) {
            let (dx, dy) = game.window().cursor_delta;
            game.scene.camera.spin_y(dx as f32 * 0.1);
            game.scene.camera.spin_x(dy as f32 * 0.1);
        }

        game.scene.camera.pre_update();

        // Exit on Escape
        if game.events.key(KeyCode::Escape, Is::Pressed) {
            game.exit();
        }

        // Render
        if let Some(mesh) = &self.mesh {
            game.renderer.render3d(mesh, &game.scene.camera);
        }
    }
}
```

## Headless Compute

```toml
[dependencies]
optic = { git = "https://github.com/Kono-o/optic-engine", default-features = false, features = ["core", "render"] }
```

```rust
use optic::prelude::*;

fn main() {
    let gpu = GPU::new_headless().unwrap();

    let src = r#"
        #version 460 core
        layout(local_size_x = 64) in;
        layout(std430, binding = 0) buffer Data { float values[]; };
        void main() {
            uint idx = gl_GlobalInvocationID.x;
            values[idx] = values[idx] * 2.0;
        }
    "#;

    let shader_asset = ShaderFile::from_src(src, ShaderType::Compute).unwrap();
    let mut shader = shader_asset.compile().unwrap();

    let mut sbo = StorageBuffer::new(64 * 4); // 64 floats
    let data: Vec<f32> = (0..64).map(|i| i as f32).collect();
    sbo.fill(bytemuck::cast_slice(&data));

    shader.set_sbo_at_slot(&sbo, Slot::S0);
    shader.workers.set_group_x(1);
    shader.compute();

    let result = sbo.fetch();
    let floats: &[f32] = bytemuck::cast_slice(&result);
    println!("{:?}", floats);
}
```

## Multi-Window

The high-level `Game` API currently supports a single window. For multiple
windows, use the low-level `GameLoop` directly:

```rust
use optic::prelude::*;

fn main() {
    let el = winit::event_loop::EventLoop::new().unwrap();

    let win1 = WindowState::new(&el, "Window 1", Size2D::from(800, 600));
    let win2 = WindowState::new(&el, "Window 2", Size2D::from(400, 300));

    let handle1 = win1.window.raw_handle().unwrap();
    let handle2 = win2.window.raw_handle().unwrap();

    let mut gpu = GPU::new_windowed(handle1, win1.window.size).unwrap();
    gpu.ctx.attach_window(handle2, win2.window.size).unwrap();

    let camera = Camera::new(win1.window.size, CamProj::Persp);

    let game = GameLoop::new(el, gpu, camera, vec![win1, win2], move |frame| {
        for ws in &mut frame.windows {
            if ws.is_closed() { continue; }
            frame.gpu.ctx.make_current(ws.surface_index).unwrap();
            frame.gpu.clear();
            frame.gpu.ctx.swap_buffers(ws.surface_index).unwrap();
        }
    });

    game.run();
}
```

## Asset Caching

Use `from_path_cached` to auto-cache decoded assets in `optc/` subdirectories:

```rust
// First run: loads PNG, saves .otxtr; subsequent runs: loads .otxtr directly
let img = TextureFile::from_path_cached("assets/tex/crate.png").unwrap();

let mesh_file = Mesh3DFile::from_obj_cached("assets/mesh/cube.obj").unwrap();
let shader = ShaderFile::from_path_cached("assets/shader/3d.glsl", ShaderType::Pipeline).unwrap();
```

Then use `game.renderer.add_*` to upload them to the GPU (inside `start()`
or `update()`).

## Full API at a Glance

| What | Type | Access |
|------|------|--------|
| Runtime trait | `Runtime` | Implement on your app struct |
| Game builder | `GameBuilder` | `GameBuilder::new().with_title(...)` |
| Game (loop + state) | `Game` | `game.renderer`, `game.scene`, `game.events`, `game.time` |
| Window info | `Window` | `game.window()` |
| Scene | `Scene` | `game.scene.camera` |
| Camera | `Camera` | Fly controls, ortho/persp |
| GPU / Renderer | `GPU` | `game.renderer.add_mesh3d()`, `.add_shader()`, `.render3d()`, etc. |
| EGL context | `RenderContext` | Low-level, via `game.renderer.ctx` |
| Shader (runtime) | `Shader` | Uniform setters, texture/SSBO binding |
| Mesh (runtime) | `Mesh3D`, `Mesh2D` | Transform, visibility, draw mode |
| Texture (runtime) | `Texture2D` | Wrap, filter, size |
| SSBO | `StorageBuffer` | Fill, fetch, resize |
| OBJ loader | `Mesh3DFile` | `from_obj()`, `from_obj_cached()` |
| GLSL loader | `ShaderFile` | `from_path()`, `compile()` |
| Image loader | `TextureFile` | `from_path()`, `ship()` |
| Transform 3D | `Transform3D` | Position, rotation, scale |
| Transform 2D | `Transform2D` | Position, rotation, layer, scale |
| Events | `Events` | `key()`, `mouse()`, `key_combo()` |
| Time | `Time` | `delta()`, `fps()`, `elapsed()` |
| File I/O | `read_bytes`, `write_string`, etc. | `optic_file` |
| Shared types | `Size2D`, `RGBA`, `ImgFormat`, etc. | `optic_core` |
