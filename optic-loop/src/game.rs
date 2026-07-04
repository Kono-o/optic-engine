/// The high-level [`Game`] — owns all engine subsystems and drives a
/// [`Runtime`] implementation through the winit event loop.
///
/// # Architecture
///
/// `Game` aggregates every subsystem the engine provides:
///
/// | Field | Type | Purpose |
/// |---|---|---|
/// | `renderer` | [`GPU`] | GL context, pipeline state, fallback assets |
/// | `camera` | [`Camera`] | Active view/projection |
/// | `events` | [`Events`] | Per-frame input collection |
/// | `time` | [`Time`] | Delta time, FPS, elapsed |
/// | `window` | [`Window`] | Application window |
///
/// # Lifecycle
///
/// ```text
/// Game::new(runtime) ──► Game::run(runtime)
///                            │
///                            ▼
///                     ┌─► Runtime::start  (once)
///                     │        │
///                     │        ▼
///                     │   Runtime::update  (every frame)
///                     │        │
///                     └────────┘
///                            │
///                            ▼
///                     Runtime::end  (on shutdown)
/// ```
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
///         // Load assets, set up scene
///         game.renderer.set_bg_color((0.1, 0.2, 0.3, 1.0).into());
///     }
///
///     fn update(&mut self, game: &mut Game) {
///         // Per-frame logic
///         game.renderer.clear();
///         // ... draw calls ...
///     }
///
///     fn end(&mut self, _game: &mut Game) {
///         // Save state, disconnect
///     }
/// }
///
/// Game::run(App);
/// ```

use gilrs::Gilrs;
use optic_core::{log_error, CamProj, Coord2D, OpticResult, Size2D, CRIMSON};
use optic_core::{end, end_success, ERROR, SUCCESS};
use optic_render::{Camera, GPU};
use optic_window::{Events, Window};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

#[cfg(feature = "online")]
use optic_online::NetworkHandle;

use crate::{Runtime, Time};

/// The primary game object — aggregates the renderer, camera, window, events,
/// timing, gamepad, and user-provided [`Runtime`].
///
/// Create via [`Game::new`] and start via [`Game::run`]. All fields are public
/// so that [`Runtime`] methods can access them directly.
pub struct Game {
    pub renderer: GPU,
    pub camera: Camera,
    pub events: Events,
    pub time: Time,
    pub window: Window,

    event_loop: Option<EventLoop<()>>,
    surface_index: usize,
    gilrs: Gilrs,
    runtime: Option<Box<dyn Runtime>>,
    running: bool,
    started: bool,
    requested_size: Size2D,
    resized_once: bool,

    #[cfg(feature = "online")]
    pub(crate) network: Option<NetworkHandle>,
}

impl Game {
    /// Creates a new game with a 500×500 window and a crimson background.
    ///
    /// Initialises:
    ///
    /// - A [`GPU`] with VSync enabled and the given background colour
    /// - A perspective [`Camera`]
    /// - Gamepad input via `gilrs`
    /// - The user's [`Runtime`] implementation
    ///
    /// # Errors
    ///
    /// Returns an error if the window, EGL/GLX surface, or gamepad cannot
    /// be initialised.
    ///
    /// ```ignore
    /// let game = Game::new(MyRuntime)?;
    /// ```
    pub fn new<R: Runtime + 'static>(runtime: R) -> OpticResult<Game> {
        let size = Size2D::from(500,500);
        let bg_color = CRIMSON;
        let title = "OPTIC GAME";
        let el = EventLoop::builder()
           .build()
           .map_err(|e| optic_core::OpticError::custom(&format!("event loop creation failed: {e}")))?;
        let window = Window::new(&el, title, size);
        let actual_size = window.size();
        let handle = window.raw_handle()
            .ok_or_else(|| optic_core::OpticError::custom("failed to get raw window handle"))?;
        let display_handle = window.raw_display_handle()
            .ok_or_else(|| optic_core::OpticError::custom("failed to get raw display handle"))?;

        let mut gpu = GPU::new_windowed(handle, display_handle, actual_size)?;
        gpu.ctx.set_vsync(true);
        gpu.canvas_size = actual_size;
        gpu.set_bg_color(bg_color);
        let surface_index = 0;

        let gilrs = Gilrs::new()
            .map_err(|e| optic_core::OpticError::custom(&format!("gilrs init failed: {e}")))?;
        Ok(Game {
            renderer: gpu,
            camera: Camera::new(size, CamProj::Persp),
            events: Events::new(),
            time: Time::new(),
            event_loop: Some(el),
            window,
            surface_index,
            gilrs,
            runtime: Some(Box::new(runtime)),
            running: true,
            started: false,
            requested_size: size,
            resized_once: false,
            #[cfg(feature = "online")]
            network: None,
        })
    }

    /// Convenience entry point: creates a [`Game`] and runs the event loop.
    ///
    /// On success exits with `SUCCESS`; on failure logs and exits with
    /// `ERROR`.
    ///
    /// This is the simplest way to start an Optic application:
    ///
    /// ```ignore
    /// Game::run(MyRuntime);
    /// ```
    pub fn run<R: Runtime + 'static>(runtime: R) {
        match Game::new(runtime) {
            Ok(game) => {
                game.start();
                end(SUCCESS);
            }
            Err(e) => {
                log_error!("{}", e);
                end(ERROR);
            }
        }
    }

    fn start(mut self) {
        let el = self.event_loop.take().unwrap();
        let _ = el.run_app(&mut self);
    }

    /// Signals the game loop to exit gracefully on the next frame.
    ///
    /// After calling this, [`Runtime::end`] will be invoked and the process
    /// will exit with `SUCCESS`.
    ///
    /// ```ignore
    /// // In your runtime:
    /// fn update(&mut self, game: &mut Game) {
    ///     if game.events.key_down(VirtualKeyCode::Escape) {
    ///         game.exit();
    ///     }
    /// }
    /// ```
    pub fn exit(&mut self) {
        self.running = false;
    }

    /// Returns a reference to the [`NetworkHandle`] if networking is enabled.
    #[cfg(feature = "online")]
    pub fn network(&self) -> Option<&NetworkHandle> {
        self.network.as_ref()
    }

    /// Returns a mutable reference to the [`NetworkHandle`] if networking is enabled.
    #[cfg(feature = "online")]
    pub fn network_mut(&mut self) -> Option<&mut NetworkHandle> {
        self.network.as_mut()
    }

    /// Enables networking with the given configuration.
    ///
    /// Spawns a background network thread. Call early in [`Runtime::start`]
    /// before any network-dependent logic runs.
    ///
    /// # Errors
    ///
    /// Returns an error if the connection cannot be established.
    #[cfg(feature = "online")]
    pub fn enable_networking(&mut self, config: optic_core::NetworkConfig) -> OpticResult<()> {
        let handle = NetworkHandle::new(config)?;
        self.network = Some(handle);
        Ok(())
    }

}

impl ApplicationHandler for Game {
    fn resumed(&mut self, _el: &ActiveEventLoop) {
        self.time.start_time = std::time::Instant::now();
        self.time.prev_time = std::time::Instant::now();
        self.time.prev_sec = std::time::Instant::now();
    }

    fn window_event(
        &mut self,
        _el: &ActiveEventLoop,
        id: WindowId,
        event: WindowEvent,
    ) {
        if !self.window.is_running() { return; }
        if self.window.id().unwrap() != id { return; }

        match &event {
            WindowEvent::Resized(_size) => {
                self.renderer.ctx.resize_window(self.surface_index, self.window.size());
                let _ = self.renderer.ctx.make_current(self.surface_index);
                self.renderer.canvas_size = self.window.size();
                self.camera.set_size(self.window.size());
                if !self.resized_once && (_size.width != self.requested_size.w || _size.height != self.requested_size.h) {
                    self.resized_once = true;
                    self.window.set_size(self.requested_size);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.window.notify_cursor_moved(Coord2D::from(position.x, position.y));
            }
            WindowEvent::CursorEntered { .. } => {
                self.window.notify_cursor_inside(true);
            }
            WindowEvent::CursorLeft { .. } => {
                self.window.notify_cursor_inside(false);
            }
            WindowEvent::CloseRequested => {
                self.events.close_requested = true;
            }
            _ => {}
        }
        self.events.process_window_event(&event, &self.window);
    }

    fn about_to_wait(&mut self, el: &ActiveEventLoop) {
        if !self.running || self.window.is_closed() {
            #[cfg(feature = "online")]
            if let Some(mut net) = self.network.take() {
                net.shutdown();
            }
            if let Some(mut runtime) = self.runtime.take() {
                runtime.end(self);
            }
            end_success();
            el.exit();
            return;
        }

        while let Some(gilrs_event) = self.gilrs.next_event() {
            self.events.process_gilrs_event(&gilrs_event);
        }

        #[cfg(feature = "online")]
        if let Some(net) = &mut self.network {
            net.poll(&mut self.events.network);
        }

        let _ = self.renderer.ctx.make_current(self.surface_index);
        self.renderer.clear();
        self.time.update();

        self.window.update_frame();

        self.camera.pre_update();

        let mut runtime = self.runtime.take().unwrap();
        if !self.started {
            runtime.start(self);
            self.started = true;
            self.window.set_visible(true);
            self.window.center_on_screen();
        }
        runtime.update(self);
        self.runtime = Some(runtime);

        let _ = self.renderer.ctx.swap_buffers(self.surface_index);
        self.events.end_frame();
        self.window.request_redraw();
    }
}
