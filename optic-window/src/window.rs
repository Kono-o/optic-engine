//! Window management, sizing, positioning, and cursor control.
//!
//! This module provides [`Window`], a wrapper around a winit window handle that
//! adds frame-based cursor tracking (delta, loopback wrapping) and cached
//! window state. All queries against the OS are live unless noted otherwise,
//! while frame-dependent values (deltas, previous positions) are computed by
//! [`Window::update_frame`] once per frame.
//!
//! Cursor behaviour is divided into three modes — grab, confine, and
//! loopback — documented on the individual setter methods.

use optic_core::{Coord2D, CoordOffset, Size2D};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::window::{CursorGrabMode, CursorIcon, Fullscreen, Window as WinitWindow};

use crate::ScreenInfo;

/// The engine's window abstraction — a winit wrapper with frame-tracking and cursor management.
///
/// `Window` is the central interface between the engine and the operating system's
/// windowing layer. It wraps a winit window handle and adds frame-based cursor delta
/// tracking, cursor mode management (grab, confine, loopback), and cached window state
/// (size, position, fullscreen) so the game loop can query window properties without
/// repeated OS calls.
///
/// Owns an optional `Arc<WinitWindow>`. When closed, the inner handle is
/// set to `None` and all methods become no-ops returning default values.
///
/// # Frame tracking
///
/// Call [`update_frame`](Window::update_frame) once per frame after
/// processing events. This computes:
/// - Cursor delta (movement since last frame)
/// - Window position delta
/// - Cursor loopback wrapping
///
/// # Cursor modes
///
/// The window supports three cursor modes:
/// - **Grab** — cursor is locked to the window (hidden, infinite movement)
/// - **Confine** — cursor is confined to the window area (visible, clamped)
/// - **Loopback** — cursor position wraps at window edges (software, for raw
///   input in first-person controls)
///
/// Grab and confine are winit-level operations; loopback is implemented by
/// [`update_frame`](Window::update_frame) warping the cursor.
#[derive(Debug)]
pub struct Window {
    inner: Option<std::sync::Arc<WinitWindow>>,
    prev_cursor_pos: Coord2D,
    cursor_delta: CoordOffset,
    prev_position: Coord2D,
    position_delta: CoordOffset,
    prev_size: Size2D,
    cursor_inside: bool,
    tracking_started: bool,
    cursor_pos: Coord2D,
    cursor_visible: bool,
    cursor_grabbed: bool,
    cursor_confined: bool,
    cursor_loopback: bool,
    min_size: Option<Size2D>,
    max_size: Option<Size2D>,
}

impl Window {
    /// Create a new window.
    ///
    /// The window starts hidden — call [`set_visible`](Window::set_visible)`(true)`
    /// when ready, or let the game loop manage visibility.
    #[allow(deprecated)]
    pub fn new(el: &winit::event_loop::EventLoop<()>, title: &str, size: Size2D) -> Self {
        let attrs = WinitWindow::default_attributes()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(size.w, size.h))
            .with_visible(false);
        let w = el.create_window(attrs).unwrap();
        let arc = std::sync::Arc::new(w);
        let window = Self {
            inner: Some(arc),
            prev_cursor_pos: Coord2D::zero(),
            cursor_delta: CoordOffset::zero(),
            prev_position: Coord2D::zero(),
            position_delta: CoordOffset::zero(),
            prev_size: size,
            cursor_inside: true,
            tracking_started: false,
            cursor_pos: Coord2D::zero(),
            cursor_visible: true,
            cursor_grabbed: false,
            cursor_confined: false,
            cursor_loopback: false,
            min_size: None,
            max_size: None,
        };
        window
    }

    /// Create a new transparent window (requires a compositor that supports transparency).
    ///
    /// Same as [`new`](Window::new) but sets `with_transparent(true)` on the winit window.
    #[allow(deprecated)]
    pub fn new_transparent(el: &winit::event_loop::EventLoop<()>, title: &str, size: Size2D) -> Self {
        let attrs = WinitWindow::default_attributes()
            .with_title(title)
            .with_inner_size(PhysicalSize::new(size.w, size.h))
            .with_visible(false)
            .with_transparent(true);
        let w = el.create_window(attrs).unwrap();
        let arc = std::sync::Arc::new(w);
        let window = Self {
            inner: Some(arc),
            prev_cursor_pos: Coord2D::zero(),
            cursor_delta: CoordOffset::zero(),
            prev_position: Coord2D::zero(),
            position_delta: CoordOffset::zero(),
            prev_size: size,
            cursor_inside: true,
            tracking_started: false,
            cursor_pos: Coord2D::zero(),
            cursor_visible: true,
            cursor_grabbed: false,
            cursor_confined: false,
            cursor_loopback: false,
            min_size: None,
            max_size: None,
        };
        window
    }

    /// Close the window. The inner winit handle is dropped.
    pub fn close(&mut self) {
        self.inner = None;
    }

    /// True if the window has been closed.
    pub fn is_closed(&self) -> bool {
        self.inner.is_none()
    }

    /// True if the window is still open.
    pub fn is_running(&self) -> bool {
        self.inner.is_some()
    }

    // ── Identity ──────────────────────────────────────────────────────────

    /// Raw window handle for EGL surface creation.
    pub fn raw_handle(&self) -> Option<raw_window_handle::RawWindowHandle> {
        use raw_window_handle::HasWindowHandle;
        self.inner.as_ref().map(|w| w.window_handle().unwrap().as_raw())
    }

    /// Raw display handle for EGL display connection.
    pub fn raw_display_handle(&self) -> Option<raw_window_handle::RawDisplayHandle> {
        use raw_window_handle::HasDisplayHandle;
        self.inner.as_ref().map(|w| w.display_handle().unwrap().as_raw())
    }

    /// The winit window ID.
    pub fn id(&self) -> Option<winit::window::WindowId> {
        self.inner.as_ref().map(|w| w.id())
    }

    // ── Sizing ────────────────────────────────────────────────────────────

    /// Current inner size (live winit query).
    pub fn size(&self) -> Size2D {
        self.inner.as_ref().map_or(self.prev_size, |w| {
            let s = w.inner_size();
            Size2D::new(s.width, s.height)
        })
    }

    /// Request a new inner size.
    ///
    /// The OS may not honor the exact request — check [`size()`](Window::size) later.
    pub fn set_size(&self, size: Size2D) {
        if let Some(ref w) = self.inner {
            let _ = w.request_inner_size(PhysicalSize::new(size.w, size.h));
        }
    }

    /// The size cached at the last [`update_frame`](Window::update_frame) call.
    pub fn prev_size(&self) -> Size2D {
        self.prev_size
    }

    /// Minimum window size (cached value, not a live winit query).
    pub fn min_size(&self) -> Option<Size2D> {
        self.min_size
    }

    /// Set the minimum window size.
    pub fn set_min_size(&mut self, size: Option<Size2D>) {
        self.min_size = size;
        if let Some(ref w) = self.inner {
            w.set_min_inner_size(size.map(|s| PhysicalSize::new(s.w, s.h)));
        }
    }

    /// Maximum window size (cached value, not a live winit query).
    pub fn max_size(&self) -> Option<Size2D> {
        self.max_size
    }

    /// Set the maximum window size.
    pub fn set_max_size(&mut self, size: Option<Size2D>) {
        self.max_size = size;
        if let Some(ref w) = self.inner {
            w.set_max_inner_size(size.map(|s| PhysicalSize::new(s.w, s.h)));
        }
    }

    /// True if the window is resizable (live winit query).
    pub fn is_resizable(&self) -> bool {
        self.inner.as_ref().map_or(true, |w| w.is_resizable())
    }

    /// Enable or disable resizing.
    pub fn set_resizable(&self, enable: bool) {
        if let Some(ref w) = self.inner {
            w.set_resizable(enable);
        }
    }

    // ── Desktop Position ──────────────────────────────────────────────────

    /// Desktop position of the window (live winit query via `outer_position`).
    pub fn position(&self) -> Coord2D {
        self.inner.as_ref().and_then(|w| {
            w.outer_position().ok().map(|p| Coord2D::new(p.x as f64, p.y as f64))
        }).unwrap_or(self.prev_position)
    }

    /// Set the desktop position.
    pub fn set_position(&self, pos: Coord2D) {
        if let Some(ref w) = self.inner {
            let _ = w.set_outer_position(PhysicalPosition::new(pos.x as i32, pos.y as i32));
        }
    }

    /// Center the window on the current monitor.
    pub fn center_on_screen(&self) {
        if let Some(ref w) = self.inner {
            if let Some(monitor) = w.current_monitor() {
                let mon_size = monitor.size();
                let win_size = w.outer_size();
                let x = (mon_size.width.saturating_sub(win_size.width)) / 2;
                let y = (mon_size.height.saturating_sub(win_size.height)) / 2;
                let _ = w.set_outer_position(PhysicalPosition::new(x as i32, y as i32));
            }
        }
    }

    /// Desktop position cached at the last [`update_frame`](Window::update_frame) call.
    pub fn prev_position(&self) -> Coord2D {
        self.prev_position
    }

    /// Cumulative window position delta since the last call to this method (reset on read).
    pub fn position_delta(&mut self) -> CoordOffset {
        let d = self.position_delta;
        self.position_delta = CoordOffset::zero();
        d
    }

    // ── Title ─────────────────────────────────────────────────────────────

    /// Current window title (live winit query).
    pub fn title(&self) -> String {
        self.inner.as_ref().map_or(String::new(), |w| w.title())
    }

    /// Set the window title.
    pub fn set_title(&self, title: &str) {
        if let Some(ref w) = self.inner {
            w.set_title(title);
        }
    }

    // ── Fullscreen ────────────────────────────────────────────────────────

    /// True if the window is currently fullscreen (live winit query).
    pub fn is_fullscreen(&self) -> bool {
        self.inner.as_ref().and_then(|w| w.fullscreen()).is_some()
    }

    /// Enter or exit borderless fullscreen.
    pub fn set_fullscreen(&self, enable: bool) {
        if let Some(ref w) = self.inner {
            if enable {
                w.set_fullscreen(Some(Fullscreen::Borderless(None)));
            } else {
                w.set_fullscreen(None);
            }
        }
    }

    /// Toggle fullscreen.
    pub fn toggle_fullscreen(&self) {
        self.set_fullscreen(!self.is_fullscreen());
    }

    // ── Window State ──────────────────────────────────────────────────────

    /// True if the window is visible (live winit query).
    pub fn is_visible(&self) -> bool {
        self.inner.as_ref().and_then(|w| w.is_visible()).unwrap_or(false)
    }

    /// Show or hide the window.
    pub fn set_visible(&self, visible: bool) {
        if let Some(ref w) = self.inner {
            w.set_visible(visible);
        }
    }

    /// Toggle window visibility.
    pub fn toggle_visible(&self) {
        self.set_visible(!self.is_visible());
    }

    /// True if the window is minimized (live winit query).
    pub fn is_minimized(&self) -> bool {
        self.inner.as_ref().and_then(|w| w.is_minimized()).unwrap_or(false)
    }

    /// Minimize the window.
    pub fn minimize(&self) {
        if let Some(ref w) = self.inner {
            w.set_minimized(true);
        }
    }

    /// Restore from minimized.
    pub fn restore(&self) {
        if let Some(ref w) = self.inner {
            w.set_minimized(false);
        }
    }

    /// Toggle minimized state.
    pub fn toggle_minimized(&self) {
        if self.is_minimized() {
            self.restore();
        } else {
            self.minimize();
        }
    }

    /// True if the window is maximized (live winit query).
    pub fn is_maximized(&self) -> bool {
        self.inner.as_ref().map_or(false, |w| w.is_maximized())
    }

    /// Maximize the window.
    pub fn maximize(&self) {
        if let Some(ref w) = self.inner {
            w.set_maximized(true);
        }
    }

    /// Unmaximize the window (restore).
    pub fn unmaximize(&self) {
        if let Some(ref w) = self.inner {
            w.set_maximized(false);
        }
    }

    /// Toggle maximized state.
    pub fn toggle_maximized(&self) {
        if self.is_maximized() {
            self.unmaximize();
        } else {
            self.maximize();
        }
    }

    /// True if the window has focus (live winit query).
    pub fn has_focus(&self) -> bool {
        self.inner.as_ref().map_or(false, |w| w.has_focus())
    }

    /// Request focus.
    pub fn focus(&self) {
        if let Some(ref w) = self.inner {
            w.focus_window();
        }
    }

    // ── Frame Control ─────────────────────────────────────────────────────

    /// Request a redraw from winit.
    pub fn request_redraw(&self) {
        if let Some(ref w) = self.inner {
            w.request_redraw();
        }
    }

    /// The DPI scale factor for this window (live winit query).
    pub fn dpi_scale(&self) -> f64 {
        self.inner.as_ref().map_or(1.0, |w| w.scale_factor())
    }

    /// Set the cursor icon.
    pub fn set_cursor(&self, cursor: CursorIcon) {
        if let Some(ref w) = self.inner {
            w.set_cursor(cursor);
        }
    }

    // ── Screen ────────────────────────────────────────────────────────

    /// Information about the monitor this window is on.
    pub fn screen_info(&self) -> Option<ScreenInfo> {
        self.inner.as_ref().and_then(|w| {
            w.current_monitor().map(|m| ScreenInfo::from_handle(&m))
        })
    }

    // ── Cursor ────────────────────────────────────────────────────────────

    /// Cached cursor position (updated via events or `set_cursor_pos`).
    pub fn cursor_pos(&self) -> Coord2D {
        self.cursor_pos
    }

    /// Move the OS cursor and update the cached position.
    pub fn set_cursor_pos(&mut self, pos: Coord2D) {
        self.cursor_pos = pos;
        if let Some(ref w) = self.inner {
            let _ = w.set_cursor_position(PhysicalPosition::new(pos.x, pos.y));
        }
    }

    /// Cursor delta since the last frame (computed by [`update_frame`](Window::update_frame)).
    ///
    /// Y is inverted (positive = up) to match screen coordinates.
    pub fn cursor_delta(&self) -> CoordOffset {
        self.cursor_delta
    }

    /// Cursor position normalized to 0..1 where (0,0) = bottom-left, (1,1) = top-right.
    pub fn cursor_pos_normalized(&self) -> Coord2D {
        let sz = self.size();
        if sz.w == 0 || sz.h == 0 {
            return Coord2D::zero();
        }
        Coord2D::new(self.cursor_pos.x / sz.w as f64, 1.0 - self.cursor_pos.y / sz.h as f64)
    }

    /// True if the cursor is inside the window client area (updated via events).
    pub fn is_cursor_inside(&self) -> bool {
        self.cursor_inside
    }

    /// True if the cursor is visible (cached, not a live winit query).
    pub fn is_cursor_visible(&self) -> bool {
        self.cursor_visible
    }

    /// Show or hide the cursor.
    pub fn set_cursor_visible(&mut self, visible: bool) {
        self.cursor_visible = visible;
        if let Some(ref w) = self.inner {
            w.set_cursor_visible(visible);
        }
    }

    /// Toggle cursor visibility.
    pub fn toggle_cursor_visible(&mut self) {
        self.set_cursor_visible(!self.cursor_visible);
    }

    /// True if the cursor is grabbed (locked + hidden, for raw input).
    pub fn is_cursor_grabbed(&self) -> bool {
        self.cursor_grabbed
    }

    /// Lock or release the cursor (grab mode).
    ///
    /// When enabled the cursor is hidden and reports infinite movement,
    /// suitable for first-person camera input.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if the platform does not support
    /// [`CursorGrabMode::Locked`] or the window handle has been closed.
    /// The cached grab state is only updated on success.
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

    /// Toggle cursor grab.
    pub fn toggle_cursor_grab(&mut self) {
        let _ = self.set_cursor_grab(!self.cursor_grabbed);
    }

    /// True if the cursor is confined (clamped to window, visible).
    pub fn is_cursor_confined(&self) -> bool {
        self.cursor_confined
    }

    /// Confine or release the cursor.
    ///
    /// When enabled the cursor is clamped to the window area but remains
    /// visible, unlike grab mode.
    ///
    /// # Errors
    ///
    /// Returns `Err(())` if the platform does not support
    /// [`CursorGrabMode::Confined`] or the window handle has been closed.
    /// The cached confine state is only updated on success.
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

    /// Toggle cursor confine.
    pub fn toggle_cursor_confine(&mut self) {
        let _ = self.set_cursor_confine(!self.cursor_confined);
    }

    /// True if cursor loopback (software edge-wrapping) is enabled.
    pub fn is_cursor_loopback(&self) -> bool {
        self.cursor_loopback
    }

    /// Enable or disable loopback mode. When enabled, [`update_frame`](Window::update_frame)
    /// will wrap the cursor position at window edges, useful for first-person camera control.
    pub fn set_cursor_loopback(&mut self, loopback: bool) {
        self.cursor_loopback = loopback;
    }

    // ── Frame Update ──────────────────────────────────────────────────────

    /// Call once per frame after processing events.
    ///
    /// Snapshots cursor position, computes cursor/window deltas, and applies
    /// loopback wrapping if enabled.
    pub fn update_frame(&mut self) {
        let new_pos = self.position();
        if !self.tracking_started {
            self.prev_cursor_pos = self.cursor_pos;
            self.prev_position = new_pos;
            self.prev_size = self.size();
            self.position_delta = CoordOffset::zero();
            self.cursor_delta = CoordOffset::zero();
            self.tracking_started = true;
        } else {
            let raw = self.cursor_pos - self.prev_cursor_pos;
            self.cursor_delta = CoordOffset { x: raw.x, y: -raw.y };
            self.prev_cursor_pos = self.cursor_pos;
            self.position_delta = self.position_delta + (new_pos - self.prev_position);
            self.prev_position = new_pos;
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

    // ── Internal ──────────────────────────────────────────────────────────

    #[doc(hidden)]
    pub fn notify_cursor_moved(&mut self, pos: Coord2D) {
        self.cursor_pos = pos;
    }

    #[doc(hidden)]
    pub fn notify_cursor_inside(&mut self, inside: bool) {
        self.cursor_inside = inside;
    }
}
