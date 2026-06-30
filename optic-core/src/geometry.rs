#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size2D {
    pub w: u32,
    pub h: u32,
}

impl Size2D {
    pub fn empty() -> Size2D {
        Self { w: 0, h: 0 }
    }
    pub fn from(w: u32, h: u32) -> Self {
        Self { w, h }
    }
    pub fn shave(&self, n: u32) -> Size2D {
        if self.w > 0 && self.h > 0 {
            Size2D {
                w: self.w - n,
                h: self.h - n,
            }
        } else {
            *self
        }
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h as f32
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size3D {
    pub w: u32,
    pub h: u32,
    pub d: u32,
}

impl Size3D {
    pub fn empty() -> Size3D {
        Self { w: 0, h: 0, d: 0 }
    }
    pub fn from(w: u32, h: u32, d: u32) -> Self {
        Self { w, h, d }
    }
    pub fn shave(&self, n: u32) -> Size3D {
        if self.w > 0 && self.h > 0 && self.d > 0 {
            Size3D {
                w: self.w - n,
                h: self.h - n,
                d: self.d - n,
            }
        } else {
            *self
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ClipDist {
    pub near: f32,
    pub far: f32,
}

impl Default for ClipDist {
    fn default() -> Self {
        ClipDist::from(0.01, 1000.0)
    }
}

impl ClipDist {
    pub fn from(near: f32, far: f32) -> ClipDist {
        ClipDist { near, far }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CamProj {
    Ortho,
    Persp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size2d_empty() {
        let s = Size2D::empty();
        assert_eq!(s.w, 0);
        assert_eq!(s.h, 0);
    }

    #[test]
    fn size2d_from() {
        let s = Size2D::from(800, 600);
        assert_eq!(s.w, 800);
        assert_eq!(s.h, 600);
    }

    #[test]
    fn size2d_shave() {
        let s = Size2D::from(100, 80).shave(10);
        assert_eq!(s.w, 90);
        assert_eq!(s.h, 70);
    }

    #[test]
    fn size2d_shave_zero() {
        let s = Size2D::empty().shave(5);
        assert_eq!(s.w, 0);
        assert_eq!(s.h, 0);
    }

    #[test]
    fn size2d_aspect_ratio() {
        let s = Size2D::from(1920, 1080);
        let ratio = s.aspect_ratio();
        assert!((ratio - 16.0 / 9.0).abs() < 0.001);
    }

    #[test]
    fn size3d_empty() {
        let s = Size3D::empty();
        assert_eq!(s.w, 0);
        assert_eq!(s.h, 0);
        assert_eq!(s.d, 0);
    }

    #[test]
    fn size3d_from() {
        let s = Size3D::from(10, 20, 30);
        assert_eq!(s.w, 10);
        assert_eq!(s.h, 20);
        assert_eq!(s.d, 30);
    }

    #[test]
    fn size3d_shave() {
        let s = Size3D::from(100, 80, 60).shave(5);
        assert_eq!(s.w, 95);
        assert_eq!(s.h, 75);
        assert_eq!(s.d, 55);
    }

    #[test]
    fn clipdist_default() {
        let c = ClipDist::default();
        assert!((c.near - 0.01).abs() < f32::EPSILON);
        assert!((c.far - 1000.0).abs() < f32::EPSILON);
    }

    #[test]
    fn clipdist_from() {
        let c = ClipDist::from(0.1, 500.0);
        assert!((c.near - 0.1).abs() < f32::EPSILON);
        assert!((c.far - 500.0).abs() < f32::EPSILON);
    }
}
