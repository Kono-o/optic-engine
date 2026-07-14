//! The game loop and runtime — drives the winit event loop and owns the GPU,
//! camera, windows, and timing.
//!
//! This crate provides two ergonomic levels for running an Optic application:
//!
//! # High-level API: [`Game`] + [`Runtime`]
//!
//! The [`Game`] struct owns all engine subsystems (GPU, camera, window, events,
//! time, gamepad). You implement the [`Runtime`] trait and pass it to
//! [`Game::run`]:
//!
//! ```ignore
//! use optic_loop::{Game, Runtime, FpsLimit};
//!
//! struct App;
//! impl Runtime for App {
//!     fn start(&mut self, game: &mut Game) {
//!         game.time.set_target_physics_rate(120.0);
//!         game.time.set_target_tps(Some(20.0));
//!         game.time.set_fps_limit(FpsLimit::Uncapped);
//!     }
//!     fn physics(&mut self, game: &mut Game) {
//!         // fixed-timestep simulation
//!     }
//!     fn update(&mut self, game: &mut Game) {
//!         // input, AI, gameplay
//!     }
//!     fn render(&mut self, game: &mut Game) {
//!         game.renderer.clear();
//!         // draw calls
//!     }
//!     fn end(&mut self, _game: &mut Game) {}
//! }
//!
//! Game::run(App);
//! ```
//!
//! # Low-level API: [`GameLoop`] + closure
//!
//! [`GameLoop`] takes a `FnMut(&mut FrameState)` closure and gives you more
//! control over setup. Use [`run`] for a quick single-window start:
//!
//! ```ignore
//! use optic_loop::run;
//!
//! run("My Window", (800, 600).into(), |frame| {
//!     frame.gpu.clear();
//!     // render things...
//! });
//! ```
//!
//! # How the Game Runtime Works
//!
//! The Optic game loop is a **three-phase, fixed-timestep frame scheduler**
//! built on top of winit's event loop. Each frame is divided into three
//! independent phases that run at their own configurable rates:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │  winit Event Loop                                            │
//! │                                                              │
//! │  ┌─── about_to_wait() ───────────────────────────────────┐  │
//! │  │                                                        │  │
//! │  │  1. Record frame start time                           │  │
//! │  │  2. Drain gamepad events (gilrs)                      │  │
//! │  │  3. Poll network events (if online feature enabled)   │  │
//! │  │  4. Make GL context current                           │  │
//! │  │  5. Clear GPU                                         │  │
//! │  │  6. Advance time (delta, FPS, elapsed)                │  │
//! │  │  7. Update window frame state (cursor delta, loopback)│  │
//! │  │  8. Recalculate camera matrices                       │  │
//! │  │                                                        │  │
//! │  │  ┌─ FIRST FRAME ONLY ─────────────────────────────┐  │  │
//! │  │  │  • runtime.start(game)                          │  │  │
//! │  │  │  • Make window visible                          │  │  │
//! │  │  │  • Center window on screen                      │  │  │
//! │  │  └─────────────────────────────────────────────────┘  │  │
//! │  │                                                        │  │
//! │  │  ┌─ Three-Phase Frame ────────────────────────────┐   │  │
//! │  │  │                                                │   │  │
//! │  │  │  Physics (fixed timestep, default 60 Hz)       │   │  │
//! │  │  │    ├ physics()  ← constant dt = 1/hz          │   │  │
//! │  │  │    ├ physics()  ← catch-up if frame was slow   │   │  │
//! │  │  │    └ physics()                                  │   │  │
//! │  │  │                                                │   │  │
//! │  │  │  Update (optional fixed timestep)              │   │  │
//! │  │  │    └ update()     ← once per frame by default  │   │  │
//! │  │  │                                                │   │  │
//! │  │  │  Render (once per presented frame)             │   │  │
//! │  │  │    └ render()     ← all draw calls go here     │   │  │
//! │  │  │                                                │   │  │
//! │  │  └────────────────────────────────────────────────┘   │  │
//! │  │                                                        │  │
//! │  │  9. Swap buffers (present to screen)                  │  │
//! │  │ 10. Clear input events for next frame                 │  │
//! │  │ 11. Request redraw                                    │  │
//! │  │ 12. Sleep if FPS-limited                              │  │
//! │  │                                                        │  │
//! │  └────────────────────────────────────────────────────────┘  │
//! │                                                              │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Frame Lifecycle
//!
//! Each rendered frame follows this exact sequence:
//!
//! 1. **Frame start** — the engine records the current wall-clock time
//!    for delta computation and FPS limiting.
//!
//! 2. **Event processing** — winit window events (resize, cursor, close)
//!    and gilrs gamepad events are drained into the [`Events`] collector.
//!    Network events are polled if the `online` feature is enabled.
//!
//! 3. **Time update** — [`Time::update`] computes `delta` (seconds since
//!    last frame), `elapsed` (total seconds), and smoothed `fps` from a
//!    32-frame rolling average.
//!
//! 4. **First-frame setup** — on the very first frame only, the engine
//!    calls [`Runtime::start`], makes the window visible, and centers it
//!    on the monitor. This is where you load assets and configure rates.
//!
//! 5. **Three-phase execution** — [`Game::run_frame`] drives the three
//!    phases:
//!
//!    - **Physics** — [`Time::advance_physics`] determines how many
//!      fixed-timestep steps to run. Each step invokes
//!      [`Runtime::physics`] with a constant delta (`1/physics_rate`).
//!      At 60 Hz, this is ~16.67 ms per step. If a frame takes 50 ms,
//!      three physics steps execute to catch up.
//!
//!    - **Update** — [`Time::advance_update`] determines how many update
//!      steps to run. By default (`None` TPS), this is exactly one step
//!      per frame. If you set a fixed TPS, multiple updates can run per
//!      frame, similar to physics.
//!
//!    - **Render** — [`Runtime::render`] is called exactly once. All draw
//!      calls belong here. Use [`Time::physics_alpha`] to interpolate
//!      between previous and current simulation state for smooth visuals
//!      when physics runs slower than rendering.
//!
//! 6. **Presentation** — the engine swaps buffers (presents to screen),
//!    clears the input event state for the next frame, and requests a
//!    redraw from winit.
//!
//! 7. **FPS limiting** — if [`FpsLimit::Limited`] is active, the engine
//!    sleeps for the remaining frame time to approximate the target FPS.
//!    VSync pacing is handled by the swap interval. Uncapped mode does
//!    no sleeping.
//!
//! ## Rate Configuration
//!
//! Each phase has an independently configurable rate:
//!
//! ```ignore
//! fn start(&mut self, game: &mut Game) {
//!     // Physics: 120 Hz fixed timestep (default: 60 Hz)
//!     game.time.set_target_physics_rate(120.0);
//!
//!     // Update: 20 Hz fixed timestep (default: once per frame)
//!     game.time.set_target_tps(Some(20.0));
//!
//!     // Render: 144 FPS cap (default: VSync)
//!     game.time.set_fps_limit(FpsLimit::Limited(144.0));
//! }
//! ```
//!
//! - **Physics rate** (`set_target_physics_rate`): controls how many
//!   simulation steps run per second. Each step uses a constant delta,
//!   making physics deterministic regardless of frame rate.
//!
//! - **Update rate** (`set_target_tps`): `None` means once per frame
//!   (the default). `Some(hz)` enables a fixed update timestep, useful
//!   for decoupling gameplay logic from render rate.
//!
//! - **FPS limit** (`set_fps_limit`): `VSync` syncs to monitor refresh,
//!   `Limited(fps)` sleeps to approximate a target, `Uncapped` renders
//!   as fast as possible.
//!
//! Rate changes take effect immediately — the next scheduler invocation
//! observes the new rate. Existing accumulator contents are preserved.
//!
//! ## Spiral-of-Death Protection
//!
//! If a frame takes too long (e.g. a garbage collection pause), the
//! physics/update accumulators could grow without bound, causing
//! ever-longer frames. The engine caps each phase at **240 steps per
//! frame**. Excess backlog is discarded with phase preservation
//! (`accumulator %= fixed_dt`) and a warning is logged. This prevents
//! the "spiral of death" where the game becomes permanently unresponsive.
//!
//! ## Interpolation
//!
//! When physics runs at a different rate than rendering, objects may
//! appear to stutter. The engine provides [`Time::physics_alpha`], a
//! value in `[0, 1)` that represents how far between the previous and
//! current physics step we are. Use it to lerp visual positions:
//!
//! ```ignore
//! fn render(&mut self, game: &mut Game) {
//!     let alpha = game.time.physics_alpha();
//!     let display_x = prev_x + (curr_x - prev_x) * alpha;
//!     // draw at (display_x, display_y)
//! }
//! ```
//!
//! The engine performs **no automatic interpolation** — you control
//! exactly how interpolation is applied.
//!
//! ## Ownership and Borrowing
//!
//! The engine uses a `take/replace` pattern to satisfy Rust's borrow
//! checker: the [`Runtime`] trait object is temporarily removed from
//! [`Game`] before each callback, then returned after. This means you
//! can borrow `game` mutably inside your callbacks without conflicts:
//!
//! ```ignore
//! fn update(&mut self, game: &mut Game) {
//!     // game is fully borrowed here — access any subsystem
//!     game.events.key(KeyCode::Space, Is::Pressed);
//!     game.renderer.set_bg_color(RED);
//!     game.audio.set_master_volume(0.8);
//! }
//! ```
//!
//! ## Shutdown
//!
//! The game exits when:
//! - [`Game::exit`] is called from any callback
//! - The window close button is pressed
//!
//! On shutdown, the engine calls [`Runtime::end`], then disconnects
//! networking (if enabled), and exits the process with code 0.

mod game;
mod runtime;
mod time;
pub use game::*;
pub use runtime::*;
pub use time::*;
pub use optic_timer::{Timer, Timers};

use gilrs::Gilrs;
use optic_core::{log_error, CamProj, Coord2D, OpticResult, Size2D};
use optic_render::{Camera, GPU};
use optic_window::{Events, Window};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

/// Pairs a Window with its Events collector and GPU surface index for the game loop.
///
/// Used by GameLoop to manage multiple windows. Each WindowState owns the window handle, the
/// event buffer for that window, and the index of its surface within the GPU context.
pub struct WindowState {
    pub window: Window,
    pub events: Events,
    pub surface_index: usize,
}

impl WindowState {
    /// Creates a new window and registers it with the event loop.
    pub fn new(el: &EventLoop<()>, title: &str, size: Size2D) -> Self {
        Self {
            window: Window::new(el, title, size),
            events: Events::new(),
            surface_index: 0,
        }
    }

    /// Closes the underlying window.
    pub fn close(&mut self) {
        self.window.close();
    }

    /// Returns `true` if the window has been closed.
    pub fn is_closed(&self) -> bool {
        self.window.is_closed()
    }

    /// Returns the GPU surface index for this window.
    pub fn surface_index(&self) -> usize {
        self.surface_index
    }
}

/// Borrowed bundle of engine state passed to per-frame closures.
///
/// Contains borrows of Time, windows, GPU, and Camera that the user's closure accesses during
/// each frame callback in the GameLoop API. This avoids cloning or wrapping individual subsystems.
pub struct FrameState<'a> {
    pub time: &'a Time,
    pub windows: &'a mut [WindowState],
    pub gpu: &'a mut GPU,
    pub camera: &'a mut Camera,
}

/// Low-level closure-based game loop for single or multi-window applications.
///
/// Owns the event loop, windows, GPU, camera, and timing. Takes a FnMut(&mut FrameState) closure
/// invoked every frame. This is the lower-level alternative to Game + Runtime, useful when you
/// need more control over setup or multiple windows.
pub struct GameLoop<F: FnMut(&mut FrameState)> {
    event_loop: Option<EventLoop<()>>,
    windows: Vec<WindowState>,
    gpu: Option<GPU>,
    camera: Camera,
    time: Time,
    gilrs: Gilrs,
    frame_fn: F,
}

impl<F: FnMut(&mut FrameState)> GameLoop<F> {
    /// Constructs a new game loop.
    ///
    /// # Errors
    ///
    /// Returns [`OpticError`](optic_core::OpticError) if:
    /// - Any window's raw handle cannot be obtained.
    /// - Attaching a window surface to the GPU context fails.
    /// - The gamepad subsystem (`gilrs`) fails to initialise.
    pub fn new(
        el: EventLoop<()>,
        mut gpu: GPU,
        camera: Camera,
        mut windows: Vec<WindowState>,
        frame_fn: F,
    ) -> OpticResult<Self> {
        for ws in windows.iter_mut() {
            if let Some(handle) = ws.window.raw_handle() {
                let size = ws.window.size();
                let idx = gpu.ctx_mut().attach_window(handle, size)
                    .map_err(|e| optic_core::OpticError::custom(&format!("attach window failed: {e}")))?;
                ws.surface_index = idx;
            }
        }

        let gilrs = Gilrs::new()
            .map_err(|e| optic_core::OpticError::custom(&format!("gilrs init failed: {e}")))?;

        Ok(Self {
            event_loop: Some(el),
            windows,
            gpu: Some(gpu),
            camera,
            time: Time::new(),
            gilrs,
            frame_fn,
        })
    }

    /// Starts the event loop, consuming `self`.
    pub fn run(mut self) {
        let el = self.event_loop.take().unwrap();
        let _ = el.run_app(&mut self);
    }
}

impl<F: FnMut(&mut FrameState)> ApplicationHandler for GameLoop<F> {
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
        for ws in &mut self.windows {
            if !ws.window.is_running() { continue; }
            if ws.window.id().map_or(true, |wid| wid != id) { continue; }

            match &event {
                WindowEvent::Resized(_size) => {
                    if let Some(gpu) = &mut self.gpu {
                        gpu.ctx_mut().resize_window(ws.surface_index, ws.window.size());
                        let _ = gpu.ctx().make_current(ws.surface_index);
                        self.camera.set_size(ws.window.size());
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    ws.window.notify_cursor_moved(Coord2D::new(position.x, position.y));
                }
                WindowEvent::CursorEntered { .. } => {
                    ws.window.notify_cursor_inside(true);
                }
                WindowEvent::CursorLeft { .. } => {
                    ws.window.notify_cursor_inside(false);
                }
                WindowEvent::CloseRequested => {
                    ws.events.close_requested = true;
                }
                _ => {}
            }
            ws.events.process_window_event(&event, &ws.window);
            break;
        }
    }

    fn about_to_wait(&mut self, _el: &ActiveEventLoop) {
        let gpu = match &mut self.gpu {
            Some(g) => g,
            None => return,
        };

        self.windows.retain(|ws| !ws.window.is_closed());

        if self.windows.is_empty() {
            return;
        }

        // Record frame start for FPS limiting
        self.time.begin_frame();

        while let Some(gilrs_event) = self.gilrs.next_event() {
            for ws in &mut self.windows {
                ws.events.process_gilrs_event(&gilrs_event);
            }
        }

        self.time.update();

        {
            let mut frame = FrameState {
                time: &self.time,
                windows: &mut self.windows,
                gpu,
                camera: &mut self.camera,
            };
            (self.frame_fn)(&mut frame);
        }

        for ws in &mut self.windows {
            ws.events.end_frame();
        }

        for ws in &mut self.windows {
            ws.window.request_redraw();
        }

        // FPS limiter
        match self.time.fps_limit() {
            FpsLimit::Uncapped => {}
            FpsLimit::VSync => {}
            FpsLimit::Limited(target_fps) => {
                if let Some(target_frame_time) = FpsLimit::Limited(*target_fps).target_frame_time() {
                    let elapsed = self.time.frame_elapsed();
                    if elapsed < target_frame_time {
                        self.time.sleep(target_frame_time - elapsed);
                    }
                }
            }
        }
    }
}

/// Runs a single-window application with a per-frame closure.
pub fn run<F>(title: &str, size: Size2D, frame_fn: F)
where
    F: FnMut(&mut FrameState) + 'static,
{
    let result = try_run(title, size, frame_fn);
    if let Err(e) = result {
        log_error!("{}", e);
        optic_core::end(optic_core::ERROR);
    }
}

fn try_run<F>(title: &str, size: Size2D, frame_fn: F) -> OpticResult<()>
where
    F: FnMut(&mut FrameState) + 'static,
{
    let el = EventLoop::new()
        .map_err(|e| optic_core::OpticError::custom(&format!("event loop creation failed: {e}")))?;
    let ws = WindowState::new(&el, title, size);
    let handle = ws.window.raw_handle()
        .ok_or_else(|| optic_core::OpticError::custom("failed to get raw window handle"))?;
    let display_handle = ws.window.raw_display_handle()
        .ok_or_else(|| optic_core::OpticError::custom("failed to get raw display handle"))?;
    let gpu = GPU::new_windowed(handle, display_handle, ws.window.size())
        .map_err(|e| optic_core::OpticError::custom(&format!("gpu init failed: {e}")))?;
    let camera = Camera::new(ws.window.size(), CamProj::Persp);
    let game = GameLoop::new(el, gpu, camera, vec![ws], frame_fn)?;
    game.run();
    Ok(())
}
