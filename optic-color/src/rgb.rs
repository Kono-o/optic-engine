use crate::RGBA;

/// RGB color with three 0..1 float channels.
///
/// This is a tuple struct with public fields:
///
/// | Field | Range  | Description  |
/// |-------|--------|--------------|
/// | `.0`  | 0..1   | Red channel  |
/// | `.1`  | 0..1   | Green channel |
/// | `.2`  | 0..1   | Blue channel  |
///
/// # Arithmetic
///
/// `RGB` implements `Add`, `Sub`, `Mul` (componentwise), `Mul<f32>`,
/// and `Div<f32>` via the [`ChannelArray`] impl. These operate on all
/// three channels independently.
///
/// # Conversions
///
/// ```
/// use optic_color::*;
///
/// let rgb = RGB(0.5, 0.2, 0.8);
/// let rgba: RGBA = rgb.into();        // alpha = 1.0
/// let rgba = rgb.to_rgba(0.5);        // custom alpha
/// let arr: [f32; 3] = rgb.into();     // flatten
/// ```
///
/// See also [`RGBA`], [`crate::HSV`], [`crate::HSL`].
///
/// [`ChannelArray`]: crate::ChannelArray
#[derive(Copy, Clone, Debug)]
pub struct RGB(pub f32, pub f32, pub f32);

impl RGB {
    /// Construct a greyscale RGB.
    ///
    /// All three channels are set to `lum`.
    ///
    /// ```
    /// use optic_color::*;
    ///
    /// let grey = RGB::grey(0.5);
    /// assert_eq!(grey.0, 0.5);
    /// assert_eq!(grey.1, 0.5);
    /// assert_eq!(grey.2, 0.5);
    /// ```
    pub fn grey(lum: f32) -> Self { RGB(lum, lum, lum) }

    /// Construct an RGB from an RGBA, dropping alpha.
    pub fn from_rgba(rgba: RGBA) -> Self { RGB(rgba.0, rgba.1, rgba.2) }

    /// Convert to RGBA with a given alpha.
    ///
    /// ```
    /// use optic_color::*;
    ///
    /// let rgb = RGB(1.0, 0.0, 0.0);
    /// let rgba = rgb.to_rgba(0.5);
    /// assert_eq!(rgba.3, 0.5);
    /// ```
    pub fn to_rgba(&self, alpha: f32) -> RGBA { RGBA(self.0, self.1, self.2, alpha) }
}
