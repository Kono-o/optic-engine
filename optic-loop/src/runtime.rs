use crate::Game;

/// The lifecycle trait for game logic.
///
/// Implement this trait to hook your application into the engine's event
/// loop. Pass an instance to [`Game::new`] or [`Game::run`] and the engine
/// drives the three lifecycle methods automatically.
///
/// | Method | Called | Purpose |
/// |---|---|---|
/// | [`start`](Runtime::start) | Once, before the first frame | One-time setup |
/// | [`update`](Runtime::update) | Every frame | Input, logic, rendering |
/// | [`end`](Runtime::end) | Once, on shutdown | Cleanup |
///
/// Each method receives `&mut Game`, giving you access to the renderer,
/// camera, window, events, and time.
///
/// # Example
///
/// ```ignore
/// use optic_loop::{Game, Runtime};
///
/// struct FpsCounter {
///     frames: u64,
/// }
///
/// impl Runtime for FpsCounter {
///     fn start(&mut self, _game: &mut Game) {
///         println!("Starting...");
///     }
///
///     fn update(&mut self, game: &mut Game) {
///         self.frames += 1;
///         if self.frames % 60 == 0 {
///             let fps = game.time.fps();
///             println!("FPS: {:.1}", fps);
///         }
///         game.renderer.clear();
///         // ... draw scene ...
///     }
///
///     fn end(&mut self, _game: &mut Game) {
///         println!("Rendered {} frames total", self.frames);
///     }
/// }
///
/// Game::run(FpsCounter { frames: 0 });
/// ```
///
/// # Why three methods?
///
/// Separating `start` from `update` avoids re-initialising assets every
/// frame. Separating `end` from `drop` gives you a predictable point to
/// save state before the engine tears down subsystems.
pub trait Runtime {
    /// Called once before the first frame.
    ///
    /// Use this for one-time initialisation:
    ///
    /// - Load meshes, textures, and shaders via [`GPU::ship_*`]
    /// - Set up initial game state
    /// - Connect to a server (see [`Game::enable_networking`])
    ///
    /// The window is not yet visible when `start` runs. The engine calls
    /// `start`, makes the window visible, then immediately enters the
    /// update loop.
    fn start(&mut self, game: &mut Game);

    /// Called every frame.
    ///
    /// This is the main game loop body. Do all per-frame work here:
    ///
    /// 1. **Poll input** — check `game.events` for keyboard, mouse, gamepad
    /// 2. **Update logic** — move entities, run physics, check collisions
    /// 3. **Render** — clear the screen, bind shaders, draw meshes
    ///
    /// Use [`Game::exit`] to stop the loop from within `update`.
    fn update(&mut self, game: &mut Game);

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
