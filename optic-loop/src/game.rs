use optic_core::{CamProj, OpticResult, RGBA, Size2D};
use optic_render::GPU;
use optic_window::{Events, Window};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::WindowId;

use crate::{Runtime, Scene, Time};

pub struct GameBuilder {
    title: String,
    size: Size2D,
    bg_color: RGBA,
}

impl GameBuilder {
    pub fn new() -> Self {
        Self {
            title: "Optic Game".into(),
            size: Size2D::from(800, 600),
            bg_color: RGBA::grey(0.5),
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn with_size(mut self, size: Size2D) -> Self {
        self.size = size;
        self
    }

    pub fn build<R: Runtime + 'static>(self, runtime: R) -> OpticResult<Game> {
        let el = EventLoop::new()
            .map_err(|e| optic_core::OpticError::custom(&format!("event loop creation failed: {e}")))?;
        let window = Window::new(&el, &self.title, self.size);
        let handle = window.raw_handle().unwrap();

        let mut gpu = GPU::new_headless()?;
        let surface_index = gpu.ctx.attach_window(handle, self.size)?;
        gpu.ctx.make_current(surface_index)?;
        gpu.ctx.set_vsync(true);
        gpu.set_bg_color(self.bg_color);

        Ok(Game {
            renderer: gpu,
            scene: Scene::new(self.size, CamProj::Persp),
            events: Events::new(),
            time: Time::new(),
            event_loop: Some(el),
            window,
            surface_index,
            runtime: Some(Box::new(runtime)),
            running: true,
            started: false,
        })
    }
}

pub struct Game {
    pub renderer: GPU,
    pub scene: Scene,
    pub events: Events,
    pub time: Time,

    event_loop: Option<EventLoop<()>>,
    window: Window,
    surface_index: usize,
    runtime: Option<Box<dyn Runtime>>,
    running: bool,
    started: bool,
}

impl Game {
    pub fn run(mut self) {
        let el = self.event_loop.take().unwrap();
        let _ = el.run_app(&mut self);
    }

    pub fn exit(&mut self) {
        self.running = false;
    }

    pub fn window(&self) -> &Window {
        &self.window
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
        let window_open = self.window.inner.is_some();
        if !window_open { return; }
        if self.window.inner.as_ref().unwrap().id() != id { return; }

        match &event {
            WindowEvent::Resized(size) => {
                self.window.size = Size2D::from(size.width, size.height);
                self.renderer.ctx.resize_window(self.surface_index, self.window.size);
                let _ = self.renderer.ctx.make_current(self.surface_index);
                self.scene.camera.set_size(self.window.size);
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.window.prev_cursor_pos = self.window.cursor_pos;
                self.window.cursor_pos = (position.x, position.y);
                self.window.cursor_delta = (
                    position.x - self.window.prev_cursor_pos.0,
                    position.y - self.window.prev_cursor_pos.1,
                );
            }
            WindowEvent::CursorEntered { .. } => {
                self.window.cursor_inside = true;
            }
            WindowEvent::CursorLeft { .. } => {
                self.window.cursor_inside = false;
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
            el.exit();
            return;
        }

        let _ = self.renderer.ctx.make_current(self.surface_index);
        self.renderer.clear();
        self.time.update();

        let mut runtime = self.runtime.take().unwrap();
        if !self.started {
            runtime.start(self);
            self.started = true;
        }
        runtime.update(self);
        self.runtime = Some(runtime);

        let _ = self.renderer.ctx.swap_buffers(self.surface_index);
        self.events.end_frame();
        self.window.request_redraw();
    }
}
