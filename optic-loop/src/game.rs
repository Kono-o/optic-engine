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

    pub fn exit(&mut self) {
        self.running = false;
    }

    /// Returns a reference to the `NetworkHandle` if networking was enabled.
    #[cfg(feature = "online")]
    pub fn network(&self) -> Option<&NetworkHandle> {
        self.network.as_ref()
    }

    /// Returns a mutable reference to the `NetworkHandle` if networking was enabled.
    #[cfg(feature = "online")]
    pub fn network_mut(&mut self) -> Option<&mut NetworkHandle> {
        self.network.as_mut()
    }

    /// Enables networking with the given config, spawning the network thread.
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
