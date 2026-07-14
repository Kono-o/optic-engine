//! RGBA color representation with alpha channel.
//!
//! [`RGBA`] is the central color type in the crate. All other color spaces
//! ([`RGB`](crate::RGB), [`HSV`](crate::HSV), [`HSL`](crate::HSL)) convert
//! through it, and most engine APIs accept or return `RGBA` directly.
//!
//! Channels are `f32` values in 0..1. Construction is available from:
//! - Individual floats via [`RGBA::new`]
//! - Hex strings via [`RGBA::from_hex`]
//! - Packed u32 via [`RGBA::from_hex_u32`]
//! - 8-bit bytes via [`RGBA::from_bytes`]

use crate::{ColorInfo, FromRgba, HSV, RGB, ToRgba};

/// RGBA color with four 0..1 float channels.
///
/// This is the primary color type in Optic. Most engine APIs accept or
/// return [`RGBA`] directly. All other color types convert through it.
///
/// | Field | Range | Description |
/// |-------|-------|-------------|
/// | `.0`  | 0..1  | Red |
/// | `.1`  | 0..1  | Green |
/// | `.2`  | 0..1  | Blue |
/// | `.3`  | 0..1  | Alpha (0 = transparent, 1 = opaque) |
///
/// # Hex parsing
///
/// ```
/// use optic_color::*;
///
/// let c = RGBA::from_hex("#ff8800").unwrap();
/// let c = RGBA::from_hex("#f80").unwrap();     // shorthand
/// let c = RGBA::from_hex("#ff880044").unwrap(); // with alpha
/// let c = RGBA::from_hex_u32(0xff880044);
/// ```
///
/// # HSV modifiers
///
/// ```
/// use optic_color::*;
///
/// let red = RED;
/// let pink = red.lighten(0.3);
/// let dull = red.desaturate(0.5);
/// let inv = red.invert();
/// ```
///
/// # sRGB conversions
///
/// [`to_linear`](RGBA::to_linear) applies the sRGB EOTF (decodes display
/// encoding to linear light). [`to_srgb`](RGBA::to_srgb) applies the OETF
/// (encodes linear light for display).
#[derive(Copy, Clone, Debug)]
pub struct RGBA(pub f32, pub f32, pub f32, pub f32);

impl RGBA {
    /// Construct an RGBA from individual 0..1 float channels.
    ///
    /// This is a `const fn`, usable in constant contexts.
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self { RGBA(r, g, b, a) }

    /// Construct a greyscale RGBA with alpha 1.0.
    ///
    /// ```
    /// use optic_color::*;
    /// let grey = RGBA::grey(0.5);
    /// ```
    pub fn grey(lum: f32) -> Self { RGBA(lum, lum, lum, 1.0) }

    /// Construct from an [`RGB`] and an alpha value.
    pub fn from_rgb(rgb: RGB, alpha: f32) -> Self { RGBA(rgb.0, rgb.1, rgb.2, alpha) }

    /// Drop alpha, returning an [`RGB`].
    pub fn to_rgb(&self) -> RGB { RGB(self.0, self.1, self.2) }

    /// Replace the alpha channel, returning a new [`RGBA`].
    ///
    /// The RGB channels are unchanged.
    pub fn with_alpha(self, a: f32) -> RGBA { RGBA(self.0, self.1, self.2, a) }

    /// Parse a hex color string.
    ///
    /// Supports the following formats (with or without `#` prefix):
    ///
    /// | Length | Format     | Example     |
    /// |--------|------------|-------------|
    /// | 3      | `#RGB`     | `#f80`      |
    /// | 4      | `#RGBA`    | `#f80c`     |
    /// | 6      | `#RRGGBB`  | `#ff8800`   |
    /// | 8      | `#RRGGBBAA`| `#ff880044` |
    ///
    /// # Errors
    ///
    /// Returns `Err` if the string contains non-hex characters or is not
    /// one of the supported lengths (3, 4, 6, or 8 hex digits).
    pub fn from_hex(hex: &str) -> Result<Self, &'static str> {
        let hex = hex.strip_prefix('#').unwrap_or(hex);
        match hex.len() {
            3 => {
                let r = u8::from_str_radix(&hex[0..1], 16).map_err(|_| "invalid hex")?;
                let g = u8::from_str_radix(&hex[1..2], 16).map_err(|_| "invalid hex")?;
                let b = u8::from_str_radix(&hex[2..3], 16).map_err(|_| "invalid hex")?;
                let r = (r as f32 / 15.0 * 255.0).round() as u8;
                let g = (g as f32 / 15.0 * 255.0).round() as u8;
                let b = (b as f32 / 15.0 * 255.0).round() as u8;
                Ok(RGBA::from_bytes(r, g, b, 255))
            }
            4 => {
                let r = u8::from_str_radix(&hex[0..1], 16).map_err(|_| "invalid hex")?;
                let g = u8::from_str_radix(&hex[1..2], 16).map_err(|_| "invalid hex")?;
                let b = u8::from_str_radix(&hex[2..3], 16).map_err(|_| "invalid hex")?;
                let a = u8::from_str_radix(&hex[3..4], 16).map_err(|_| "invalid hex")?;
                let r = (r as f32 / 15.0 * 255.0).round() as u8;
                let g = (g as f32 / 15.0 * 255.0).round() as u8;
                let b = (b as f32 / 15.0 * 255.0).round() as u8;
                let a = (a as f32 / 15.0 * 255.0).round() as u8;
                Ok(RGBA::from_bytes(r, g, b, a))
            }
            6 => {
                let val = u32::from_str_radix(hex, 16).map_err(|_| "invalid hex")?;
                let r = ((val >> 16) & 0xFF) as u8;
                let g = ((val >> 8) & 0xFF) as u8;
                let b = (val & 0xFF) as u8;
                Ok(RGBA::from_bytes(r, g, b, 255))
            }
            8 => {
                let val = u32::from_str_radix(hex, 16).map_err(|_| "invalid hex")?;
                let r = ((val >> 24) & 0xFF) as u8;
                let g = ((val >> 16) & 0xFF) as u8;
                let b = ((val >> 8) & 0xFF) as u8;
                let a = (val & 0xFF) as u8;
                Ok(RGBA::from_bytes(r, g, b, a))
            }
            _ => Err("hex must be 3, 4, 6, or 8 hex digits (optionally with # prefix)"),
        }
    }

    /// Construct from a packed `0xRRGGBBAA` u32.
    ///
    /// ```
    /// use optic_color::*;
    /// let c = RGBA::from_hex_u32(0xff8800ff);
    /// ```
    pub fn from_hex_u32(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as u8;
        let g = ((hex >> 16) & 0xFF) as u8;
        let b = ((hex >> 8) & 0xFF) as u8;
        let a = (hex & 0xFF) as u8;
        RGBA::from_bytes(r, g, b, a)
    }

    /// Encode as a `0xRRGGBBAA` u32.
    pub fn to_hex_u32(self) -> u32 {
        let (r, g, b, a) = self.to_bytes();
        (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | a as u32
    }

    /// Construct from 8-bit channels (0..255).
    ///
    /// Values are divided by 255.0 to produce the 0..1 float representation.
    ///
    /// ```
    /// use optic_color::*;
    /// let c = RGBA::from_bytes(255, 136, 0, 255);
    /// ```
    pub fn from_bytes(r: u8, g: u8, b: u8, a: u8) -> Self {
        RGBA(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }

    /// Lighten by a fixed amount in HSV value space.
    ///
    /// Positive `amount` increases value; negative decreases it.
    /// The result is clamped to 0..1. Alpha is preserved.
    ///
    /// ```
    /// use optic_color::*;
    /// let lighter = RED.lighten(0.2);
    /// ```
    pub fn lighten(self, amount: f32) -> RGBA {
        let mut hsv: HSV = HSV::from_rgba(self);
        hsv.v = (hsv.v + amount).clamp(0.0, 1.0);
        hsv.to_rgba().with_alpha(self.3)
    }

    /// Darken by a fixed amount in HSV value space.
    ///
    /// Equivalent to `lighten(-amount)`.
    pub fn darken(self, amount: f32) -> RGBA {
        self.lighten(-amount)
    }

    /// Increase saturation by a fixed amount in HSV space.
    ///
    /// Positive `amount` increases saturation; negative decreases it.
    /// The result is clamped to 0..1. Alpha is preserved.
    pub fn saturate(self, amount: f32) -> RGBA {
        let mut hsv: HSV = HSV::from_rgba(self);
        hsv.s = (hsv.s + amount).clamp(0.0, 1.0);
        hsv.to_rgba().with_alpha(self.3)
    }

    /// Decrease saturation by a fixed amount in HSV space.
    ///
    /// Equivalent to `saturate(-amount)`.
    pub fn desaturate(self, amount: f32) -> RGBA {
        self.saturate(-amount)
    }

    /// Invert the RGB channels (alpha unchanged).
    ///
    /// Each channel becomes `1.0 - channel`.
    ///
    /// ```
    /// use optic_color::*;
    /// let inv = WHITE.invert();
    /// assert_eq!(inv.0, 0.0); // BLACK
    /// ```
    pub fn invert(self) -> RGBA {
        RGBA(1.0 - self.0, 1.0 - self.1, 1.0 - self.2, self.3)
    }

    /// Convert from sRGB display encoding to linear light (EOTF).
    ///
    /// Applies the sRGB gamma expansion curve. Use this before doing
    /// physically based lighting calculations.
    pub fn to_linear(self) -> RGBA {
        fn srgb_eotf(c: f32) -> f32 {
            if c <= 0.04045 { c / 12.92 }
            else { ((c + 0.055) / 1.055).powf(2.4) }
        }
        RGBA(srgb_eotf(self.0), srgb_eotf(self.1), srgb_eotf(self.2), self.3)
    }

    /// Convert from linear light to sRGB display encoding (OETF).
    ///
    /// Applies the sRGB gamma compression curve. Use this before writing
    /// to a framebuffer that expects sRGB.
    pub fn to_srgb(self) -> RGBA {
        fn srgb_oetf(c: f32) -> f32 {
            if c <= 0.0031308 { c * 12.92 }
            else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
        }
        RGBA(srgb_oetf(self.0), srgb_oetf(self.1), srgb_oetf(self.2), self.3)
    }
}
