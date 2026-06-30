use optic_core::Size2D;
use winit::dpi::PhysicalSize;
use winit::window::{CursorGrabMode, Fullscreen, Window as WinitWindow};

#[derive(Clone, Debug)]
pub struct Window {
    pub inner: Option<std::sync::Arc<WinitWindow>>,
    pub size: Size2D,
    pub title: String,
    pub fullscreen: bool,
    pub resizable: bool,
    pub cursor_hidden: bool,
    pub cursor_grabbed: bool,
    pub cursor_inside: bool,
    pub cursor_pos: (f64, f64),
    pub prev_cursor_pos: (f64, f64),
    pub cursor_delta: (f64, f64),
}

impl Window {
    #[allow(deprecated)]
    pub fn new(el: &winit::event_loop::EventLoop<()>, title: &str, size: Size2D) -> Self {
        let attrs = WinitWindow::default_attributes()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(size.w, size.h));
        let w = el.create_window(attrs).unwrap();
        let arc = std::sync::Arc::new(w);
        Self {
            inner: Some(arc),
            size,
            title: title.to_string(),
            fullscreen: false,
            resizable: true,
            cursor_hidden: false,
            cursor_grabbed: false,
            cursor_inside: true,
            cursor_pos: (0.0, 0.0),
            prev_cursor_pos: (0.0, 0.0),
            cursor_delta: (0.0, 0.0),
        }
    }

    pub fn close(&mut self) {
        self.inner = None;
    }

    pub fn is_closed(&self) -> bool {
        self.inner.is_none()
    }

    pub fn raw_handle(&self) -> Option<raw_window_handle::RawWindowHandle> {
        use raw_window_handle::HasWindowHandle;
        self.inner.as_ref().map(|w| w.window_handle().unwrap().as_raw())
    }

    pub fn size(&self) -> Size2D { self.size }
    pub fn title(&self) -> &str { &self.title }
    pub fn fullscreen(&self) -> bool { self.fullscreen }

    pub fn cursor_coord_normalized(&self) -> (f64, f64) {
        if self.size.w == 0 || self.size.h == 0 {
            return (0.0, 0.0);
        }
        (
            self.cursor_pos.0 / self.size.w as f64,
            self.cursor_pos.1 / self.size.h as f64,
        )
    }

    pub fn set_cursor_coord(&self, x: f64, y: f64) {
        use winit::dpi::PhysicalPosition;
        if let Some(ref w) = self.inner {
            let _ = w.set_cursor_position(PhysicalPosition::new(x, y));
        }
    }

    pub fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        if let Some(ref w) = self.inner {
            w.set_title(title);
        }
    }

    pub fn set_size(&mut self, size: Size2D) {
        self.size = size;
        if let Some(ref w) = self.inner {
            let _ = w.request_inner_size(PhysicalSize::new(size.w, size.h));
        }
    }

    pub fn set_fullscreen(&mut self, enable: bool) {
        self.fullscreen = enable;
        if let Some(ref w) = self.inner {
            if enable {
                w.set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                w.set_fullscreen(None);
            }
        }
    }

    pub fn toggle_fullscreen(&mut self) {
        self.set_fullscreen(!self.fullscreen);
    }

    pub fn set_resizable(&mut self, enable: bool) {
        self.resizable = enable;
        if let Some(ref w) = self.inner {
            w.set_resizable(enable);
        }
    }

    pub fn set_cursor_grab(&mut self, grab: bool) -> Result<(), ()> {
        self.cursor_grabbed = grab;
        match self.inner.as_ref() {
            Some(w) => match w.set_cursor_grab(if grab { CursorGrabMode::Locked } else { CursorGrabMode::None }) {
                Ok(_) => Ok(()),
                Err(_) => Err(()),
            },
            None => Err(()),
        }
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_hidden = !visible;
        if let Some(ref w) = self.inner {
            w.set_cursor_visible(visible);
        }
    }

    pub fn cursor_position(&self) -> (f64, f64) {
        self.cursor_pos
    }

    pub fn cursor_delta(&self) -> (f64, f64) {
        self.cursor_delta
    }

    pub fn request_redraw(&self) {
        if let Some(ref w) = self.inner {
            w.request_redraw();
        }
    }
}
