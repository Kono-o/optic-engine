# Getting Started with Optic

## Prerequisites

- **GPU**: OpenGL 4.6 capable (for compute shader support)
- **EGL 1.5**: Provided by your GPU driver (`libegl1-mesa-dev` on Debian/Ubuntu, `libegl1` on Fedora)
- **Rust**: Edition 2021, stable toolchain

## Quick Start: Single-Window App

The `optic::run()` convenience function creates a window, initializes EGL/OpenGL, and runs your frame callback:

```rust
use optic::prelude::*;

fn main() {
    optic::run("hello optic", Size2D::from(800, 600), |frame| {
        frame.gpu.clear();
    });
}
```

`optic::run` gives you a `FrameState` with:
- `frame.gpu` — the GPU/GL context wrapper
- `frame.camera` — a perspective camera
- `frame.windows` — all open windows
- `frame.time` — delta, FPS, elapsed

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

fn main() {
    // Load the cube mesh from OBJ
    let mesh_file = Mesh3DFile::from_obj("assets/mesh/cube.obj").unwrap();
    let mesh_handle = mesh_file.ship();

    // Load and compile the shader
    let shader_asset = ShaderAsset::from_path("assets/shader/3d.glsl", ShaderType::Pipeline).unwrap();
    let shader = shader_asset.compile().unwrap();

    // Create a renderable mesh with a transform
    let mut mesh = Mesh3D {
        visibility: true,
        handle: mesh_handle,
        shader: Some(shader),
        transform: Transform3D::default(),
        draw_mode: DrawMode::Triangles,
    };

    optic::run("3D Viewer", Size2D::from(1024, 768), move |frame| {
        // Rotate the mesh
        mesh.transform.rotate_y(30.0 * frame.time.delta());
        mesh.update();

        // Clear and render
        frame.gpu.clear();
        mesh.render(
            &frame.camera.transform.view_matrix,
            &frame.camera.transform.persp_matrix,
        );
    });
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

And in Rust:

```rust
// Load texture and attach to shader
let img = Image::from_path("assets/tex/crate.png").unwrap();
let tex = img.ship();
let shader = shader_asset.compile().unwrap();
shader.set_tex_at_slot(&tex, Slot::S0);
```

Now the cube will be textured.

## Input & Camera Control

```rust
use optic::prelude::*;

fn main() {
    // Manual setup for full control
    let el = winit::event_loop::EventLoop::new().unwrap();
    let mut ws = WindowState::new(&el, "controls", Size2D::from(800, 600));

    let handle = ws.window.raw_handle().unwrap();
    let mut gpu = GPU::new_windowed(handle, ws.window.size).unwrap();
    let mut camera = Camera::new(ws.window.size, CamProj::Persp);

    // Set up meshes, shaders, etc. ...

    let game = GameLoop::new(el, gpu, camera, vec![ws], move |frame| {
        // Access window + events from the first window
        let ws = &mut frame.windows[0];
        let speed = 3.0 * frame.time.delta();

        // Camera movement
        if ws.events.key(Key::W, Is::Held) {
            frame.camera.transform.fly_forw(speed);
        }
        if ws.events.key(Key::S, Is::Held) {
            frame.camera.transform.fly_back(speed);
        }
        if ws.events.key(Key::A, Is::Held) {
            frame.camera.transform.fly_left(speed);
        }
        if ws.events.key(Key::D, Is::Held) {
            frame.camera.transform.fly_right(speed);
        }
        if ws.events.key(Key::Space, Is::Held) {
            frame.camera.transform.fly_up(speed);
        }
        if ws.events.key(Key::ShiftLeft, Is::Held) {
            frame.camera.transform.fly_down(speed);
        }

        // Camera look (mouse)
        if ws.events.mouse(Mouse::Right, Is::Held) {
            let (dx, dy) = ws.window.cursor_delta;
            frame.camera.transform.spin_y(dx as f32 * 0.1);
            frame.camera.transform.spin_x(dy as f32 * 0.1);
        }

        // Exit on Escape
        if ws.events.key(Key::Escape, Is::Pressed) {
            ws.close();
        }

        frame.camera.pre_update();
        frame.gpu.clear();
    });

    game.run();
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

    let shader_asset = ShaderAsset::from_src(src, ShaderType::Compute).unwrap();
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

```rust
use optic::prelude::*;

fn main() {
    let el = winit::event_loop::EventLoop::new().unwrap();

    let win1 = WindowState::new(&el, "Window 1", Size2D::from(800, 600));
    let win2 = WindowState::new(&el, "Window 2", Size2D::from(400, 300));

    let handle1 = win1.window.raw_handle().unwrap();
    let handle2 = win2.window.raw_handle().unwrap();

    let mut gpu = GPU::new_windowed(handle1, win1.window.size).unwrap();
    // Attach second window to the same GL context
    gpu.ctx.attach_window(handle2, win2.window.size).unwrap();

    let camera = Camera::new(win1.window.size, CamProj::Persp);

    let game = GameLoop::new(el, gpu, camera, vec![win1, win2], move |frame| {
        // Render to each window
        for ws in &mut frame.windows {
            if ws.is_closed() { continue; }
            frame.gpu.ctx.make_current(ws.surface_index).unwrap();
            frame.gpu.clear();
            // ... draw calls here
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
let img = Image::from_path_cached("assets/tex/crate.png").unwrap();
let tex = img.ship();

let mesh_file = Mesh3DFile::from_obj_cached("assets/mesh/cube.obj").unwrap();
let shader = ShaderAsset::from_path_cached("assets/shader/3d.glsl", ShaderType::Pipeline).unwrap();
```

## Full API at a Glance

| What | Type / Function | Module |
|------|----------------|--------|
| Window | `Window` | `optic_window` |
| Events (key/mouse) | `Events` | `optic_window` |
| EGL context | `RenderContext` | `optic_render` |
| GPU wrapper | `GPU` | `optic_render` |
| Camera | `Camera` | `optic_render` |
| Shader (runtime) | `Shader` | `optic_render::handles` |
| Mesh (runtime) | `Mesh3D`, `Mesh2D` | `optic_render::handles` |
| Texture (runtime) | `Texture2D` | `optic_render::handles` |
| SSBO | `StorageBuffer` | `optic_render::handles` |
| OBJ loader | `Mesh3DFile` | `optic_render::asset` |
| GLSL loader | `ShaderAsset` | `optic_render::asset` |
| Image loader | `Image` | `optic_render::asset` |
| Transform 3D | `Transform3D` | `optic_render::util::transform` |
| Transform 2D | `Transform2D` | `optic_render::util::transform` |
| Camera Transform | `CamTransform` | `optic_render::util::transform` |
| Game Loop | `GameLoop` | `optic_loop` |
| Time | `Time` | `optic_loop` |
| File I/O | `read_bytes`, `write_string`, etc. | `optic_file` |
| Shared types | `Size2D`, `RGBA`, `ImgFormat`, etc. | `optic_core` |
