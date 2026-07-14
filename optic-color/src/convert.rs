//! Color space conversions and the conversion traits.
//!
//! This module defines the two core conversion traits:
//!
//! - [`ToRgba`] — lossless conversion *to* [`RGBA`].
//! - [`FromRgba`] — construction *from* [`RGBA`].
//!
//! Every color type ([`RGB`], [`HSV`], [`HSL`]) implements both traits, and
//! the corresponding [`From`] impls are provided so that `.into()` works in
//! both directions. Internal conversion functions (e.g. `hsv_to_rgba`) are
//! `pub(crate)` and not part of the public API.

use crate::{HSL, HSV, RGB, RGBA};

/// Trait for types that can be converted to RGBA losslessly.
///
/// This is the primary conversion trait in the color system. Any type
/// implementing `ToRgba` can be used as a color argument anywhere in
/// the engine:
///
/// ```
/// use optic_color::*;
///
/// fn set_color(c: impl ToRgba) {
///     let rgba = c.to_rgba();
///     // ...
/// }
///
/// set_color(RED);
/// set_color(HSV::new(200.0, 0.8, 0.9));
/// ```
///
/// Implemented for [`RGBA`], [`RGB`], [`HSV`], [`HSL`].
pub trait ToRgba: Copy {
    /// Convert to RGBA.
    fn to_rgba(self) -> RGBA;
}

/// Trait for types that can be constructed from RGBA.
///
/// Implemented for [`RGBA`], [`RGB`], [`HSV`], [`HSL`].
pub trait FromRgba: Sized {
    /// Construct this color type from an [`RGBA`].
    fn from_rgba(rgba: RGBA) -> Self;
}

impl ToRgba for RGBA {
    fn to_rgba(self) -> RGBA { self }
}

impl FromRgba for RGBA {
    fn from_rgba(rgba: RGBA) -> Self { rgba }
}

impl ToRgba for RGB {
    fn to_rgba(self) -> RGBA { RGBA(self.0, self.1, self.2, 1.0) }
}

impl FromRgba for RGB {
    fn from_rgba(rgba: RGBA) -> Self { RGB(rgba.0, rgba.1, rgba.2) }
}

impl ToRgba for HSV {
    fn to_rgba(self) -> RGBA { hsv_to_rgba(self) }
}

impl FromRgba for HSV {
    fn from_rgba(rgba: RGBA) -> Self { rgba_to_hsv(rgba) }
}

impl ToRgba for HSL {
    fn to_rgba(self) -> RGBA { hsl_to_rgba(self) }
}

impl FromRgba for HSL {
    fn from_rgba(rgba: RGBA) -> Self { rgba_to_hsl(rgba) }
}

impl From<RGB> for RGBA { fn from(rgb: RGB) -> Self { rgb.to_rgba() } }
impl From<HSV> for RGBA { fn from(hsv: HSV) -> Self { hsv.to_rgba() } }
impl From<HSL> for RGBA { fn from(hsl: HSL) -> Self { hsl.to_rgba() } }
impl From<RGBA> for RGB { fn from(rgba: RGBA) -> Self { RGB::from_rgba(rgba) } }
impl From<RGBA> for HSV { fn from(rgba: RGBA) -> Self { HSV::from_rgba(rgba) } }
impl From<RGBA> for HSL { fn from(rgba: RGBA) -> Self { HSL::from_rgba(rgba) } }

pub(crate) fn hsv_to_rgba(hsv: HSV) -> RGBA {
    let h = hsv.h / 60.0;
    let s = hsv.s;
    let v = hsv.v;
    let i = h.floor() as i32;
    let f = h - h.floor();
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    let (r, g, b) = match i % 6 {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        5 => (v, p, q),
        _ => (v, p, q),
    };
    RGBA(r, g, b, 1.0)
}

pub(crate) fn rgba_to_hsv(rgba: RGBA) -> HSV {
    let r = rgba.0;
    let g = rgba.1;
    let b = rgba.2;
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let d = mx - mn;
    let h = if d == 0.0 {
        0.0
    } else if mx == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if mx == g {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    let s = if mx == 0.0 { 0.0 } else { d / mx };
    HSV { h: h.clamp(0.0, 360.0), s: s.clamp(0.0, 1.0), v: mx.clamp(0.0, 1.0) }
}

pub(crate) fn hsl_to_rgba(hsl: HSL) -> RGBA {
    let h = hsl.h / 360.0;
    let s = hsl.s;
    let l = hsl.l;
    if s == 0.0 {
        return RGBA(l, l, l, 1.0);
    }
    fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { p + (q - p) * 6.0 * t }
        else if t < 1.0 / 2.0 { q }
        else if t < 2.0 / 3.0 { p + (q - p) * (2.0 / 3.0 - t) * 6.0 }
        else { p }
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);
    RGBA(r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0), 1.0)
}

pub(crate) fn rgba_to_hsl(rgba: RGBA) -> HSL {
    let r = rgba.0;
    let g = rgba.1;
    let b = rgba.2;
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let d = mx - mn;
    let l = (mx + mn) / 2.0;
    let s = if d == 0.0 { 0.0 } else { d / (1.0 - (2.0 * l - 1.0).abs()) };
    let h = if d == 0.0 {
        0.0
    } else if mx == r {
        60.0 * (((g - b) / d) % 6.0)
    } else if mx == g {
        60.0 * (((b - r) / d) + 2.0)
    } else {
        60.0 * (((r - g) / d) + 4.0)
    };
    let h = if h < 0.0 { h + 360.0 } else { h };
    HSL { h: h.clamp(0.0, 360.0), s: s.clamp(0.0, 1.0), l: l.clamp(0.0, 1.0) }
}

/// Trait for computing luminance, contrast, and hex/byte serialization.
///
/// This trait has a blanket impl for all [`ToRgba`] types, so every color
/// type gets these methods automatically:
///
/// ```
/// use optic_color::*;
///
/// let c = RGB(0.5, 0.2, 0.8);
/// let lum = c.luminance();
/// let hex = c.to_hex();
/// let (r, g, b, a) = c.to_bytes();
/// ```
pub trait ColorInfo: ToRgba {
    /// Relative luminance per ITU-R BT.709.
    ///
    /// Uses the standard coefficients: 0.2126 R + 0.7152 G + 0.0722 B.
    fn luminance(self) -> f32 {
        let c = self.to_rgba();
        0.2126 * c.0 + 0.7152 * c.1 + 0.0722 * c.2
    }

    /// Returns true if the luminance is greater than 0.5.
    fn is_light(self) -> bool { self.luminance() > 0.5 }

    /// Compute the WCAG contrast ratio against another color.
    ///
    /// The result is a value in 1..21. WCAG AA requires 4.5:1 for normal
    /// text; WCAG AAA requires 7:1.
    fn contrast_ratio(self, other: impl ToRgba) -> f32 {
        let l1 = self.luminance();
        let l2 = other.luminance();
        let (lighter, darker) = if l1 > l2 { (l1, l2) } else { (l2, l1) };
        (lighter + 0.05) / (darker + 0.05)
    }

    /// Encode as a hex string: `#RRGGBBAA`.
    fn to_hex(self) -> String {
        let (r, g, b, a) = self.to_bytes();
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    }

    /// Convert to 8-bit byte channels: `(r, g, b, a)` in 0..255.
    fn to_bytes(self) -> (u8, u8, u8, u8) {
        let c = self.to_rgba();
        let r = (c.0.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (c.1.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (c.2.clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (c.3.clamp(0.0, 1.0) * 255.0).round() as u8;
        (r, g, b, a)
    }
}

impl<T: ToRgba> ColorInfo for T {}
