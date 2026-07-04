# optic-loop

Game loop, [`Runtime`] trait, and [`Game`] builder for the Optic engine.

Wires together windowing, rendering, and networking into a single run loop.
Implement the [`Runtime`] trait to receive `setup`, `update`, `fixed_update`,
`render`, and `close` callbacks.

```rust
use optic_loop::{Game, Runtime};
use optic_core::OpticResult;

struct MyGame;
impl Runtime for MyGame {
    fn update(&mut self) { /* per-frame logic */ }
}

Game::new("My Game", (800, 600), MyGame)?.run();
```

[`Runtime`]: https://docs.rs/optic-loop/latest/optic_loop/runtime/trait.Runtime.html
[`Game`]: https://docs.rs/optic-loop/latest/optic_loop/game/struct.Game.html
