mod game;
mod runtime;
mod time;

pub use game::*;
pub use runtime::*;
pub use time::*;

use gilrs::Gilrs;
use optic_core::{log_error, CamProj, Coord2D, OpticResult, Size2D};
use optic_render::{Camera, GPU};
use optic_window::{Events, Window};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

pub struct WindowState {
    pub window: Window,
    pub events: Events,
    pub surface_index: usize,
}

impl WindowState {
    pub fn new(el: &EventLoop<()>, title: &str, size: Size2D) -> Self {
        Self {
            window: Window::new(el, title, size),
            events: Events::new(),
            surface_index: 0,
        }
    }

    pub fn close(&mut self) {
        self.window.close();
    }

    pub fn is_closed(&self) -> bool {
        self.window.is_closed()
    }

    pub fn surface_index(&self) -> usize {
        self.surface_index
    }
}

pub struct FrameState<'a> {
    pub time: &'a Time,
    pub windows: &'a mut [WindowState],
    pub gpu: &'a mut GPU,
    pub camera: &'a mut Camera,
}

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
                let idx = gpu.ctx.attach_window(handle, size)
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
                        gpu.ctx.resize_window(ws.surface_index, ws.window.size());
                        let _ = gpu.ctx.make_current(ws.surface_index);
                        self.camera.set_size(ws.window.size());
                    }
                }
                WindowEvent::CursorMoved { position, .. } => {
                    ws.window.notify_cursor_moved(Coord2D::from(position.x, position.y));
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

        // Remove closed windows
        self.windows.retain(|ws| !ws.window.is_closed());

        if self.windows.is_empty() {
            return;
        }

        // Poll gamepad events
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

/// Simple single-window entry point
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
