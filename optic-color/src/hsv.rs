use crate::{RGBA, ToRgba};

/// HSV color. Hue is 0..360 (wraps), saturation/value are 0..1.
///
/// HSV intentionally does NOT implement `ChannelArray`, `Add`, `Sub`, `Mul`, or `lerp` —
/// hue wraparound makes naive component-wise arithmetic produce incorrect results
/// (e.g. lerping hue 350° to 10° the naive way drifts through 180° instead of the short
/// way through 360°/0°). Convert to RGBA (`.into()`) for arithmetic, or use `Gradient`
/// with `GradientColorSpace::Hsv` for hue-aware interpolation.
#[derive(Copy, Clone, Debug)]
pub struct HSV {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl HSV {
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        HSV { h: h.clamp(0.0, 360.0), s: s.clamp(0.0, 1.0), v: v.clamp(0.0, 1.0) }
    }

    pub fn to_rgba_alpha(self, alpha: f32) -> RGBA {
        self.to_rgba().with_alpha(alpha)
    }
}
