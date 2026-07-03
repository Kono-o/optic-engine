use std::ops::{Add, Mul, Sub};

// ── Components trait ─────────────────────────────────────────────────────────

pub trait Components<T, const N: usize>: Copy {
    fn to_array(self) -> [T; N];
    fn from_array(a: [T; N]) -> Self;
}

pub fn componentwise_min<T: PartialOrd + Copy, C: Components<T, N>, const N: usize>(a: C, b: C) -> C {
    let (a, b) = (a.to_array(), b.to_array());
    let mut out = a;
    for i in 0..N {
        if b[i] < out[i] {
            out[i] = b[i];
        }
    }
    C::from_array(out)
}

pub fn componentwise_max<T: PartialOrd + Copy, C: Components<T, N>, const N: usize>(a: C, b: C) -> C {
    let (a, b) = (a.to_array(), b.to_array());
    let mut out = a;
    for i in 0..N {
        if b[i] > out[i] {
            out[i] = b[i];
        }
    }
    C::from_array(out)
}

// ── Size2D ───────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size2D {
    pub w: u32,
    pub h: u32,
}

impl Components<u32, 2> for Size2D {
    fn to_array(self) -> [u32; 2] { [self.w, self.h] }
    fn from_array(a: [u32; 2]) -> Self { Size2D { w: a[0], h: a[1] } }
}

impl From<[u32; 2]> for Size2D { fn from(a: [u32; 2]) -> Self { Size2D::from_array(a) } }
impl From<Size2D> for [u32; 2] { fn from(s: Size2D) -> Self { s.to_array() } }
impl From<(u32, u32)> for Size2D { fn from(t: (u32, u32)) -> Self { Size2D { w: t.0, h: t.1 } } }

macro_rules! impl_size_ops {
    ($ty:ty, $n:literal) => {
        impl Add for $ty {
            type Output = Self;
            fn add(self, rhs: Self) -> Self {
                let (a, b) = (self.to_array(), rhs.to_array());
                let mut out = [0u32; $n];
                for i in 0..$n { out[i] = a[i].saturating_add(b[i]); }
                Self::from_array(out)
            }
        }
        impl Sub for $ty {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self {
                let (a, b) = (self.to_array(), rhs.to_array());
                let mut out = [0u32; $n];
                for i in 0..$n { out[i] = a[i].saturating_sub(b[i]); }
                Self::from_array(out)
            }
        }
        impl Mul<f32> for $ty {
            type Output = Self;
            fn mul(self, rhs: f32) -> Self {
                let a = self.to_array();
                let mut out = [0u32; $n];
                for i in 0..$n { out[i] = ((a[i] as f32) * rhs).round().max(0.0) as u32; }
                Self::from_array(out)
            }
        }
    };
}

impl_size_ops!(Size2D, 2);

impl Size2D {
    pub fn empty() -> Size2D {
        Self { w: 0, h: 0 }
    }
    pub fn from(w: u32, h: u32) -> Self {
        Self { w, h }
    }
    pub fn shave(&self, n: u32) -> Size2D {
        Size2D {
            w: self.w.saturating_sub(n),
            h: self.h.saturating_sub(n),
        }
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h.max(1) as f32
    }
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
    pub fn area(&self) -> u64 {
        self.w as u64 * self.h as u64
    }
    pub fn min(&self, other: Size2D) -> Size2D {
        componentwise_min(*self, other)
    }
    pub fn max(&self, other: Size2D) -> Size2D {
        componentwise_max(*self, other)
    }
    pub fn fit_within(&self, max: Size2D) -> Size2D {
        if self.w <= max.w && self.h <= max.h { return *self; }
        let scale = (max.w as f32 / self.w as f32).min(max.h as f32 / self.h as f32);
        *self * scale
    }
    pub fn scaled_to_width(&self, w: u32) -> Size2D {
        let scale = w as f32 / self.w.max(1) as f32;
        *self * scale
    }
    pub fn scaled_to_height(&self, h: u32) -> Size2D {
        let scale = h as f32 / self.h.max(1) as f32;
        *self * scale
    }
    pub fn to_size3d(&self, depth: u32) -> Size3D {
        Size3D { w: self.w, h: self.h, d: depth }
    }
}

// ── Size3D ───────────────────────────────────────────────────────────────────

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size3D {
    pub w: u32,
    pub h: u32,
    pub d: u32,
}

impl Components<u32, 3> for Size3D {
    fn to_array(self) -> [u32; 3] { [self.w, self.h, self.d] }
    fn from_array(a: [u32; 3]) -> Self { Size3D { w: a[0], h: a[1], d: a[2] } }
}

impl From<[u32; 3]> for Size3D { fn from(a: [u32; 3]) -> Self { Size3D::from_array(a) } }
impl From<Size3D> for [u32; 3] { fn from(s: Size3D) -> Self { s.to_array() } }
impl From<(u32, u32, u32)> for Size3D { fn from(t: (u32, u32, u32)) -> Self { Size3D { w: t.0, h: t.1, d: t.2 } } }

impl_size_ops!(Size3D, 3);

impl Size3D {
    pub fn empty() -> Size3D {
        Self { w: 0, h: 0, d: 0 }
    }
    pub fn from(w: u32, h: u32, d: u32) -> Self {
        Self { w, h, d }
    }
    pub fn shave(&self, n: u32) -> Size3D {
        Size3D {
            w: self.w.saturating_sub(n),
            h: self.h.saturating_sub(n),
            d: self.d.saturating_sub(n),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0 || self.d == 0
    }
    pub fn volume(&self) -> u64 {
        self.w as u64 * self.h as u64 * self.d as u64
    }
    pub fn min(&self, other: Size3D) -> Size3D {
        componentwise_min(*self, other)
    }
    pub fn max(&self, other: Size3D) -> Size3D {
        componentwise_max(*self, other)
    }
    pub fn to_size2d(&self) -> Size2D {
        Size2D { w: self.w, h: self.h }
    }
}

// ── ClipDist ─────────────────────────────────────────────────────────────────

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

// ── CamProj ──────────────────────────────────────────────────────────────────

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CamProj {
    Ortho,
    Persp,
}

// ── Tests ────────────────────────────────────────────────────────────────────

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
    fn size2d_shave_saturating() {
        let s = Size2D::from(3, 3).shave(10);
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
    fn size2d_is_empty() {
        assert!(Size2D::empty().is_empty());
        assert!(Size2D::from(0, 100).is_empty());
        assert!(Size2D::from(100, 0).is_empty());
        assert!(!Size2D::from(100, 100).is_empty());
    }

    #[test]
    fn size2d_area() {
        let s = Size2D::from(1920, 1080);
        assert_eq!(s.area(), 1920 * 1080);
    }

    #[test]
    fn size2d_min_max() {
        let a = Size2D::from(100, 200);
        let b = Size2D::from(300, 50);
        assert_eq!(a.min(b), Size2D::from(100, 50));
        assert_eq!(a.max(b), Size2D::from(300, 200));
    }

    #[test]
    fn size2d_fit_within_noop() {
        let s = Size2D::from(100, 100);
        let max = Size2D::from(200, 200);
        assert_eq!(s.fit_within(max), s);
    }

    #[test]
    fn size2d_fit_within_scales() {
        let s = Size2D::from(1920, 1080);
        let max = Size2D::from(512, 512);
        let fitted = s.fit_within(max);
        assert!(fitted.w <= 512);
        assert!(fitted.h <= 512);
        let orig_ratio = s.w as f32 / s.h as f32;
        let fit_ratio = fitted.w as f32 / fitted.h as f32;
        assert!((orig_ratio - fit_ratio).abs() < 0.01);
    }

    #[test]
    fn size2d_scaled_to_width() {
        let s = Size2D::from(100, 50).scaled_to_width(200);
        assert_eq!(s.w, 200);
        assert_eq!(s.h, 100);
    }

    #[test]
    fn size2d_scaled_to_height() {
        let s = Size2D::from(100, 50).scaled_to_height(100);
        assert_eq!(s.h, 100);
        assert_eq!(s.w, 200);
    }

    #[test]
    fn size2d_add_saturating() {
        let a = Size2D::from(100, 200);
        let b = Size2D::from(50, u32::MAX);
        let c = a + b;
        assert_eq!(c.w, 150);
        assert_eq!(c.h, u32::MAX);
    }

    #[test]
    fn size2d_sub_saturating() {
        let a = Size2D::from(5, 5);
        let b = Size2D::from(10, 10);
        let c = a - b;
        assert_eq!(c.w, 0);
        assert_eq!(c.h, 0);
    }

    #[test]
    fn size2d_mul_scalar() {
        let s = Size2D::from(100, 50) * 1.5;
        assert_eq!(s.w, 150);
        assert_eq!(s.h, 75);
    }

    #[test]
    fn size2d_array_roundtrip() {
        let s = Size2D::from(800, 600);
        let arr: [u32; 2] = s.into();
        let back: Size2D = arr.into();
        assert_eq!(s, back);
    }

    #[test]
    fn size2d_tuple_roundtrip() {
        let s: Size2D = (800, 600).into();
        assert_eq!(s.w, 800);
        assert_eq!(s.h, 600);
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
    fn size3d_is_empty() {
        assert!(Size3D::empty().is_empty());
        assert!(Size3D::from(0, 1, 1).is_empty());
        assert!(!Size3D::from(1, 1, 1).is_empty());
    }

    #[test]
    fn size3d_volume() {
        let s = Size3D::from(10, 20, 30);
        assert_eq!(s.volume(), 6000);
    }

    #[test]
    fn size3d_sub_saturating() {
        let a = Size3D::from(5, 5, 5);
        let b = Size3D::from(10, 10, 10);
        let c = a - b;
        assert_eq!(c.w, 0);
        assert_eq!(c.h, 0);
        assert_eq!(c.d, 0);
    }

    #[test]
    fn size2d_to_size3d() {
        let s2 = Size2D::from(800, 600);
        let s3 = s2.to_size3d(256);
        assert_eq!(s3.w, 800);
        assert_eq!(s3.h, 600);
        assert_eq!(s3.d, 256);
    }

    #[test]
    fn size3d_to_size2d() {
        let s3 = Size3D::from(800, 600, 256);
        let s2 = s3.to_size2d();
        assert_eq!(s2.w, 800);
        assert_eq!(s2.h, 600);
    }

    #[test]
    fn size3d_array_roundtrip() {
        let s = Size3D::from(10, 20, 30);
        let arr: [u32; 3] = s.into();
        let back: Size3D = arr.into();
        assert_eq!(s, back);
    }

    #[test]
    fn size3d_tuple_roundtrip() {
        let s: Size3D = (10, 20, 30).into();
        assert_eq!(s.w, 10);
        assert_eq!(s.h, 20);
        assert_eq!(s.d, 30);
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
