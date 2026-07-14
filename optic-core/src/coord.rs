//! 2D coordinate types with point vs. vector semantics.
//!
//! Optic distinguishes between **points** ([`Coord2D`]) and **vectors**
//! ([`CoordOffset`]) to prevent accidental misuse:
//!
//! - [`Coord2D`] — an absolute position in 2D space. Subtracting two
//!   points produces a [`CoordOffset`].
//! - [`CoordOffset`] — a displacement or direction. Adding a vector to a
//!   point produces a new point.
//!
//! Both types implement [`Components<f64, 2>`] and support conversion to
//! and from `[f64; 2]` arrays.

use std::ops::{Add, Mul, Neg, Sub};

use crate::{componentwise_min, componentwise_max, Components, Size2D};

/// A 2D point in continuous space.
///
/// `Coord2D` represents a position. Subtracting two points yields a
/// [`CoordOffset`] (vector). Adding a vector to a point yields a new point.
///
/// ```
/// use optic_core::*;
///
/// let a = Coord2D::new(100.0, 200.0);
/// let b = Coord2D::new(150.0, 180.0);
/// let d: CoordOffset = b - a;
/// let mid = a.lerp(b, 0.5);
/// ```
///
/// Implements [`Components<f64, 2>`].
#[derive(Copy, Clone, Debug)]
pub struct Coord2D {
    pub x: f64,
    pub y: f64,
}

impl Components<f64, 2> for Coord2D {
    fn to_array(self) -> [f64; 2] { [self.x, self.y] }
    fn from_array(a: [f64; 2]) -> Self { Coord2D { x: a[0], y: a[1] } }
}

impl From<[f64; 2]> for Coord2D { fn from(a: [f64; 2]) -> Self { Coord2D::from_array(a) } }
impl From<Coord2D> for [f64; 2] { fn from(c: Coord2D) -> Self { c.to_array() } }
impl From<(f64, f64)> for Coord2D { fn from(t: (f64, f64)) -> Self { Coord2D { x: t.0, y: t.1 } } }

// Point - Point = Vector
impl Sub for Coord2D {
    type Output = CoordOffset;
    fn sub(self, rhs: Coord2D) -> CoordOffset {
        CoordOffset { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

// Point + Vector = Point, Point - Vector = Point
impl Add<CoordOffset> for Coord2D {
    type Output = Coord2D;
    fn add(self, rhs: CoordOffset) -> Coord2D {
        Coord2D { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
impl Sub<CoordOffset> for Coord2D {
    type Output = Coord2D;
    fn sub(self, rhs: CoordOffset) -> Coord2D {
        Coord2D { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}

impl Coord2D {
    /// The origin point (0, 0).
    pub fn zero() -> Coord2D {
        Coord2D { x: 0.0, y: 0.0 }
    }
    /// Construct from x, y coordinates.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    /// True if the point lies within the rectangle `(0, 0)` to `(size.w, size.h)`.
    pub fn is_inside(&self, size: Size2D) -> bool {
        let [w, h] = size.to_array();
        self.x >= 0.0 && self.y >= 0.0 && self.x <= w as f64 && self.y <= h as f64
    }
    /// Euclidean distance to another point.
    pub fn distance_to(&self, other: Coord2D) -> f64 {
        (*self - other).length()
    }
    /// Midpoint between two points.
    pub fn midpoint(&self, other: Coord2D) -> Coord2D {
        Coord2D { x: (self.x + other.x) * 0.5, y: (self.y + other.y) * 0.5 }
    }
    /// Linearly interpolate toward `other` by factor `t` (clamped 0..1).
    pub fn lerp(&self, other: Coord2D, t: f64) -> Coord2D {
        *self + (other - *self) * t.clamp(0.0, 1.0)
    }
    /// Componentwise minimum.
    pub fn min(&self, other: Coord2D) -> Coord2D {
        componentwise_min(*self, other)
    }
    /// Componentwise maximum.
    pub fn max(&self, other: Coord2D) -> Coord2D {
        componentwise_max(*self, other)
    }
}

/// A 2D vector/displacement.
///
/// `CoordOffset` represents the difference between two [`Coord2D`] points.
/// It supports vector arithmetic: addition, subtraction, scalar multiplication,
/// negation, normalization, and dot products.
#[derive(Copy, Clone, Debug)]
pub struct CoordOffset {
    pub x: f64,
    pub y: f64,
}

impl Components<f64, 2> for CoordOffset {
    fn to_array(self) -> [f64; 2] { [self.x, self.y] }
    fn from_array(a: [f64; 2]) -> Self { CoordOffset { x: a[0], y: a[1] } }
}

impl From<[f64; 2]> for CoordOffset { fn from(a: [f64; 2]) -> Self { CoordOffset::from_array(a) } }
impl From<CoordOffset> for [f64; 2] { fn from(v: CoordOffset) -> Self { v.to_array() } }
impl From<(f64, f64)> for CoordOffset { fn from(t: (f64, f64)) -> Self { CoordOffset { x: t.0, y: t.1 } } }

impl Add for CoordOffset {
    type Output = CoordOffset;
    fn add(self, rhs: CoordOffset) -> CoordOffset {
        CoordOffset { x: self.x + rhs.x, y: self.y + rhs.y }
    }
}
impl Sub for CoordOffset {
    type Output = CoordOffset;
    fn sub(self, rhs: CoordOffset) -> CoordOffset {
        CoordOffset { x: self.x - rhs.x, y: self.y - rhs.y }
    }
}
impl Mul<f64> for CoordOffset {
    type Output = CoordOffset;
    fn mul(self, rhs: f64) -> CoordOffset {
        CoordOffset { x: self.x * rhs, y: self.y * rhs }
    }
}
impl Neg for CoordOffset {
    type Output = CoordOffset;
    fn neg(self) -> CoordOffset {
        CoordOffset { x: -self.x, y: -self.y }
    }
}

impl CoordOffset {
    /// Zero vector (0, 0).
    pub fn zero() -> CoordOffset {
        CoordOffset { x: 0.0, y: 0.0 }
    }
    /// Construct from x, y components.
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    /// True if both components are exactly zero.
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
    /// Euclidean length.
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    /// Squared length (avoids the sqrt).
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
    /// Unit vector in the same direction. Returns zero if length is near-zero.
    pub fn normalize(&self) -> CoordOffset {
        let len = self.length();
        if len < f64::EPSILON { return CoordOffset { x: 0.0, y: 0.0 }; }
        CoordOffset { x: self.x / len, y: self.y / len }
    }
    /// Dot product with another vector.
    pub fn dot(&self, other: CoordOffset) -> f64 {
        self.x * other.x + self.y * other.y
    }
    /// Linearly interpolate toward `other` by factor `t` (clamped 0..1).
    pub fn lerp(&self, other: CoordOffset, t: f64) -> CoordOffset {
        *self + (other - *self) * t.clamp(0.0, 1.0)
    }
    /// Componentwise minimum.
    pub fn min(&self, other: CoordOffset) -> CoordOffset {
        componentwise_min(*self, other)
    }
    /// Componentwise maximum.
    pub fn max(&self, other: CoordOffset) -> CoordOffset {
        componentwise_max(*self, other)
    }
    /// Angle (radians) from the positive X axis to this vector.
    pub fn angle(&self) -> f64 {
        self.y.atan2(self.x)
    }
    /// Perpendicular vector (rotated 90° counter-clockwise).
    pub fn perpendicular(&self) -> CoordOffset {
        CoordOffset { x: -self.y, y: self.x }
    }
    /// Reflect this vector across a surface with the given normal.
    pub fn reflect(&self, normal: CoordOffset) -> CoordOffset {
        *self - normal * 2.0 * self.dot(normal)
    }
    /// Project this vector onto another.
    pub fn project(&self, onto: CoordOffset) -> CoordOffset {
        onto * (self.dot(onto) / onto.length_squared())
    }
}
