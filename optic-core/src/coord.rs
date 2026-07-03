use std::ops::{Add, Mul, Neg, Sub};

use crate::{componentwise_min, componentwise_max, Components, Size2D};

// ── Coord2D (point) ─────────────────────────────────────────────────────────

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
    pub fn empty() -> Coord2D {
        Coord2D { x: 0.0, y: 0.0 }
    }
    pub fn from(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn from_tup((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
    pub fn is_inside(&self, size: Size2D) -> bool {
        let [w, h] = size.to_array();
        self.x >= 0.0 && self.y >= 0.0 && self.x <= w as f64 && self.y <= h as f64
    }
    pub fn distance_to(&self, other: Coord2D) -> f64 {
        (*self - other).length()
    }
    pub fn midpoint(&self, other: Coord2D) -> Coord2D {
        Coord2D { x: (self.x + other.x) * 0.5, y: (self.y + other.y) * 0.5 }
    }
    pub fn lerp(&self, other: Coord2D, t: f64) -> Coord2D {
        *self + (other - *self) * t.clamp(0.0, 1.0)
    }
    pub fn min(&self, other: Coord2D) -> Coord2D {
        componentwise_min(*self, other)
    }
    pub fn max(&self, other: Coord2D) -> Coord2D {
        componentwise_max(*self, other)
    }
}

// ── CoordOffset (vector/displacement) ───────────────────────────────────────

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

// Vector + Vector = Vector, Vector - Vector = Vector
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
    pub fn empty() -> CoordOffset {
        CoordOffset { x: 0.0, y: 0.0 }
    }
    pub fn from(x: f64, y: f64) -> Self {
        Self { x, y }
    }
    pub fn from_tup((x, y): (f64, f64)) -> Self {
        Self { x, y }
    }
    pub fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0
    }
    pub fn length(&self) -> f64 {
        (self.x * self.x + self.y * self.y).sqrt()
    }
    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }
    pub fn normalize(&self) -> CoordOffset {
        let len = self.length();
        if len < f64::EPSILON { return CoordOffset { x: 0.0, y: 0.0 }; }
        CoordOffset { x: self.x / len, y: self.y / len }
    }
    pub fn dot(&self, other: CoordOffset) -> f64 {
        self.x * other.x + self.y * other.y
    }
    pub fn lerp(&self, other: CoordOffset, t: f64) -> CoordOffset {
        *self + (other - *self) * t.clamp(0.0, 1.0)
    }
    pub fn min(&self, other: CoordOffset) -> CoordOffset {
        componentwise_min(*self, other)
    }
    pub fn max(&self, other: CoordOffset) -> CoordOffset {
        componentwise_max(*self, other)
    }
}
