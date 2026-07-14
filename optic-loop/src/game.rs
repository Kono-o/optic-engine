use gilrs::Gilrs;
use optic_core::{log_error, CamProj, Coord2D, OpticResult, Size2D, CRIMSON};
use optic_core::{end, end_success, ERROR, SUCCESS};
use optic_render::{Camera, GPU};
use optic_sound::AudioEngine;
use optic_window::{Events, Window};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

#[cfg(feature = "online")]
use optic_online::NetworkHandle;

use crate::{FpsLimit, Runtime, Time};

/// The primary game object — aggregates the renderer, camera, window, events,
/// timing, gamepad, audio, and user-provided [`Runtime`].
///
/// Create via [`Game::new`] and start via [`Game::run`]. All fields are public
/// so that [`Runtime`] methods can access them directly.
///
/// # Execution model
///
/// Each frame executes in three independent phases:
///
/// 1. **Physics** — fixed-timestep simulation (default 60 Hz)
/// 2. **Update** — gameplay logic (default: once per frame)
/// 3. **Render** — draw calls, presented once per frame
///
/// Each phase runs at its own independently configurable rate via
/// [`Time::set_target_physics_rate`], [`Time::set_target_tps`], and
/// [`Time::set_fps_limit`].
pub struct Game {
    pub renderer: GPU,
    pub camera: Camera,
    pub events: Events,
    pub time: Time,
    pub window: Window,
    pub audio: AudioEngine,

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
    /// Creates a new game with a 500x500 window and a crimson background.
    pub fn new<R: Runtime + 'static>(runtime: R) -> OpticResult<Game> {
        let size = Size2D::new(500,500);
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
        gpu.ctx().set_vsync(true);
        gpu.set_canvas_size(actual_size);
        gpu.set_bg_color(bg_color);
        let surface_index = 0;

        let gilrs = Gilrs::new()
            .map_err(|e| optic_core::OpticError::custom(&format!("gilrs init failed: {e}")))?;
        let audio = AudioEngine::new()?;
        Ok(Game {
            renderer: gpu,
            camera: Camera::new(size, CamProj::Persp),
            events: Events::new(),
            time: Time::new(),
            audio,
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
    pub fn exit(&mut self) {
        self.running = false;
    }

    /// Returns a read-only reference to the [`NetworkHandle`], if networking
    /// has been enabled.
    #[cfg(feature = "online")]
    pub fn network(&self) -> Option<&NetworkHandle> {
        self.network.as_ref()
    }

    /// Returns a mutable reference to the [`NetworkHandle`], if networking
    /// has been enabled.
    #[cfg(feature = "online")]
    pub fn network_mut(&mut self) -> Option<&mut NetworkHandle> {
        self.network.as_mut()
    }

    /// Initialises the networking subsystem with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`OpticError`](optic_core::OpticError) if the underlying
    /// [`NetworkHandle`] fails to create (e.g. bind or connect failure).
    #[cfg(feature = "online")]
    pub fn enable_networking(&mut self, config: optic_core::NetworkConfig) -> OpticResult<()> {
        let handle = NetworkHandle::new(config)?;
        self.network = Some(handle);
        Ok(())
    }

    /// Execute the three-phase frame: physics, update, render.
    fn run_frame(&mut self, frame_delta: f64) {
        // ----------------------------------------------------------------
        // Physics
        // ----------------------------------------------------------------
        let physics_steps = self.time.advance_physics(frame_delta);
        for _ in 0..physics_steps {
            let mut runtime = self.runtime.take().unwrap();
            runtime.physics(self);
            self.runtime = Some(runtime);
        }

        // ----------------------------------------------------------------
        // Update
        // ----------------------------------------------------------------
        let update_steps = self.time.advance_update(frame_delta);
        for _ in 0..update_steps {
            let mut runtime = self.runtime.take().unwrap();
            runtime.update(self);
            self.runtime = Some(runtime);
        }

        // ----------------------------------------------------------------
        // Render
        // ----------------------------------------------------------------
        let mut runtime = self.runtime.take().unwrap();
        runtime.render(self);
        self.runtime = Some(runtime);
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
                self.renderer.ctx_mut().resize_window(self.surface_index, self.window.size());
                let _ = self.renderer.ctx().make_current(self.surface_index);
                self.renderer.set_canvas_size(self.window.size());
                self.camera.set_size(self.window.size());
                if !self.resized_once && (_size.width != self.requested_size.w || _size.height != self.requested_size.h) {
                    self.resized_once = true;
                    self.window.set_size(self.requested_size);
                }
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.window.notify_cursor_moved(Coord2D::new(position.x, position.y));
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

        // Record frame start for FPS limiting
        self.time.begin_frame();

        while let Some(gilrs_event) = self.gilrs.next_event() {
            self.events.process_gilrs_event(&gilrs_event);
        }

        #[cfg(feature = "online")]
        if let Some(net) = &mut self.network {
            net.poll(&mut self.events.network);
        }

        let _ = self.renderer.ctx().make_current(self.surface_index);
        self.renderer.clear();
        self.time.update();

        self.window.update_frame();

        self.camera.pre_update();

        // First frame: run start() and show window
        if !self.started {
            let mut runtime = self.runtime.take().unwrap();
            runtime.start(self);
            self.runtime = Some(runtime);
            self.started = true;
            self.window.set_visible(true);
            self.window.center_on_screen();
        }

        let frame_delta = self.time.delta();

        // Three-phase frame: physics → update → render
        self.run_frame(frame_delta);

        // Present
        let _ = self.renderer.ctx().swap_buffers(self.surface_index);
        self.events.end_frame();
        self.window.request_redraw();

        // FPS limiter
        match self.time.fps_limit() {
            FpsLimit::Uncapped => {}
            FpsLimit::VSync => {
                // swap interval already performed pacing
            }
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
