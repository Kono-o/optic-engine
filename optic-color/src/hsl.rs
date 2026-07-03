use crate::{RGBA, ToRgba};

/// HSL color. Hue is 0..360 (wraps), saturation/lightness are 0..1.
///
/// HSL intentionally does NOT implement `ChannelArray`, `Add`, `Sub`, `Mul`, or `lerp` —
/// hue wraparound makes naive component-wise arithmetic produce incorrect results.
/// Convert to RGBA (`.into()`) for arithmetic, or use `Gradient` with
/// `GradientColorSpace::Hsv` for hue-aware interpolation.
#[derive(Copy, Clone, Debug)]
pub struct HSL {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl HSL {
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        HSL { h: h.clamp(0.0, 360.0), s: s.clamp(0.0, 1.0), l: l.clamp(0.0, 1.0) }
    }

    pub fn to_rgba_alpha(self, alpha: f32) -> RGBA {
        self.to_rgba().with_alpha(alpha)
    }
}
