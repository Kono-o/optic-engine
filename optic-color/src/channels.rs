//! Channel-wise operations and the [`ChannelArray`] trait.
//!
//! This module provides the [`ChannelArray`] trait and a set of free
//! functions for performing element-wise arithmetic on color channels:
//!
//! - [`channel_lerp`] — linear interpolation
//! - [`channel_add`], [`channel_sub`], [`channel_mul`] — component-wise math
//! - [`channel_mul_scalar`], [`channel_div_scalar`] — scalar operations
//!
//! [`RGBA`] and [`RGB`] both implement [`ChannelArray`] and gain
//! `Add`, `Sub`, `Mul`, and `Div` operators via a macro.

use crate::{RGB, RGBA};

/// Trait for types whose channels can be accessed as a fixed-size float array.
///
/// This is the foundation for channel arithmetic in the color system.
/// Implementations exist for [`RGBA`] (4 channels) and [`RGB`] (3 channels).
///
/// The trait enables generic channel-wise operations:
///
/// ```
/// use optic_color::*;
///
/// fn half_brightness<T: ChannelArray<N>, const N: usize>(c: T) -> T {
///     channel_mul_scalar(c, 0.5)
/// }
/// ```
///
/// See also: [`channel_lerp`], [`channel_add`], [`channel_mul`].
pub trait ChannelArray<const N: usize>: Copy {
    /// Convert to a float array of length N.
    fn to_array(self) -> [f32; N];
    /// Construct from a float array of length N.
    fn from_array(arr: [f32; N]) -> Self;
}

impl ChannelArray<4> for RGBA {
    fn to_array(self) -> [f32; 4] { [self.0, self.1, self.2, self.3] }
    fn from_array(a: [f32; 4]) -> Self { RGBA(a[0], a[1], a[2], a[3]) }
}

impl ChannelArray<3> for RGB {
    fn to_array(self) -> [f32; 3] { [self.0, self.1, self.2] }
    fn from_array(a: [f32; 3]) -> Self { RGB(a[0], a[1], a[2]) }
}

/// Linearly interpolate between two colors channel by channel.
///
/// `t` is clamped to 0..1.
///
/// ```
/// use optic_color::*;
///
/// let mid = channel_lerp(RED, BLUE, 0.5);
/// ```
pub fn channel_lerp<T: ChannelArray<N>, const N: usize>(a: T, b: T, t: f32) -> T {
    let (a, b) = (a.to_array(), b.to_array());
    let t = t.clamp(0.0, 1.0);
    let mut out = [0.0f32; N];
    for i in 0..N { out[i] = a[i] + (b[i] - a[i]) * t; }
    T::from_array(out)
}

/// Channel-wise addition of two colors.
pub fn channel_add<T: ChannelArray<N>, const N: usize>(a: T, b: T) -> T {
    let (a, b) = (a.to_array(), b.to_array());
    let mut out = [0.0f32; N];
    for i in 0..N { out[i] = a[i] + b[i]; }
    T::from_array(out)
}

/// Channel-wise subtraction of two colors.
pub fn channel_sub<T: ChannelArray<N>, const N: usize>(a: T, b: T) -> T {
    let (a, b) = (a.to_array(), b.to_array());
    let mut out = [0.0f32; N];
    for i in 0..N { out[i] = a[i] - b[i]; }
    T::from_array(out)
}

/// Channel-wise multiplication of two colors.
pub fn channel_mul<T: ChannelArray<N>, const N: usize>(a: T, b: T) -> T {
    let (a, b) = (a.to_array(), b.to_array());
    let mut out = [0.0f32; N];
    for i in 0..N { out[i] = a[i] * b[i]; }
    T::from_array(out)
}

/// Multiply all channels by a scalar.
pub fn channel_mul_scalar<T: ChannelArray<N>, const N: usize>(a: T, s: f32) -> T {
    let a = a.to_array();
    let mut out = [0.0f32; N];
    for i in 0..N { out[i] = a[i] * s; }
    T::from_array(out)
}

/// Divide all channels by a scalar.
///
/// Equivalent to `channel_mul_scalar(a, 1.0 / s)`.
pub fn channel_div_scalar<T: ChannelArray<N>, const N: usize>(a: T, s: f32) -> T {
    channel_mul_scalar(a, 1.0 / s)
}

macro_rules! impl_channel_ops {
    ($ty:ty, $n:literal) => {
        impl std::ops::Add for $ty {
            type Output = Self;
            fn add(self, rhs: Self) -> Self { channel_add(self, rhs) }
        }
        impl std::ops::Sub for $ty {
            type Output = Self;
            fn sub(self, rhs: Self) -> Self { channel_sub(self, rhs) }
        }
        impl std::ops::Mul<f32> for $ty {
            type Output = Self;
            fn mul(self, rhs: f32) -> Self { channel_mul_scalar(self, rhs) }
        }
        impl std::ops::Mul for $ty {
            type Output = Self;
            fn mul(self, rhs: Self) -> Self { channel_mul(self, rhs) }
        }
        impl std::ops::Div<f32> for $ty {
            type Output = Self;
            fn div(self, rhs: f32) -> Self { channel_div_scalar(self, rhs) }
        }
        impl From<[f32; $n]> for $ty {
            fn from(arr: [f32; $n]) -> Self { Self::from_array(arr) }
        }
        impl From<$ty> for [f32; $n] {
            fn from(c: $ty) -> Self { c.to_array() }
        }
    };
}

impl_channel_ops!(RGBA, 4);
impl_channel_ops!(RGB, 3);

impl From<(f32, f32, f32, f32)> for RGBA {
    fn from(t: (f32, f32, f32, f32)) -> Self { RGBA(t.0, t.1, t.2, t.3) }
}

impl From<(f32, f32, f32)> for RGB {
    fn from(t: (f32, f32, f32)) -> Self { RGB(t.0, t.1, t.2) }
}
