# optic-window

Windowing and input for the Optic engine — winit wrapper with per-frame
key, mouse, and gamepad state.

Provides a [`Window`] handle with cursor tracking (grab, confine, loopback),
and an [`Events`] accumulator that collects winit + gilrs events each frame
for polling-style input queries.

```rust
use optic_window::{Window, Events};
use optic_core::Size2D;

let el = winit::event_loop::EventLoop::new().unwrap();
let mut window = Window::new(&el, "My App", Size2D::from(800, 600));
let mut events = Events::new();
```

[`Window`]: https://docs.rs/optic-window/latest/optic_window/window/struct.Window.html
[`Events`]: https://docs.rs/optic-window/latest/optic_window/events/struct.Events.html
