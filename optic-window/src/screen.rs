//! Monitor / display information.
//!
//! Provides [`ScreenInfo`], a platform-agnostic snapshot of a connected
//! monitor's resolution, refresh rate, scale factor, and desktop position.
//! Obtained via [`Window::screen_info`](crate::Window::screen_info).

use optic_core::{Coord2D, Size2D};

/// Information about a connected display monitor.
///
/// Provides a platform-agnostic snapshot of a monitor's native resolution, DPI
/// scale factor, refresh rate, and desktop position. Obtained via
/// [`Window::screen_info`](crate::Window::screen_info) and used by the engine
/// to adapt rendering resolution, choose display modes, or position the window
/// on a specific monitor.
#[derive(Debug, Clone)]
pub struct ScreenInfo {
    /// Human-readable monitor name (e.g. `"DP-1"`).
    pub name: String,
    /// Native resolution in pixels.
    pub size: Size2D,
    /// Refresh rate in Hz.
    pub refresh_rate: u32,
    /// OS scaling factor (e.g. `1.0` = 100%, `2.0` = 200%).
    pub scale_factor: f64,
    /// Top-left corner position in desktop coordinates.
    pub position: Coord2D,
}

impl ScreenInfo {
    /// Construct from a winit [`MonitorHandle`](winit::monitor::MonitorHandle).
    pub fn from_handle(handle: &winit::monitor::MonitorHandle) -> Self {
        let sz = handle.size();
        let pos = handle.position();
        Self {
            name: handle.name().unwrap_or_default(),
            size: Size2D::new(sz.width, sz.height),
            refresh_rate: handle.refresh_rate_millihertz().unwrap_or(0) / 1000,
            scale_factor: handle.scale_factor(),
            position: Coord2D::new(pos.x as f64, pos.y as f64),
        }
    }
}
