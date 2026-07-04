//! Zero-dependency color library for the Optic engine.
//!
//! This crate provides color types, conversions, channel arithmetic, and
//! a gradient evaluator. It has no external dependencies and can be used
//! independently of the rest of the engine.
//!
//! # Types
//!
//! | Type  | Components | Channels | Arithmetic |
//! |-------|------------|----------|------------|
//! | [`RGBA`] | red, green, blue, alpha | [`ChannelArray<4>`] | Add, Sub, Mul, Div |
//! | [`RGB`]  | red, green, blue        | [`ChannelArray<3>`] | Add, Sub, Mul, Div |
//! | [`HSV`]  | hue, saturation, value  | — | — |
//! | [`HSL`]  | hue, saturation, lightness | — | — |
//!
//! `HSV` and `HSL` deliberately avoid arithmetic because hue wraparound
//! makes componentwise operations produce wrong colors. Convert to [`RGBA`]
//! first (via `.into()` or [`ToRgba`]), then operate, then convert back.
//!
//! # Conversions
//!
//! Every color type implements [`ToRgba`] and [`FromRgba`], so you can use
//! generics that accept "any color-like type":
//!
//! ```
//! use optic_color::*;
//!
//! fn set_bg(color: impl ToRgba) {
//!     let rgba = color.to_rgba();
//!     // ...
//! }
//! ```
//!
//! Direct `From` impls exist for all pairs:
//! - `From<RGB/HSV/HSL> for RGBA`
//! - `From<RGBA> for RGB/HSV/HSL`
//!
//! # Gradients
//!
//! [`Gradient`] supports multiple interpolation modes, color spaces,
//! and wrap modes:
//!
//! ```
//! use optic_color::*;
//!
//! let grad = Gradient::two_color(RED, BLUE)
//!     .set_color_space(GradientColorSpace::Hsv);
//!
//! let mid = grad.sample(0.5); // RGBA
//! ```
//!
//! # Named colors
//!
//! The crate exports ~90 named [`RGBA`] constants (see [`optic_color::constants`]).
//! Examples: [`RED`], [`MIDNIGHT`], [`GOLD`], [`LAVENDER`], [`OBSIDIAN`].

mod channels;
mod constants;
mod convert;
mod gradient;
mod hsl;
mod hsv;
mod rgb;
mod rgba;

pub use channels::*;
pub use constants::*;
pub use gradient::*;
pub use hsl::*;
pub use hsv::*;
pub use rgb::*;
pub use rgba::*;

pub use convert::{ColorInfo, FromRgba, ToRgba};
