//! Asset loading, caching, and GPU upload.
//!
//! This module is the heart of the Optic asset pipeline. It handles everything
//! from reading source files on disk, through binary cache generation, to
//! producing GPU-ready handles. Every asset type follows the same pattern:
//! parse → bake → cache → upload.
//!
//! # How Asset Caching Works
//!
//! Optic uses **runtime binary caching** — not build-time asset compilation.
//! There are no `build.rs` scripts, no CLI baking tools, and no asset manifest
//! files. Caching happens transparently inside `from_disk()`:
//!
//! ```text
//! ┌──────────────────────────────────────────────────────────────┐
//! │                  Debug Build                                 │
//! │                                                              │
//! │  Source file ──parse/bake──▶ CPU data ──save──▶ Binary cache │
//! │  (PNG/OBJ/GLSL/TTF/OGG)                   (optc/*.otxtr…)  │
//! │                                                              │
//! ├──────────────────────────────────────────────────────────────┤
//! │                  Release Build                               │
//! │                                                              │
//! │  Binary cache ──load directly──▶ CPU data                   │
//! │  (optc/*.otxtr…)                                             │
//! │  Source file is never touched                                │
//! │                                                              │
//! └──────────────────────────────────────────────────────────────┘
//! ```
//!
//! In **debug builds**, `from_disk(path)` always re-parses the source file
//! and overwrites the binary cache. This means editing a `.png`, `.obj`, or
//! `.glsl` file takes effect immediately on next run — no manual rebuild step.
//!
//! In **release builds**, `from_disk(path)` loads directly from the binary
//! cache for faster startup. The source file is never read. If the cache does
//! not exist in a release build, loading fails.
//!
//! # Cache Path Convention
//!
//! All asset types use [`optic_file::cached_path`] to compute cache locations.
//! Cache files are placed in an `optc/` subdirectory adjacent to the source:
//!
//! ```text
//! assets/tex/foo.png     → assets/tex/optc/foo.otxtr
//! models/cube.obj        → models/optc/cube.omesh
//! shaders/main.glsl      → shaders/optc/main.oshdr
//! fonts/arial.ttf        → fonts/optc/arial.ofont
//! sound/bgm.ogg          → sound/optc/bgm.omusic
//! ```
//!
//! # Binary Cache Format
//!
//! Every binary cache file starts with an identical header:
//!
//! | Offset | Size | Field | Description |
//! |--------|------|-------|-------------|
//! | 0 | 8 | Magic | `b"/0PTIC_x"` — never changes |
//! | 8 | 2 | Version | `OPTIC_CACHE_VERSION` (currently `1`, u16 LE) |
//! | 10+ | varies | Payload | Asset-specific data |
//!
//! The version field allows the cache format to evolve without breaking old
//! caches — if the version doesn't match, the cache is rejected and (in debug)
//! regenerated from source.
//!
//! # Asset Types
//!
//! | Type | Source formats | Cache ext | Cache contents |
//! |------|---------------|-----------|----------------|
//! | [`TextureFile`] | `.png`, `.jpg`, … | `.otxtr` | Raw pixels + dimensions + format |
//! | [`Mesh3DFile`] | `.obj`, `.stl`, procedural | `.omesh` | Vertex arrays + index buffer |
//! | [`Mesh2DFile`] | procedural | `.omesh` | 2D vertices + indices + layer |
//! | [`ShaderFile`] | `.glsl` | `.oshdr` | Vertex + fragment source strings |
//! | [`FontFamilyFile`] | `.ttf`, `.otf` | `.ofont` | MSDF atlas + glyph metrics + TTF bytes |
//! | [`SoundFile`] | `.ogg`, … | `.omusic` | Interleaved PCM samples |
//!
//! # Font Baking Pipeline (MSDF)
//!
//! Fonts are the most complex asset type. When a TTF/OTF font is loaded for
//! the first time, the engine performs a multi-step baking process:
//!
//! 1. **Parse** — `ttf_parser::Face::parse()` reads the TrueType font data.
//!
//! 2. **Extract edges** — For each codepoint in the range (default: ASCII
//!    32..126), [`extract_glyph_edges`] converts the glyph outline into
//!    [`Contour`]s made of [`EdgeSegment`]s (line, quadratic bezier, cubic
//!    bezier).
//!
//! 3. **Bake MSDF** — [`bake_msdf`] rasterises each glyph into a
//!    **multi-channel signed distance field** — a 3-channel (RGB) texture
//!    where each channel stores signed distance classified by the nearest
//!    edge's normal direction. This preserves sharp corners at any scale.
//!
//! 4. **Pack atlas** — Individual glyph MSDF textures are packed into a
//!    512×512 atlas grid. [`GlyphMetrics`] record the UV rect, size, bearing,
//!    and advance for each glyph.
//!
//! 5. **Cache** — The assembled [`FontFamilyFile`] is saved as a `.ofont`
//!    binary containing the atlas textures, glyph metric tables, font metrics,
//!    and the raw TTF source bytes (needed by rustybuzz for text shaping).
//!
//! For **bitmap fonts**, [`bake_sdf_from_bitmap`] converts a binary sprite
//! sheet into a single-channel SDF atlas using distance transform.
//!
//! # GPU Upload
//!
//! CPU-side asset types (`TextureFile`, `Mesh3DFile`, `ShaderFile`, etc.) are
//! uploaded to the GPU via the [`GPU`](crate::GPU) renderer:
//!
//! ```ignore
//! // Load from disk (with caching)
//! let tex_file = TextureFile::from_disk("assets/grass.png")?;
//!
//! // Upload to GPU → returns a handle
//! let tex: Texture2D = gpu.upload_texture(&tex_file);
//!
//! // Meshes get a fallback shader automatically
//! let cube = Mesh3DFile::cube(2.0);
//! let mesh: Mesh3D = gpu.upload_mesh3d(&cube);
//! ```
//!
//! The `GPU` struct also loads **fallback assets** on construction — built-in
//! shaders, a checkerboard texture, and an 8×8 bitmap font — so you can
//! render something immediately without providing any custom assets.
//!
//! # Fallback Assets
//!
//! If no user asset is available, the engine provides built-in defaults:
//!
//! | Asset | Source | Behaviour |
//! |-------|--------|-----------|
//! | 2D shader | `optic/assets/shdr/fallback2d.glsl` | Used by `upload_mesh2d` |
//! | 3D shader | `optic/assets/shdr/fallback3d.glsl` | Used by `upload_mesh3d` |
//! | Text shaders | `assets/shdr/fallback_text{2d,3d}.glsl` | Used by `Text2D`/`Text3D` |
//! | Texture | `optic/assets/txtr/fallback.png` | Checkerboard pattern |
//! | Font | Hardcoded 8×8 bitmap | ASCII 32–126, always available |
//!
//! # Sub-modules
//!
//! - [`attr`] — vertex and instance attribute descriptors for meshes

pub mod attr;
pub mod font;
mod img;
mod msh;
mod msdf;
mod shdr;

pub use font::*;
pub use img::*;
pub use msh::*;
pub use msdf::*;
pub use shdr::*;
