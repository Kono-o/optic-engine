use crate::Game;

/// The lifecycle trait for game logic.
///
/// Implement this trait to hook your application into the engine's event
/// loop. Pass an instance to [`Game::new`] or [`Game::run`] and the engine
/// drives the lifecycle methods automatically.
///
/// | Method | Called | Purpose |
/// |---|---|---|
/// | [`start`](Runtime::start) | Once, before the first frame | One-time setup |
/// | [`physics`](Runtime::physics) | Zero or more times per frame | Fixed-timestep simulation |
/// | [`update`](Runtime::update) | According to target TPS | Input, AI, gameplay logic |
/// | [`render`](Runtime::render) | Once per presented frame | Draw calls |
/// | [`end`](Runtime::end) | Once, on shutdown | Cleanup |
///
/// # Example
///
/// ```ignore
/// use optic_loop::{Game, Runtime};
///
/// struct App;
///
/// impl Runtime for App {
///     fn start(&mut self, game: &mut Game) {
///         game.time.set_target_physics_rate(120.0);
///         game.time.set_target_tps(Some(20.0));
///     }
///
///     fn physics(&mut self, game: &mut Game) {
///         let dt = game.time.physics_delta();
///         // move entities, collide, integrate...
///     }
///
///     fn update(&mut self, game: &mut Game) {
///         // input, AI, gameplay state...
///     }
///
///     fn render(&mut self, game: &mut Game) {
///         game.renderer.clear();
///         // draw calls...
///     }
///
///     fn end(&mut self, _game: &mut Game) {}
/// }
///
/// Game::run(App);
/// ```
///
/// # Migration note
///
/// Draw calls that previously lived in `update()` **must** be moved to
/// `render()`. The engine no longer presents a frame between `update()`
/// calls — rendering happens in its own stage after all updates complete.
pub trait Runtime {
    /// Called once before the first frame.
    ///
    /// Use this for one-time initialisation:
    ///
    /// - Load meshes, textures, and shaders via `GPU::upload_*` methods
    /// - Set up initial game state
    /// - Configure physics/update/render rates via `game.time`
    /// - Connect to a server (see `Game::enable_networking`, requires `online` feature)
    ///
    /// The window is not yet visible when `start` runs. The engine calls
    /// `start`, makes the window visible, then immediately enters the
    /// main loop.
    fn start(&mut self, game: &mut Game);

    /// Fixed-timestep simulation.
    ///
    /// Called zero or more times every rendered frame, according to the
    /// physics rate configured via [`Time::set_target_physics_rate`](crate::Time::set_target_physics_rate).
    ///
    /// Uses a constant timestep equal to [`Time::physics_delta`](crate::Time::physics_delta).
    /// If a frame takes longer than expected, multiple physics callbacks
    /// execute to catch up, up to a configurable maximum (default 240).
    ///
    /// Intended for:
    /// - Movement integration
    /// - Collision detection / resolution
    /// - Deterministic simulation
    /// - Animation advancement
    ///
    /// The engine intentionally does not provide a built-in physics
    /// engine. This callback is simply the correctly timed place for
    /// user-written simulation.
    fn physics(&mut self, _game: &mut Game) {}

    /// General gameplay logic.
    ///
    /// Called according to the target TPS configured via
    /// [`Time::set_target_tps`](crate::Time::set_target_tps).
    ///
    /// If TPS is `None` (the default), this executes once every rendered
    /// frame, preserving today's update behaviour.
    ///
    /// Intended for:
    /// - Input handling
    /// - AI
    /// - Scripting
    /// - Gameplay state
    ///
    /// **Rendering should NOT occur here.** Use [`render`](Runtime::render)
    /// for draw calls.
    fn update(&mut self, game: &mut Game);

    /// Rendering.
    ///
    /// Called exactly once for every presented frame, after all physics and
    /// update callbacks for that frame have completed.
    ///
    /// Issue every draw call here. Use
    /// [`physics_alpha`](crate::Time::physics_alpha) to interpolate between
    /// previous and current simulation state for smooth visual presentation
    /// when physics runs slower than rendering.
    ///
    /// The engine performs no automatic interpolation.
    fn render(&mut self, _game: &mut Game) {}

    /// Called once on shutdown.
    ///
    /// Triggered by [`Game::exit`], window close, or `SIGINT`. Use this to:
    ///
    /// - Save persistent state (scores, settings)
    /// - Disconnect from servers
    /// - Release non-GPU resources
    ///
    /// The GPU and window are still alive during `end`; they are destroyed
    /// after it returns.
    fn end(&mut self, game: &mut Game);
}
