//! Predefined color constants.
//!
//! This module re-exports a curated palette of named [`RGBA`] constants for
//! quick reference. All colors are fully opaque (`alpha = 1.0`).
//!
//! ```rust
//! use optic_color::*;
//!
//! let sky = SKY;
//! assert!(sky.2 > 0.9); // high blue channel
//! ```

use crate::RGBA;

// ── Reds ──────────────────────────────────────────────────────────────

/// Pure red (1.0, 0.0, 0.0, 1.0).
pub const RED: RGBA = RGBA(1.0, 0.0, 0.0, 1.0);
/// Deep red with a blue undertone (0.86, 0.08, 0.24, 1.0).
pub const CRIMSON: RGBA = RGBA(0.86, 0.08, 0.24, 1.0);
/// Light pink (1.0, 0.75, 0.8, 1.0).
pub const PINK: RGBA = RGBA(1.0, 0.75, 0.8, 1.0);
/// Warm pink (1.0, 0.4, 0.5, 1.0).
pub const BLUSH: RGBA = RGBA(1.0, 0.4, 0.5, 1.0);
/// Pinkish orange (1.0, 0.5, 0.31, 1.0).
pub const CORAL: RGBA = RGBA(1.0, 0.5, 0.31, 1.0);
/// Vivid red-orange (1.0, 0.14, 0.0, 1.0).
pub const SCARLET: RGBA = RGBA(1.0, 0.14, 0.0, 1.0);
/// Dark brownish red (0.5, 0.0, 0.0, 1.0).
pub const MAROON: RGBA = RGBA(0.5, 0.0, 0.0, 1.0);
/// Deep purplish red (0.6, 0.0, 0.13, 1.0).
pub const BURGUNDY: RGBA = RGBA(0.6, 0.0, 0.13, 1.0);
/// Light orange-pink (0.98, 0.5, 0.45, 1.0).
pub const SALMON: RGBA = RGBA(0.98, 0.5, 0.45, 1.0);
/// Very dark red (0.4, 0.0, 0.04, 1.0).
pub const ROSEWOOD: RGBA = RGBA(0.4, 0.0, 0.04, 1.0);
/// Dark reddish brown (0.65, 0.16, 0.16, 1.0).
pub const MAHOGANY: RGBA = RGBA(0.65, 0.16, 0.16, 1.0);

// ── Oranges ───────────────────────────────────────────────────────────

/// Pure orange (1.0, 0.65, 0.0, 1.0).
pub const ORANGE: RGBA = RGBA(1.0, 0.65, 0.0, 1.0);
/// Golden amber (1.0, 0.75, 0.0, 1.0).
pub const AMBER: RGBA = RGBA(1.0, 0.75, 0.0, 1.0);
/// Bright gold (1.0, 0.84, 0.0, 1.0).
pub const GOLD: RGBA = RGBA(1.0, 0.84, 0.0, 1.0);
/// Bright orange (1.0, 0.58, 0.0, 1.0).
pub const TANGERINE: RGBA = RGBA(1.0, 0.58, 0.0, 1.0);

// ── Yellows ───────────────────────────────────────────────────────────

/// Pure yellow (1.0, 1.0, 0.0, 1.0).
pub const YELLOW: RGBA = RGBA(1.0, 1.0, 0.0, 1.0);
/// Yellow-orange (1.0, 0.75, 0.2, 1.0).
pub const MANGO: RGBA = RGBA(1.0, 0.75, 0.2, 1.0);
/// Warm yellow (1.0, 0.86, 0.35, 1.0).
pub const MUSTARD: RGBA = RGBA(1.0, 0.86, 0.35, 1.0);

// ── Greens ────────────────────────────────────────────────────────────

/// Bright yellow-green (0.75, 1.0, 0.0, 1.0).
pub const LIME: RGBA = RGBA(0.75, 1.0, 0.0, 1.0);
/// Vivid green-cyan (0.0, 1.0, 0.5, 1.0).
pub const SPRING: RGBA = RGBA(0.0, 1.0, 0.5, 1.0);
/// Muted sea green (0.18, 0.55, 0.34, 1.0).
pub const SEA: RGBA = RGBA(0.18, 0.55, 0.34, 1.0);
/// Dark forest green (0.13, 0.55, 0.13, 1.0).
pub const FOREST: RGBA = RGBA(0.13, 0.55, 0.13, 1.0);
/// Pure green (0.0, 1.0, 0.0, 1.0).
pub const GREEN: RGBA = RGBA(0.0, 1.0, 0.0, 1.0);
/// Medium fern green (0.42, 0.74, 0.42, 1.0).
pub const FERN: RGBA = RGBA(0.42, 0.74, 0.42, 1.0);
/// Dark yellow-green (0.5, 0.5, 0.0, 1.0).
pub const OLIVE: RGBA = RGBA(0.5, 0.5, 0.0, 1.0);
/// Pale jade green (0.67, 0.88, 0.69, 1.0).
pub const CELADON: RGBA = RGBA(0.67, 0.88, 0.69, 1.0);
/// Light mint green (0.74, 1.0, 0.85, 1.0).
pub const MINT: RGBA = RGBA(0.74, 1.0, 0.85, 1.0);

// ── Cyans & Blues ─────────────────────────────────────────────────────

/// Dark cyan (0.0, 0.5, 0.5, 1.0).
pub const TEAL: RGBA = RGBA(0.0, 0.5, 0.5, 1.0);
/// Bright aqua (0.25, 0.88, 0.82, 1.0).
pub const AQUA: RGBA = RGBA(0.25, 0.88, 0.82, 1.0);
/// Light sky blue (0.53, 0.81, 0.92, 1.0).
pub const SKY: RGBA = RGBA(0.53, 0.81, 0.92, 1.0);
/// Pure cyan (0.0, 1.0, 1.0, 1.0).
pub const CYAN: RGBA = RGBA(0.0, 1.0, 1.0, 1.0);
/// Pure blue (0.0, 0.0, 1.0, 1.0).
pub const BLUE: RGBA = RGBA(0.0, 0.0, 1.0, 1.0);
/// Very dark blue (0.1, 0.1, 0.44, 1.0).
pub const MIDNIGHT: RGBA = RGBA(0.1, 0.1, 0.44, 1.0);
/// Deep indigo (0.29, 0.0, 0.51, 1.0).
pub const INDIGO: RGBA = RGBA(0.29, 0.0, 0.51, 1.0);
/// Bright turquoise (0.25, 0.88, 0.82, 1.0).
pub const TURQUOISE: RGBA = RGBA(0.25, 0.88, 0.82, 1.0);
/// Dark cobalt blue (0.0, 0.28, 0.67, 1.0).
pub const COBALT: RGBA = RGBA(0.0, 0.28, 0.67, 1.0);
/// Dark navy blue (0.0, 0.0, 0.5, 1.0).
pub const NAVY: RGBA = RGBA(0.0, 0.0, 0.5, 1.0);
/// Medium lapis blue (0.15, 0.38, 0.61, 1.0).
pub const LAPIS: RGBA = RGBA(0.15, 0.38, 0.61, 1.0);

// ── Purples & Magentas ────────────────────────────────────────────────

/// Medium purple (0.5, 0.0, 0.5, 1.0).
pub const PURPLE: RGBA = RGBA(0.5, 0.0, 0.5, 1.0);
/// Light purple (0.87, 0.63, 0.87, 1.0).
pub const PLUM: RGBA = RGBA(0.87, 0.63, 0.87, 1.0);
/// Dark bluish purple (0.25, 0.22, 0.45, 1.0).
pub const DUSK: RGBA = RGBA(0.25, 0.22, 0.45, 1.0);
/// Pure magenta (1.0, 0.0, 1.0, 1.0).
pub const MAGENTA: RGBA = RGBA(1.0, 0.0, 1.0, 1.0);
/// Soft lavender (0.9, 0.9, 0.98, 1.0).
pub const LAVENDER: RGBA = RGBA(0.9, 0.9, 0.98, 1.0);
/// Vivid violet (0.56, 0.0, 1.0, 1.0).
pub const VIOLET: RGBA = RGBA(0.56, 0.0, 1.0, 1.0);
/// Soft lavender (0.79, 0.63, 0.86, 1.0).
pub const WISTERIA: RGBA = RGBA(0.79, 0.63, 0.86, 1.0);
/// Medium reddish purple (0.77, 0.29, 0.55, 1.0).
pub const MULBERRY: RGBA = RGBA(0.77, 0.29, 0.55, 1.0);

// ── Neutrals ──────────────────────────────────────────────────────────

/// Mid grey (0.5, 0.5, 0.5, 1.0).
pub const GRAY: RGBA = RGBA(0.5, 0.5, 0.5, 1.0);
/// Light grey (0.75, 0.75, 0.75, 1.0).
pub const SILVER: RGBA = RGBA(0.75, 0.75, 0.75, 1.0);
/// Pure white (1.0, 1.0, 1.0, 1.0).
pub const WHITE: RGBA = RGBA(1.0, 1.0, 1.0, 1.0);
/// Pure black (0.0, 0.0, 0.0, 1.0).
pub const BLACK: RGBA = RGBA(0.0, 0.0, 0.0, 1.0);
/// Near-black with a blue tint (0.05, 0.05, 0.08, 1.0).
pub const OBSIDIAN: RGBA = RGBA(0.05, 0.05, 0.08, 1.0);
/// Blue-grey (0.44, 0.5, 0.56, 1.0).
pub const SLATE: RGBA = RGBA(0.44, 0.5, 0.56, 1.0);
/// Dark grey (0.21, 0.27, 0.31, 1.0).
pub const CHARCOAL: RGBA = RGBA(0.21, 0.27, 0.31, 1.0);

// ── Warm Neutrals & Earth Tones ───────────────────────────────────────

/// Off-white with a warm yellow tint (1.0, 1.0, 0.94, 1.0).
pub const IVORY: RGBA = RGBA(1.0, 1.0, 0.94, 1.0);
/// Pale warm grey (0.94, 0.92, 0.84, 1.0).
pub const ALABASTER: RGBA = RGBA(0.94, 0.92, 0.84, 1.0);
/// Slightly warm white (1.0, 0.98, 0.98, 1.0).
pub const SNOW: RGBA = RGBA(1.0, 0.98, 0.98, 1.0);
/// Warm light pink (1.0, 0.85, 0.73, 1.0).
pub const PEACH: RGBA = RGBA(1.0, 0.85, 0.73, 1.0);
/// Soft orange-pink (0.98, 0.81, 0.69, 1.0).
pub const APRICOT: RGBA = RGBA(0.98, 0.81, 0.69, 1.0);
/// Dark reddish brown (0.65, 0.16, 0.16, 1.0).
pub const BROWN: RGBA = RGBA(0.65, 0.16, 0.16, 1.0);
/// Muted yellowish grey (0.76, 0.69, 0.57, 1.0).
pub const KHAKI: RGBA = RGBA(0.76, 0.69, 0.57, 1.0);
/// Pale warm off-white (0.96, 0.96, 0.86, 1.0).
pub const BEIGE: RGBA = RGBA(0.96, 0.96, 0.86, 1.0);
/// Muted yellow-grey (0.76, 0.7, 0.5, 1.0).
pub const SAND: RGBA = RGBA(0.76, 0.7, 0.5, 1.0);
/// Warm metallic brown (0.72, 0.45, 0.2, 1.0).
pub const COPPER: RGBA = RGBA(0.72, 0.45, 0.2, 1.0);
/// Warm metallic gold-brown (0.8, 0.5, 0.2, 1.0).
pub const BRONZE: RGBA = RGBA(0.8, 0.5, 0.2, 1.0);
