#[derive(Copy, Clone, Debug)]
pub struct RGBA(pub f32, pub f32, pub f32, pub f32);

#[derive(Copy, Clone, Debug)]
pub struct RGB(pub f32, pub f32, pub f32);

impl RGBA {
    pub fn grey(lum: f32) -> Self {
        Self(lum, lum, lum, 1.0)
    }
    pub fn from_rgb(rgb: RGB, alpha: f32) -> Self {
        Self(rgb.0, rgb.1, rgb.2, alpha)
    }
    pub fn to_rgb(&self) -> RGB {
        RGB(self.0, self.1, self.2)
    }
}

impl RGB {
    pub fn grey(lum: f32) -> Self {
        Self(lum, lum, lum)
    }
    pub fn from_rgba(rgba: RGBA) -> Self {
        Self(rgba.0, rgba.1, rgba.2)
    }
    pub fn to_rgba(&self, alpha: f32) -> RGBA {
        RGBA(self.0, self.1, self.2, alpha)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgba_grey() {
        let c = RGBA::grey(0.5);
        assert_eq!(c.0, 0.5);
        assert_eq!(c.1, 0.5);
        assert_eq!(c.2, 0.5);
        assert_eq!(c.3, 1.0);
    }

    #[test]
    fn rgb_grey() {
        let c = RGB::grey(0.3);
        assert_eq!(c.0, 0.3);
        assert_eq!(c.1, 0.3);
        assert_eq!(c.2, 0.3);
    }

    #[test]
    fn rgba_from_rgb() {
        let rgb = RGB(0.1, 0.2, 0.3);
        let rgba = RGBA::from_rgb(rgb, 0.5);
        assert_eq!(rgba.0, 0.1);
        assert_eq!(rgba.1, 0.2);
        assert_eq!(rgba.2, 0.3);
        assert_eq!(rgba.3, 0.5);
    }

    #[test]
    fn rgba_to_rgb() {
        let rgba = RGBA(0.1, 0.2, 0.3, 0.5);
        let rgb = rgba.to_rgb();
        assert_eq!(rgb.0, 0.1);
        assert_eq!(rgb.1, 0.2);
        assert_eq!(rgb.2, 0.3);
    }

    #[test]
    fn rgb_from_rgba() {
        let rgba = RGBA(0.1, 0.2, 0.3, 0.5);
        let rgb = RGB::from_rgba(rgba);
        assert_eq!(rgb.0, 0.1);
        assert_eq!(rgb.1, 0.2);
        assert_eq!(rgb.2, 0.3);
    }

    #[test]
    fn rgb_to_rgba() {
        let rgb = RGB(0.1, 0.2, 0.3);
        let rgba = rgb.to_rgba(0.8);
        assert_eq!(rgba.0, 0.1);
        assert_eq!(rgba.1, 0.2);
        assert_eq!(rgba.2, 0.3);
        assert_eq!(rgba.3, 0.8);
    }
}
