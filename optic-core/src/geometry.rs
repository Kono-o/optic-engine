use std::ops::{Add, Mul, Sub};

/// Trait for types whose components can be accessed as a fixed-size array.
///
/// This is the geometric analogue of [`ChannelArray`] in `optic-color`. It
/// enables generic componentwise operations (`min`, `max`) across different
/// geometric types.
///
/// Implemented for [`Size2D`], [`Size3D`], [`Coord2D`], [`CoordOffset`].
///
/// [`ChannelArray`]: optic_color::ChannelArray
pub trait Components<T, const N: usize>: Copy {
    /// Convert to an array of length N.
    fn to_array(self) -> [T; N];
    /// Construct from an array of length N.
    fn from_array(a: [T; N]) -> Self;
}

/// Componentwise minimum of two values.
///
/// Each component of the result is the minimum of the corresponding
/// components of `a` and `b`.
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

/// Componentwise maximum of two values.
///
/// Each component of the result is the maximum of the corresponding
/// components of `a` and `b`.
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

/// A 2D size with non-negative integer dimensions.
///
/// ```
/// use optic_core::*;
///
/// let s = Size2D::from(1920, 1080);
/// assert_eq!(s.aspect_ratio(), 16.0 / 9.0);
/// ```
///
/// Supports [`Add`], [`Sub`] (saturating), and [`Mul<f32>`] componentwise.
/// Conversion from/to `[u32; 2]` and `(u32, u32)` via [`Components`].
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
    /// Zero-size (0, 0).
    pub fn empty() -> Size2D {
        Self { w: 0, h: 0 }
    }
    /// Construct from explicit width and height.
    pub fn from(w: u32, h: u32) -> Self {
        Self { w, h }
    }
    /// Reduce both dimensions by `n` (saturating).
    pub fn shave(&self, n: u32) -> Size2D {
        Size2D {
            w: self.w.saturating_sub(n),
            h: self.h.saturating_sub(n),
        }
    }
    /// Aspect ratio as `w / h` (f32). Returns 0 if height is 0.
    pub fn aspect_ratio(&self) -> f32 {
        self.w as f32 / self.h.max(1) as f32
    }
    /// True if either dimension is zero.
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0
    }
    /// Area in pixels (`w * h` as u64).
    pub fn area(&self) -> u64 {
        self.w as u64 * self.h as u64
    }
    /// Componentwise minimum.
    pub fn min(&self, other: Size2D) -> Size2D {
        componentwise_min(*self, other)
    }
    /// Componentwise maximum.
    pub fn max(&self, other: Size2D) -> Size2D {
        componentwise_max(*self, other)
    }
    /// Scale down to fit within `max` while preserving aspect ratio.
    ///
    /// If already within bounds, returns unchanged.
    pub fn fit_within(&self, max: Size2D) -> Size2D {
        if self.w <= max.w && self.h <= max.h { return *self; }
        let scale = (max.w as f32 / self.w as f32).min(max.h as f32 / self.h as f32);
        *self * scale
    }
    /// Scale to a specific width, preserving aspect ratio.
    pub fn scaled_to_width(&self, w: u32) -> Size2D {
        let scale = w as f32 / self.w.max(1) as f32;
        *self * scale
    }
    /// Scale to a specific height, preserving aspect ratio.
    pub fn scaled_to_height(&self, h: u32) -> Size2D {
        let scale = h as f32 / self.h.max(1) as f32;
        *self * scale
    }
    /// Convert to a [`Size3D`] with the given depth.
    pub fn to_size3d(&self, depth: u32) -> Size3D {
        Size3D { w: self.w, h: self.h, d: depth }
    }
}

/// A 3D size with non-negative integer dimensions.
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
    /// Zero-size (0, 0, 0).
    pub fn empty() -> Size3D {
        Self { w: 0, h: 0, d: 0 }
    }
    /// Construct from width, height, and depth.
    pub fn from(w: u32, h: u32, d: u32) -> Self {
        Self { w, h, d }
    }
    /// Reduce all three dimensions by `n` (saturating).
    pub fn shave(&self, n: u32) -> Size3D {
        Size3D {
            w: self.w.saturating_sub(n),
            h: self.h.saturating_sub(n),
            d: self.d.saturating_sub(n),
        }
    }
    /// True if any dimension is zero.
    pub fn is_empty(&self) -> bool {
        self.w == 0 || self.h == 0 || self.d == 0
    }
    /// Volume as u64 (`w * h * d`).
    pub fn volume(&self) -> u64 {
        self.w as u64 * self.h as u64 * self.d as u64
    }
    /// Componentwise minimum.
    pub fn min(&self, other: Size3D) -> Size3D {
        componentwise_min(*self, other)
    }
    /// Componentwise maximum.
    pub fn max(&self, other: Size3D) -> Size3D {
        componentwise_max(*self, other)
    }
    /// Drop the depth component, returning a [`Size2D`].
    pub fn to_size2d(&self) -> Size2D {
        Size2D { w: self.w, h: self.h }
    }
}

/// Near/far clip plane distances for a camera.
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

/// Camera projection mode.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CamProj {
    Ortho,
    Persp,
}
