use optic_core::{Coord2D, Size2D};

/// Information about a connected monitor/screen.
#[derive(Debug, Clone)]
pub struct ScreenInfo {
    pub name: String,
    pub size: Size2D,
    pub refresh_rate: u32,
    pub scale_factor: f64,
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
