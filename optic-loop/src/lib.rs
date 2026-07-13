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
//! use optic_loop::{Game, Runtime};
//!
//! struct App;
//! impl Runtime for App {
//!     fn start(&mut self, _game: &mut Game) {}
//!     fn update(&mut self, game: &mut Game) {
//!         game.renderer.clear();
//!         // render things...
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
//! # Frame timing
//!
//! Both APIs update [`Time`] automatically each frame. Access delta time and
//! FPS through the [`FrameState`] (low-level) or `game.time` (high-level).

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

/// A single window and its associated event sink and GPU surface index.
///
/// Used by [`GameLoop`] to manage multiple windows. Each `WindowState`
/// owns a [`Window`], an [`Events`] collector, and the index of its
/// surface within the GPU's context.
///
/// # Example
///
/// ```ignore
/// let ws = WindowState::new(&event_loop, "My Window", (800, 600).into());
/// ```
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

/// A snapshot of per-frame mutable state, passed to the user's closure.
///
/// Contains borrows of the engine subsystems that the user may access
/// during a frame callback in the [`GameLoop`] API.
pub struct FrameState<'a> {
    pub time: &'a Time,
    pub windows: &'a mut [WindowState],
    pub gpu: &'a mut GPU,
    pub camera: &'a mut Camera,
}

/// A low-level game loop that drives a closure once per frame.
///
/// Owns the event loop, one or more windows, the GPU, a camera, timing,
/// and gamepad state. The user provides a `FnMut(&mut FrameState)` closure
/// that is invoked every frame.
///
/// This is the lower-level alternative to [`Game`] + [`Runtime`]. Use it
/// when you want more control over the setup process or need multiple
/// windows.
///
/// # Example
///
/// ```ignore
/// use optic_loop::{GameLoop, WindowState};
///
/// let el = EventLoop::new().unwrap();
/// let ws = WindowState::new(&el, "App", (800, 600).into());
/// let gpu = GPU::new_headless()?;
/// let camera = Camera::new((800, 600).into(), CamProj::Persp);
///
/// let game = GameLoop::new(el, gpu, camera, vec![ws], |frame| {
///     frame.gpu.clear();
/// })?;
/// game.run();
/// ```
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
    /// Attaches each window's raw handle to the GPU context and initialises
    /// gamepad support via `gilrs`.
    ///
    /// # Errors
    ///
    /// Returns an error if window attachment to the GPU surface or gamepad
    /// initialisation fails.
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
    ///
    /// This call blocks until all windows are closed or the application
    /// exits.
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
    }
}

/// Runs a single-window application with a per-frame closure.
///
/// This is the simplest way to get a window on screen. On error, the
/// error is logged and the process exits with `ERROR`.
///
/// # Example
///
/// ```ignore
/// use optic_loop::run;
///
/// run("Hello", (800, 600).into(), |frame| {
///     frame.gpu.clear();
/// });
/// ```
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
