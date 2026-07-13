//! Text rendering pipeline — BBCode parsing, glyph layout, and instance
//! descriptors for [`Text2D`] and [`Text3D`].
//!
//! # Overview
//!
//! Text is rendered by decomposing a BBCode string into styled spans, shaping
//! glyphs against the loaded [`FontFamily`](crate::FontFamily), laying them
//! out into quads, and uploading the quads as GPU instance buffers.
//!
//! The pipeline is:
//!
//! ```text
//! raw BBCode string
//!   → bbcode::parse()     → ParsedText (styled spans)
//!   → layout::layout_text() → TextLayout (positioned glyphs + decorations)
//!   → build_glyph_desc_*()  → InstanceDesc2D / InstanceDesc3D
//!   → desc.upload()         → InstanceBuffer
//!   → GPU instanced draw
//! ```
//!
//! # Supported BBCode tags
//!
//! | Tag | Syntax | Effect |
//! |-----|--------|--------|
//! | `[b]` | `[b]text[/b]` | Bold (real atlas or faux) |
//! | `[i]` | `[i]text[/i]` | Italic (real atlas or faux) |
//! | `[color=#rrggbbaa]` | `[color=#ff0000ff]red[/color]` | Foreground color |
//! | `[bgcolor=#rrggbbaa]` | `[bgcolor=#00000080]bg[/bgcolor]` | Background highlight |
//! | `[size=N]` | `[size=2.0]big[/size]` | Size multiplier |
//! | `[s]` | `[s]struck[/s]` | Strikethrough |
//! | `[u]` | `[u]under[/u]` | Underline |
//! | `[border=#rrggbbaa,W]` | `[border=#000000ff,2]outline[/border]` | Border outline |
//! | `[kerning=N]` | `[kerning=0.5]spaced[/kerning]` | Extra kerning |
//! | `[offset=x,y]` | `[offset=2,-1]shifted[/offset]` | Pixel offset |
//! | `[wave]` | `[wave amp=4,freq=0.1,speed=2]` | Sine wave Y displacement |
//! | `[shake]` | `[shake amp=2,speed=8]` | Random XY shake |
//! | `[rainbow]` | `[rainbow speed=1]` | HSV rainbow color cycle |
//! | `[pulse]` | `[pulse amp=0.2,speed=3]` | Sine scale pulse |
//!
//! Tags can be nested. Dynamic tags (`wave`, `shake`, `rainbow`, `pulse`)
//! require calling [`Text2D::update`] / [`Text3D::update`] each frame.

pub mod bbcode;
pub mod layout;

pub use bbcode::*;
pub use layout::*;
