use crate::RGBA;

#[derive(Copy, Clone, Debug)]
pub struct RGB(pub f32, pub f32, pub f32);

impl RGB {
    pub fn grey(lum: f32) -> Self { RGB(lum, lum, lum) }

    pub fn from_rgba(rgba: RGBA) -> Self { RGB(rgba.0, rgba.1, rgba.2) }

    pub fn to_rgba(&self, alpha: f32) -> RGBA { RGBA(self.0, self.1, self.2, alpha) }
}
