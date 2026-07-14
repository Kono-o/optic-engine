//! ANSI terminal colour constants for log output.
//!
//! This module provides pre-built [`ANSI`] escape-sequence pairs that can
//! be passed to the [`log_color!`](crate::log_color) macro (or used
//! directly with `print!`/`eprint!`) to produce coloured terminal output.
//!
//! Constants are organised into six groups:
//!
//! | Group | Example | Description |
//! |---|---|---|
//! | Foreground | [`RED`] | Standard foreground colours |
//! | Bold foreground | [`BOLD_RED`] | Bold/bright foreground colours |
//! | Dark foreground | [`DARK_RED`] | "Bright" ANSI foreground (high-intensity) |
//! | Bold dark foreground | [`BOLD_DARK_RED`] | Bold + high-intensity foreground |
//! | Background | [`BG_RED`] | Standard background colours |
//! | Bold background | [`BOLD_BG_RED`] | Bold/bright background colours |
//! | Dark background | [`BG_DARK_RED`] | High-intensity background colours |
//! | Bold dark background | [`BOLD_BG_DARK_RED`] | Bold + high-intensity background |

/// ANSI terminal escape pair for colored output.
///
/// Used with the [`log_color!`](crate::log_color) macro.
///
/// Each constant combines a SGR prefix (e.g. `\x1b[31m` for red foreground)
/// and a reset suffix (`\x1b[0m`).
pub struct ANSI {
    /// SGR escape sequence that begins the colour (e.g. `\x1b[31m`).
    pub prefix: &'static str,
    /// SGR escape sequence that resets attributes back to default.
    pub suffix: &'static str,
}

// ── Standard foreground ──────────────────────────────────────────────

/// Standard red foreground text.
pub const RED: ANSI = ANSI { prefix: "\x1b[31m", suffix: "\x1b[0m" };
/// Standard green foreground text.
pub const GREEN: ANSI = ANSI { prefix: "\x1b[32m", suffix: "\x1b[0m" };
/// Standard yellow foreground text.
pub const YELLOW: ANSI = ANSI { prefix: "\x1b[33m", suffix: "\x1b[0m" };
/// Standard blue foreground text.
pub const BLUE: ANSI = ANSI { prefix: "\x1b[34m", suffix: "\x1b[0m" };
/// Standard magenta foreground text.
pub const MAGENTA: ANSI = ANSI { prefix: "\x1b[35m", suffix: "\x1b[0m" };
/// Standard cyan foreground text.
pub const CYAN: ANSI = ANSI { prefix: "\x1b[36m", suffix: "\x1b[0m" };

// ── Bold foreground ──────────────────────────────────────────────────

/// Bold red foreground text.
pub const BOLD_RED: ANSI = ANSI { prefix: "\x1b[1m\x1b[31m", suffix: "\x1b[0m" };
/// Bold green foreground text.
pub const BOLD_GREEN: ANSI = ANSI { prefix: "\x1b[1m\x1b[32m", suffix: "\x1b[0m" };
/// Bold yellow foreground text.
pub const BOLD_YELLOW: ANSI = ANSI { prefix: "\x1b[1m\x1b[33m", suffix: "\x1b[0m" };
/// Bold blue foreground text.
pub const BOLD_BLUE: ANSI = ANSI { prefix: "\x1b[1m\x1b[34m", suffix: "\x1b[0m" };
/// Bold magenta foreground text.
pub const BOLD_MAGENTA: ANSI = ANSI { prefix: "\x1b[1m\x1b[35m", suffix: "\x1b[0m" };
/// Bold cyan foreground text.
pub const BOLD_CYAN: ANSI = ANSI { prefix: "\x1b[1m\x1b[36m", suffix: "\x1b[0m" };

// ── Dark (high-intensity) foreground ─────────────────────────────────

/// High-intensity (bright) red foreground text.
pub const DARK_RED: ANSI = ANSI { prefix: "\x1b[91m", suffix: "\x1b[0m" };
/// High-intensity (bright) green foreground text.
pub const DARK_GREEN: ANSI = ANSI { prefix: "\x1b[92m", suffix: "\x1b[0m" };
/// High-intensity (bright) yellow foreground text.
pub const DARK_YELLOW: ANSI = ANSI { prefix: "\x1b[93m", suffix: "\x1b[0m" };
/// High-intensity (bright) blue foreground text.
pub const DARK_BLUE: ANSI = ANSI { prefix: "\x1b[94m", suffix: "\x1b[0m" };
/// High-intensity (bright) magenta foreground text.
pub const DARK_MAGENTA: ANSI = ANSI { prefix: "\x1b[95m", suffix: "\x1b[0m" };
/// High-intensity (bright) cyan foreground text.
pub const DARK_CYAN: ANSI = ANSI { prefix: "\x1b[96m", suffix: "\x1b[0m" };

// ── Bold dark (high-intensity) foreground ────────────────────────────

/// Bold high-intensity red foreground text.
pub const BOLD_DARK_RED: ANSI = ANSI { prefix: "\x1b[1m\x1b[91m", suffix: "\x1b[0m" };
/// Bold high-intensity green foreground text.
pub const BOLD_DARK_GREEN: ANSI = ANSI { prefix: "\x1b[1m\x1b[92m", suffix: "\x1b[0m" };
/// Bold high-intensity yellow foreground text.
pub const BOLD_DARK_YELLOW: ANSI = ANSI { prefix: "\x1b[1m\x1b[93m", suffix: "\x1b[0m" };
/// Bold high-intensity blue foreground text.
pub const BOLD_DARK_BLUE: ANSI = ANSI { prefix: "\x1b[1m\x1b[94m", suffix: "\x1b[0m" };
/// Bold high-intensity magenta foreground text.
pub const BOLD_DARK_MAGENTA: ANSI = ANSI { prefix: "\x1b[1m\x1b[95m", suffix: "\x1b[0m" };
/// Bold high-intensity cyan foreground text.
pub const BOLD_DARK_CYAN: ANSI = ANSI { prefix: "\x1b[1m\x1b[96m", suffix: "\x1b[0m" };

// ── Standard background ──────────────────────────────────────────────

/// Red background fill.
pub const BG_RED: ANSI = ANSI { prefix: "\x1b[41m", suffix: "\x1b[0m" };
/// Green background fill.
pub const BG_GREEN: ANSI = ANSI { prefix: "\x1b[42m", suffix: "\x1b[0m" };
/// Yellow background fill.
pub const BG_YELLOW: ANSI = ANSI { prefix: "\x1b[43m", suffix: "\x1b[0m" };
/// Blue background fill.
pub const BG_BLUE: ANSI = ANSI { prefix: "\x1b[44m", suffix: "\x1b[0m" };
/// Magenta background fill.
pub const BG_MAGENTA: ANSI = ANSI { prefix: "\x1b[45m", suffix: "\x1b[0m" };
/// Cyan background fill.
pub const BG_CYAN: ANSI = ANSI { prefix: "\x1b[46m", suffix: "\x1b[0m" };

// ── Bold background ──────────────────────────────────────────────────

/// Bold red background fill.
pub const BOLD_BG_RED: ANSI = ANSI { prefix: "\x1b[1m\x1b[41m", suffix: "\x1b[0m" };
/// Bold green background fill.
pub const BOLD_BG_GREEN: ANSI = ANSI { prefix: "\x1b[1m\x1b[42m", suffix: "\x1b[0m" };
/// Bold yellow background fill.
pub const BOLD_BG_YELLOW: ANSI = ANSI { prefix: "\x1b[1m\x1b[43m", suffix: "\x1b[0m" };
/// Bold blue background fill.
pub const BOLD_BG_BLUE: ANSI = ANSI { prefix: "\x1b[1m\x1b[44m", suffix: "\x1b[0m" };
/// Bold magenta background fill.
pub const BOLD_BG_MAGENTA: ANSI = ANSI { prefix: "\x1b[1m\x1b[45m", suffix: "\x1b[0m" };
/// Bold cyan background fill.
pub const BOLD_BG_CYAN: ANSI = ANSI { prefix: "\x1b[1m\x1b[46m", suffix: "\x1b[0m" };

// ── Dark (high-intensity) background ─────────────────────────────────

/// High-intensity (bright) red background fill.
pub const BG_DARK_RED: ANSI = ANSI { prefix: "\x1b[101m", suffix: "\x1b[0m" };
/// High-intensity (bright) green background fill.
pub const BG_DARK_GREEN: ANSI = ANSI { prefix: "\x1b[102m", suffix: "\x1b[0m" };
/// High-intensity (bright) yellow background fill.
pub const BG_DARK_YELLOW: ANSI = ANSI { prefix: "\x1b[103m", suffix: "\x1b[0m" };
/// High-intensity (bright) blue background fill.
pub const BG_DARK_BLUE: ANSI = ANSI { prefix: "\x1b[104m", suffix: "\x1b[0m" };
/// High-intensity (bright) magenta background fill.
pub const BG_DARK_MAGENTA: ANSI = ANSI { prefix: "\x1b[105m", suffix: "\x1b[0m" };
/// High-intensity (bright) cyan background fill.
pub const BG_DARK_CYAN: ANSI = ANSI { prefix: "\x1b[106m", suffix: "\x1b[0m" };

// ── Bold dark (high-intensity) background ────────────────────────────

/// Bold high-intensity red background fill.
pub const BOLD_BG_DARK_RED: ANSI = ANSI { prefix: "\x1b[1m\x1b[101m", suffix: "\x1b[0m" };
/// Bold high-intensity green background fill.
pub const BOLD_BG_DARK_GREEN: ANSI = ANSI { prefix: "\x1b[1m\x1b[102m", suffix: "\x1b[0m" };
/// Bold high-intensity yellow background fill.
pub const BOLD_BG_DARK_YELLOW: ANSI = ANSI { prefix: "\x1b[1m\x1b[103m", suffix: "\x1b[0m" };
/// Bold high-intensity blue background fill.
pub const BOLD_BG_DARK_BLUE: ANSI = ANSI { prefix: "\x1b[1m\x1b[104m", suffix: "\x1b[0m" };
/// Bold high-intensity magenta background fill.
pub const BOLD_BG_DARK_MAGENTA: ANSI = ANSI { prefix: "\x1b[1m\x1b[105m", suffix: "\x1b[0m" };
/// Bold high-intensity cyan background fill.
pub const BOLD_BG_DARK_CYAN: ANSI = ANSI { prefix: "\x1b[1m\x1b[106m", suffix: "\x1b[0m" };
