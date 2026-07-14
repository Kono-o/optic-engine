//! HSL (Hue, Saturation, Lightness) color representation.
//!
//! [`HSL`] is similar to [`HSV`](crate::HSV) but uses lightness instead of
//! value. Lightness maps more directly to perceptual brightness, making HSL
//! a natural choice when adjusting brightness is the primary goal.

use crate::{RGBA, ToRgba};

/// HSL color.
///
/// An alternative perceptual color model similar to [`HSV`] but using
/// lightness instead of value. Lightness maps more directly to perceptual
/// brightness, making HSL a natural choice when adjusting brightness is the
/// primary goal — for example, generating hover-state variants of UI
/// colors or computing accessible contrast ratios.
///
/// | Field | Range | Description |
/// |-------|-------|-------------|
/// | `h`   | 0..360 | Hue angle (wraps at 360) |
/// | `s`   | 0..1   | Saturation |
/// | `l`   | 0..1   | Lightness |
///
/// # Why no arithmetic?
///
/// Same reasoning as [`HSV`]: hue wraparound makes naive componentwise
/// operations incorrect. Convert to [`RGBA`], manipulate there, convert back.
///
/// See [`HSV`] for details and alternatives.
///
/// [`HSV`]: crate::HSV
#[derive(Copy, Clone, Debug)]
pub struct HSL {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}

impl HSL {
    /// Construct an HSL color with clamping.
    ///
    /// Hue is clamped to 0..360, saturation and lightness to 0..1.
    pub fn new(h: f32, s: f32, l: f32) -> Self {
        HSL { h: h.clamp(0.0, 360.0), s: s.clamp(0.0, 1.0), l: l.clamp(0.0, 1.0) }
    }

    /// Convert to RGBA with a custom alpha.
    ///
    /// Equivalent to `self.to_rgba().with_alpha(alpha)`.
    pub fn to_rgba_alpha(self, alpha: f32) -> RGBA {
        self.to_rgba().with_alpha(alpha)
    }
}
