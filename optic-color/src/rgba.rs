use crate::{ColorInfo, FromRgba, HSV, RGB, ToRgba};

#[derive(Copy, Clone, Debug)]
pub struct RGBA(pub f32, pub f32, pub f32, pub f32);

impl RGBA {
    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self { RGBA(r, g, b, a) }

    pub fn grey(lum: f32) -> Self { RGBA(lum, lum, lum, 1.0) }

    pub fn from_rgb(rgb: RGB, alpha: f32) -> Self { RGBA(rgb.0, rgb.1, rgb.2, alpha) }

    pub fn to_rgb(&self) -> RGB { RGB(self.0, self.1, self.2) }

    pub fn with_alpha(self, a: f32) -> RGBA { RGBA(self.0, self.1, self.2, a) }

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

    pub fn from_hex_u32(hex: u32) -> Self {
        let r = ((hex >> 24) & 0xFF) as u8;
        let g = ((hex >> 16) & 0xFF) as u8;
        let b = ((hex >> 8) & 0xFF) as u8;
        let a = (hex & 0xFF) as u8;
        RGBA::from_bytes(r, g, b, a)
    }

    pub fn to_hex_u32(self) -> u32 {
        let (r, g, b, a) = self.to_bytes();
        (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | a as u32
    }

    pub fn from_bytes(r: u8, g: u8, b: u8, a: u8) -> Self {
        RGBA(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, a as f32 / 255.0)
    }

    pub fn lighten(self, amount: f32) -> RGBA {
        let mut hsv: HSV = HSV::from_rgba(self);
        hsv.v = (hsv.v + amount).clamp(0.0, 1.0);
        hsv.to_rgba().with_alpha(self.3)
    }

    pub fn darken(self, amount: f32) -> RGBA {
        self.lighten(-amount)
    }

    pub fn saturate(self, amount: f32) -> RGBA {
        let mut hsv: HSV = HSV::from_rgba(self);
        hsv.s = (hsv.s + amount).clamp(0.0, 1.0);
        hsv.to_rgba().with_alpha(self.3)
    }

    pub fn desaturate(self, amount: f32) -> RGBA {
        self.saturate(-amount)
    }

    pub fn invert(self) -> RGBA {
        RGBA(1.0 - self.0, 1.0 - self.1, 1.0 - self.2, self.3)
    }

    pub fn to_linear(self) -> RGBA {
        fn srgb_eotf(c: f32) -> f32 {
            if c <= 0.04045 { c / 12.92 }
            else { ((c + 0.055) / 1.055).powf(2.4) }
        }
        RGBA(srgb_eotf(self.0), srgb_eotf(self.1), srgb_eotf(self.2), self.3)
    }

    pub fn to_srgb(self) -> RGBA {
        fn srgb_oetf(c: f32) -> f32 {
            if c <= 0.0031308 { c * 12.92 }
            else { 1.055 * c.powf(1.0 / 2.4) - 0.055 }
        }
        RGBA(srgb_oetf(self.0), srgb_oetf(self.1), srgb_oetf(self.2), self.3)
    }
}
