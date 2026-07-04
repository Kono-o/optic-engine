# Optic Engine

A modular Rust game engine composed of independently-compilable sub-crates.

`optic-engine` is the **facade crate** — add it to `Cargo.toml` and enable
the features you need. All public items from sub-crates are re-exported.

## Feature flags

| Feature | Crate | Description |
|---|---|---|
| `core` | [`optic_core`] | Shared types, math, colours, logging |
| `file` | [`optic_file`] | Asset file I/O and cached paths |
| `render` | [`optic_render`] | OpenGL 4.6 rendering (EGL, shaders, meshes) |
| `window` | [`optic_window`] | Windowing, input, gamepad (winit + gilrs) |
| `minimal` | [`optic_loop`] | Game loop + [`Runtime`] trait |
| `online` | [`optic_online`] | UDP networking |

## Quick start

```toml
[dependencies]
optic-engine = { version = "0.0", features = ["minimal"] }
```

```rust,no_run
use optic_engine::{Game, Runtime};

struct App;
impl Runtime for App {
    fn update(&mut self) { /* game logic */ }
}

fn main() {
    Game::new("My Game", (800, 600), App).unwrap().run();
}
```

## Architecture

```text
┌──────────────┐
│ optic-engine │  facade (re-exports everything)
├──────────────┤
│  optic_core   │  base types, math, colour
│  optic_file   │  asset I/O
│  optic_render │  GPU / GL rendering
│  optic_window │  winit windowing + input
│  optic_loop   │  game loop + Runtime
│  optic_online │  UDP networking
└──────────────┘
```

[`optic_core`]: https://docs.rs/optic-core
[`optic_file`]: https://docs.rs/optic-file
[`optic_render`]: https://docs.rs/optic-render
[`optic_window`]: https://docs.rs/optic-window
[`optic_loop`]: https://docs.rs/optic-loop
[`optic_online`]: https://docs.rs/optic-online
[`Runtime`]: https://docs.rs/optic-loop/latest/optic_loop/runtime/trait.Runtime.html
