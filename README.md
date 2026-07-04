# Optic

A modular OpenGL 4.6 engine for Rust. EGL native — same context API for windowed,
headless, and compute. Used in production by one guy's hobby projects.

```toml
[dependencies]
optic = { git = "https://github.com/Kono-o/optic-engine" }
```

Requires: OpenGL 4.6 GPU, EGL 1.5 drivers, Rust 1.70+.

## Why optic

- **No GLFW, no SDL, no glutin.** Just EGL directly. One context, any number
  of windows, pbuffer surfaces for headless work. Zero window-manager coupling.
- **Modular by default.** Seven crates with clean dependency edges. Want only
  the color library? `optic-color` is zero-dependency. Headless compute?
  `optic-core + optic-render`, no window crate needed.
- **No retained state guessing.** `Events` is a flat bit-field snapshot per
  frame. No callback spaghetti, no event queue to drain. Poll what you need.
- **Color types that actually work.** HSV/HSL conversions that handle hue
  wraparound. 115 named constants. A `Gradient` built on channel-arithmetic
  primitives so lerp doesn't give you wrong colors.

## Quick start

```rust
use optic::*;

struct App;

impl Runtime for App {
    fn start(&mut self, _game: &mut Game) {}
    fn update(&mut self, _game: &mut Game) {}
}

fn main() {
    GameBuilder::new()
        .with_title("Hello")
        .with_size(Size2D::from(800, 600))
        .build(App)
        .unwrap()
        .run();
}
```

Or with the simpler closure-based entry point:

```rust
use optic::*;

fn main() {
    optic::run("Hello", Size2D::from(800, 600), |frame| {
        frame.gpu.clear_frame(RGBA::grey(0.1));
    });
}
```

## Loading and rendering a 3D mesh

```rust
use optic::*;

struct App {
    mesh: Option<Mesh3D>,
    ready: bool,
}

impl Runtime for App {
    fn start(&mut self, game: &mut Game) {
        let file = Mesh3DFile::from_obj_cached("opt/mesh/cube.obj").unwrap();
        let mut mesh = game.renderer.add_mesh3d(&file);
        let shader = game.renderer.add_shader(
            &ShaderFile::from_path_cached("opt/shdr/3d.glsl", ShaderType::Pipeline).unwrap()
        ).unwrap();
        mesh.set_shader(shader);
        self.mesh = Some(mesh);
        self.ready = true;
    }

    fn update(&mut self, game: &mut Game) {
        if !self.ready { return; }
        let mesh = self.mesh.as_mut().unwrap();
        mesh.transform.rotate_y(30.0 * game.time.delta());
        mesh.update();
        game.renderer.render3d(mesh, &game.scene.camera);
    }
}
// GameBuilder::new().with_title("3D").build(App { .. }).unwrap().run();
```

## Architecture

```
optic-color       — standalone color library (~90 named constants, gradients)
optic-core        — shared types, geometry, errors, ANSI logging
optic-file        — read/write bytes/string, cached path resolution
optic-render      — EGL context, GL wrappers, shaders, meshes, textures,
                    framebuffers (Canvas), storage buffers, instance buffers,
                    camera, asset loading (OBJ, PNG, GLSL parse)
optic-window      — winit wrapper, per-frame key/mouse/gamepad state
optic-loop        — Game + GameBuilder, Runtime trait, Time, frame pacing
optic-online      — UDP networking on a background tokio thread (opt-in)
```

The `optic` meta-crate re-exports everything behind feature gates. Default
features give you the full stack. Cherry-pick what you need:

```toml
# headless compute only
optic = { git = "..", default-features = false, features = ["core", "render"] }

# just colors
optic-color = { git = ".." }
```

## Feature highlights

### Color

```rust
use optic::*;

let bg = RGBA::from_hex("#1a1a2e").unwrap();
let accent = GOLD_METALLIC;

let gradient = Gradient::two_color(bg, accent)
    .set_color_space(GradientColorSpace::Hsv);

for i in 0..10 {
    let t = i as f32 / 9.0;
    let c = gradient.sample(t);
    // c is an RGBA interpolated through HSV space
}
```

### 2D rendering with transforms

```rust
let mut mesh = game.renderer.add_mesh2d(&quad_file);
mesh.transform.set_pos(Vector2::new(400.0, 300.0));
mesh.transform.set_rot(core::f32::consts::FRAC_PI_4);
mesh.transform.set_scale(Vector2::new(2.0, 2.0));
mesh.update();
game.renderer.render2d(&mesh);
```

### Instanced rendering

```rust
let attrs = &[InstanceAttribute::new(
    ATTRName::Pos3D, ATTRType::F32, 3, 1
)];
let instances = game.renderer.add_instance_buffer(1000, attrs);
mesh.set_instances(&instances, 1000);

// per-instance data
let data: Vec<f32> = positions.iter().flat_map(|p| vec![p.x, p.y, p.z]).collect();
instances.set_data(0, bytemuck::cast_slice(&data));
```

### Canvas (framebuffer)

```rust
let canvas = game.renderer.create_canvas(
    Size2D::from(512, 512),
    Some(ImgFormat::RGBA(8)),
    None, None, false,
);
game.renderer.render_to_canvas(&canvas);
// canvas.color_texture(0) → use in a shader
```

### Headless compute

```rust
use optic::*;

let gpu = GPU::new_headless(1024, 1024, 1).unwrap();
let shader = gpu.create_shader(&ShaderType::Compute { .. });
shader.bind();
shader.dispatch(32, 32, 1);
// read back SSBO data with StorageBuffer
```

### Asset caching

Optic bakes decoded assets into a binary cache with magic header validation
(`/0PTIC_x`). On subsequent loads it skips parsing entirely.

| Source | Cache |
|--------|-------|
| `.obj` → parsed mesh | `.omesh` |
| `.glsl` → compiled (not cached) | `.oshdr` (source post-process) |
| `.png` / `.jpg` → decoded pixels | `.otxtr` |

Use `from_path_cached()` on `Mesh3DFile`, `ShaderFile`, or `TextureFile`.

### Networking (opt-in)

```rust
use optic::*;

let config = NetworkConfig::host(27015);
let game = GameBuilder::new()
    .with_network(config)
    .build(App)?
    .enable_networking();

// In update(): game.network() → &NetworkHandle
// game.network().send_all(data);
```

## API reference

Full documentation lives in [`docs/API.md`](docs/API.md).

## License

MIT
