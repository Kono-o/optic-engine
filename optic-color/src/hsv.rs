use crate::{RGBA, ToRgba};

/// HSV color.
///
/// | Field | Range | Description |
/// |-------|-------|-------------|
/// | `h`   | 0..360 | Hue angle (wraps at 360) |
/// | `s`   | 0..1   | Saturation |
/// | `v`   | 0..1   | Value (brightness) |
///
/// # Why no arithmetic?
///
/// `HSV` does not implement [`ChannelArray`], [`Add`], [`Sub`], [`Mul`],
/// or `lerp`. Hue is an angle on a circle — componentwise interpolation
/// between 350° and 10° would pass through 180° instead of the short arc
/// through 0°. This produces incorrect visual results.
///
/// To manipulate HSV colors, convert to [`RGBA`], do your math there,
/// then convert back:
///
/// ```
/// use optic_color::*;
///
/// let hsv = HSV::new(350.0, 0.8, 0.9);
/// let mut rgba: RGBA = hsv.into();
/// rgba = rgba.lighten(0.1);
/// ```
///
/// For hue-aware interpolation between two colors, use [`Gradient`] with
/// [`GradientColorSpace::Hsv`].
///
/// [`ChannelArray`]: crate::ChannelArray
/// [`Gradient`]: crate::Gradient
#[derive(Copy, Clone, Debug)]
pub struct HSV {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}

impl HSV {
    /// Construct an HSV color with clamping.
    ///
    /// Hue is clamped to 0..360, saturation and value to 0..1.
    pub fn new(h: f32, s: f32, v: f32) -> Self {
        HSV { h: h.clamp(0.0, 360.0), s: s.clamp(0.0, 1.0), v: v.clamp(0.0, 1.0) }
    }

    /// Convert to RGBA with a custom alpha, without going through [`ToRgba`].
    ///
    /// Equivalent to `self.to_rgba().with_alpha(alpha)`.
    pub fn to_rgba_alpha(self, alpha: f32) -> RGBA {
        self.to_rgba().with_alpha(alpha)
    }
}
