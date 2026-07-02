use optic_core::{Coord2D, CoordOffset, Size2D};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::{CursorGrabMode, Fullscreen, Window as WinitWindow};

#[derive(Debug)]
pub struct Window {
    inner: Option<std::sync::Arc<WinitWindow>>,

    // ── Frame-tracking (computed each frame) ─────────────────────────────
    prev_cursor_pos: Coord2D,
    cursor_delta: CoordOffset,
    prev_position: Coord2D,
    prev_size: Size2D,
    cursor_inside: bool,
    tracking_started: bool,

    // ── Cached state (no live winit query available) ──────────────────────
    cursor_pos: Coord2D,
    cursor_visible: bool,
    cursor_grabbed: bool,
    cursor_confined: bool,
    cursor_loopback: bool,
    min_size: Option<Size2D>,
    max_size: Option<Size2D>,
}

impl Window {
    // ── Construction ──────────────────────────────────────────────────────
    #[allow(deprecated)]
    pub fn new(el: &winit::event_loop::EventLoop<()>, title: &str, size: Size2D) -> Self {
        let attrs = WinitWindow::default_attributes()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(size.w, size.h));
        let w = el.create_window(attrs).unwrap();
        let arc = std::sync::Arc::new(w);
        Self {
            inner: Some(arc),
            prev_cursor_pos: Coord2D::empty(),
            cursor_delta: CoordOffset::empty(),
            prev_position: Coord2D::empty(),
            prev_size: size,
            cursor_inside: true,
            tracking_started: false,
            cursor_pos: Coord2D::empty(),
            cursor_visible: true,
            cursor_grabbed: false,
            cursor_confined: false,
            cursor_loopback: false,
            min_size: None,
            max_size: None,
        }
    }

    pub fn close(&mut self) {
        self.inner = None;
    }

    pub fn is_closed(&self) -> bool {
        self.inner.is_none()
    }

    pub fn is_running(&self) -> bool {
        self.inner.is_some()
    }

    // ── Identity ──────────────────────────────────────────────────────────
    pub fn raw_handle(&self) -> Option<raw_window_handle::RawWindowHandle> {
        use raw_window_handle::HasWindowHandle;
        self.inner.as_ref().map(|w| w.window_handle().unwrap().as_raw())
    }

    pub fn id(&self) -> Option<winit::window::WindowId> {
        self.inner.as_ref().map(|w| w.id())
    }

    // ── Sizing ────────────────────────────────────────────────────────────
    /// Live query via winit `inner_size()`.
    pub fn size(&self) -> Size2D {
        self.inner.as_ref().map_or(self.prev_size, |w| {
            let s = w.inner_size();
            Size2D::from(s.width, s.height)
        })
    }

    pub fn set_size(&self, size: Size2D) {
        if let Some(ref w) = self.inner {
            let _ = w.request_inner_size(PhysicalSize::new(size.w, size.h));
        }
    }

    pub fn prev_size(&self) -> Size2D {
        self.prev_size
    }

    /// Cached from the last `set_min_size` call (no live winit query).
    pub fn min_size(&self) -> Option<Size2D> {
        self.min_size
    }

    pub fn set_min_size(&mut self, size: Option<Size2D>) {
        self.min_size = size;
        if let Some(ref w) = self.inner {
            w.set_min_inner_size(size.map(|s| PhysicalSize::new(s.w, s.h)));
        }
    }

    /// Cached from the last `set_max_size` call (no live winit query).
    pub fn max_size(&self) -> Option<Size2D> {
        self.max_size
    }

    pub fn set_max_size(&mut self, size: Option<Size2D>) {
        self.max_size = size;
        if let Some(ref w) = self.inner {
            w.set_max_inner_size(size.map(|s| PhysicalSize::new(s.w, s.h)));
        }
    }

    /// Live query via winit `is_resizable()`.
    pub fn resizable(&self) -> bool {
        self.inner.as_ref().map_or(true, |w| w.is_resizable())
    }

    pub fn set_resizable(&self, enable: bool) {
        if let Some(ref w) = self.inner {
            w.set_resizable(enable);
        }
    }

    // ── Desktop Position ──────────────────────────────────────────────────
    /// Live query via winit `outer_position()`.
    pub fn position(&self) -> Coord2D {
        self.inner.as_ref().and_then(|w| {
            w.outer_position().ok().map(|p| Coord2D::from(p.x as f64, p.y as f64))
        }).unwrap_or(self.prev_position)
    }

    pub fn set_position(&self, pos: Coord2D) {
        if let Some(ref w) = self.inner {
            let _ = w.set_outer_position(PhysicalPosition::new(pos.x as i32, pos.y as i32));
        }
    }

    pub fn prev_position(&self) -> Coord2D {
        self.prev_position
    }

    // ── Title ─────────────────────────────────────────────────────────────
    /// Live query via winit `title()`.
    pub fn title(&self) -> String {
        self.inner.as_ref().map_or(String::new(), |w| w.title())
    }

    pub fn set_title(&self, title: &str) {
        if let Some(ref w) = self.inner {
            w.set_title(title);
        }
    }

    // ── Fullscreen ────────────────────────────────────────────────────────
    /// Live query via winit `fullscreen()`.
    pub fn is_fullscreen(&self) -> bool {
        self.inner.as_ref().and_then(|w| w.fullscreen()).is_some()
    }

    pub fn set_fullscreen(&self, enable: bool) {
        if let Some(ref w) = self.inner {
            if enable {
                w.set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                w.set_fullscreen(None);
            }
        }
    }

    pub fn toggle_fullscreen(&self) {
        self.set_fullscreen(!self.is_fullscreen());
    }

    // ── Window State ──────────────────────────────────────────────────────
    /// Live query via winit `is_visible()`.
    pub fn is_visible(&self) -> bool {
        self.inner.as_ref().and_then(|w| w.is_visible()).unwrap_or(false)
    }

    pub fn set_visible(&self, visible: bool) {
        if let Some(ref w) = self.inner {
            w.set_visible(visible);
        }
    }

    /// Live query via winit `is_minimized()`.
    pub fn is_minimized(&self) -> bool {
        self.inner.as_ref().and_then(|w| w.is_minimized()).unwrap_or(false)
    }

    pub fn minimize(&self) {
        if let Some(ref w) = self.inner {
            w.set_minimized(true);
        }
    }

    pub fn restore(&self) {
        if let Some(ref w) = self.inner {
            w.set_minimized(false);
        }
    }

    /// Live query via winit `is_maximized()`.
    pub fn is_maximized(&self) -> bool {
        self.inner.as_ref().map_or(false, |w| w.is_maximized())
    }

    pub fn maximize(&self) {
        if let Some(ref w) = self.inner {
            w.set_maximized(true);
        }
    }

    pub fn unmaximize(&self) {
        if let Some(ref w) = self.inner {
            w.set_maximized(false);
        }
    }

    /// Live query via winit `has_focus()`.
    pub fn has_focus(&self) -> bool {
        self.inner.as_ref().map_or(false, |w| w.has_focus())
    }

    pub fn focus(&self) {
        if let Some(ref w) = self.inner {
            w.focus_window();
        }
    }

    // ── Frame Control ─────────────────────────────────────────────────────
    pub fn request_redraw(&self) {
        if let Some(ref w) = self.inner {
            w.request_redraw();
        }
    }

    // ── Cursor ────────────────────────────────────────────────────────────
    /// Returns the last-known cursor position (updated via events or `set_cursor_pos`).
    pub fn cursor_pos(&self) -> Coord2D {
        self.cursor_pos
    }

    /// Moves the OS cursor and updates the cached position.
    pub fn set_cursor_pos(&mut self, pos: Coord2D) {
        self.cursor_pos = pos;
        if let Some(ref w) = self.inner {
            let _ = w.set_cursor_position(PhysicalPosition::new(pos.x, pos.y));
        }
    }

    pub fn cursor_delta(&self) -> CoordOffset {
        self.cursor_delta
    }

    pub fn cursor_pos_normalized(&self) -> Coord2D {
        let sz = self.size();
        if sz.w == 0 || sz.h == 0 {
            return Coord2D::empty();
        }
        Coord2D::from(self.cursor_pos.x / sz.w as f64, 1.0 - self.cursor_pos.y / sz.h as f64)
    }

    /// Updated via `CursorEntered`/`CursorLeft` events.
    pub fn is_cursor_inside(&self) -> bool {
        self.cursor_inside
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        if let Some(ref w) = self.inner {
            w.set_cursor_visible(visible);
        }
    }

    pub fn toggle_cursor_visible(&mut self) {
        self.set_cursor_visible(!self.cursor_visible);
    }

    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    pub fn set_cursor_grab(&mut self, grab: bool) -> Result<(), ()> {
        let result = match self.inner.as_ref() {
            Some(w) => w.set_cursor_grab(if grab { CursorGrabMode::Locked } else { CursorGrabMode::None })
                .map_err(|_| ()),
            None => Err(()),
        };
        if result.is_ok() {
            self.cursor_grabbed = grab;
        }
        result
    }

    pub fn toggle_cursor_grab(&mut self) {
        let _ = self.set_cursor_grab(!self.cursor_grabbed);
    }

    pub fn is_cursor_confined(&self) -> bool {
        self.cursor_confined
    }

    pub fn set_cursor_confine(&mut self, confine: bool) -> Result<(), ()> {
        let result = match self.inner.as_ref() {
            Some(w) => w.set_cursor_grab(if confine { CursorGrabMode::Confined } else { CursorGrabMode::None })
                .map_err(|_| ()),
            None => Err(()),
        };
        if result.is_ok() {
            self.cursor_confined = confine;
        }
        result
    }

    pub fn toggle_cursor_confine(&mut self) {
        let _ = self.set_cursor_confine(!self.cursor_confined);
    }

    pub fn is_cursor_loopback(&self) -> bool {
        self.cursor_loopback
    }

    pub fn set_cursor_loopback(&mut self, loopback: bool) {
        self.cursor_loopback = loopback;
    }

    // ── Frame Update ──────────────────────────────────────────────────────
    /// Call once per frame after processing events.
    /// Snapshots live/tracked state and computes cursor delta.
    pub fn update_frame(&mut self) {
        if !self.tracking_started {
            self.prev_cursor_pos = self.cursor_pos;
            self.prev_position = self.position();
            self.prev_size = self.size();
            self.cursor_delta = CoordOffset::empty();
            self.tracking_started = true;
        } else {
            self.cursor_delta = CoordOffset::from(
                self.cursor_pos.x - self.prev_cursor_pos.x,
                self.prev_cursor_pos.y - self.cursor_pos.y,
            );
            self.prev_cursor_pos = self.cursor_pos;
            self.prev_position = self.position();
            self.prev_size = self.size();
        }

        if self.cursor_loopback {
            let sz = self.size();
            if sz.w > 0 && sz.h > 0 {
                let mut wrapped = false;
                let mut new_pos = self.cursor_pos;
                if new_pos.x <= 0.0 { new_pos.x = (sz.w - 1) as f64; wrapped = true; }
                else if new_pos.x >= (sz.w - 1) as f64 { new_pos.x = 0.0; wrapped = true; }
                if new_pos.y <= 0.0 { new_pos.y = (sz.h - 1) as f64; wrapped = true; }
                else if new_pos.y >= (sz.h - 1) as f64 { new_pos.y = 0.0; wrapped = true; }
                if wrapped {
                    self.set_cursor_pos(new_pos);
                    self.prev_cursor_pos = self.cursor_pos;
                }
            }
        }
    }

    // ── Internal (called by optic-loop event handlers) ───────────────────
    #[doc(hidden)]
    pub fn notify_cursor_moved(&mut self, pos: Coord2D) {
        self.cursor_pos = pos;
    }

    #[doc(hidden)]
    pub fn notify_cursor_inside(&mut self, inside: bool) {
        self.cursor_inside = inside;
    }
}
