//! Rendering enums controlling rasterization, texture, and vertex formats.
//!
//! These enums are used throughout the Optic pipeline to configure:
//!
//! - How polygons are drawn ([`PolygonMode`], [`DrawMode`])
//! - Which faces are culled ([`CullFace`])
//! - Texture formats and filtering ([`ImgFormat`], [`ImgFilter`], [`ImgWrap`])
//! - Vertex attribute types ([`ATTRType`])

/// Polygon rasterization mode.
///
/// Controls how triangles (or other primitives) are rendered on screen.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PolygonMode {
    /// Render each vertex as a point. Useful for particle systems or
    /// debug visualizations.
    Points,
    /// Render polygon edges only, producing a wireframe look. Handy for
    /// level-design overlays and debug views.
    WireFrame,
    /// Render solid filled polygons with interpolated shading. This is
    /// the standard rendering mode.
    Filled,
}

/// Face culling direction.
///
/// Determines which side of a triangle is considered "back-facing" and
/// therefore hidden. Back-face culling improves performance by skipping
/// triangles the viewer cannot see.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CullFace {
    /// Cull faces with clockwise winding order.
    Clock,
    /// Cull faces with counter-clockwise winding order.
    AntiClock,
}

/// Primitive topology for draw calls.
///
/// Defines how vertex data is interpreted when issuing a draw call.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum DrawMode {
    /// Individual, unconnected vertices.
    Points,
    /// Pairs of vertices forming discrete line segments.
    Lines,
    /// Independent triangles — every three vertices define a triangle.
    /// This is the most common mode.
    #[default]
    Triangles,
    /// A triangle strip where each new vertex forms a triangle with the
    /// two preceding vertices, reducing vertex redundancy.
    Strip,
}

/// Texture image format, encoding channel count and bit depth.
///
/// Each variant carries the per-channel bit depth (e.g. `RGBA(8)` = 4×8-bit).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImgFormat {
    /// Single-channel (grayscale) image.
    R(u8),
    /// Two-channel image (e.g. grayscale + alpha or XY normal map).
    RG(u8),
    /// Three-channel image without alpha (e.g. RGB color or RGB normal map).
    RGB(u8),
    /// Four-channel image with alpha (e.g. RGBA diffuse texture).
    RGBA(u8),
}

impl ImgFormat {
    /// Number of color channels (1–4).
    pub fn channels(&self) -> u8 {
        match self {
            ImgFormat::R(_) => 1,
            ImgFormat::RG(_) => 2,
            ImgFormat::RGB(_) => 3,
            ImgFormat::RGBA(_) => 4,
        }
    }
    /// Bits per channel.
    pub fn bit_depth(&self) -> u8 {
        *match self {
            ImgFormat::R(bd) => bd,
            ImgFormat::RG(bd) => bd,
            ImgFormat::RGB(bd) => bd,
            ImgFormat::RGBA(bd) => bd,
        }
    }
    /// Total bytes per pixel (channels × bit_depth / 8).
    pub fn pixel_size(&self) -> u8 {
        self.channels() * self.bit_depth() / 8
    }
    /// Construct from channel count and bit depth.
    ///
    /// Channels ≥ 4 produce `RGBA`. Unknown channels default to `RGBA`.
    pub fn new(channels: u8, bit_depth: u8) -> ImgFormat {
        match channels {
            1 => ImgFormat::R(bit_depth),
            2 => ImgFormat::RG(bit_depth),
            3 => ImgFormat::RGB(bit_depth),
            _ => ImgFormat::RGBA(bit_depth),
        }
    }
}

/// Texture filter (minification/magnification) mode.
///
/// Determines how texels are sampled when a texel does not map 1:1 to a pixel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgFilter {
    /// Nearest-neighbor filtering — picks the closest texel. Produces
    /// sharp, pixelated results. Ideal for pixel art and retro aesthetics.
    Closest,
    /// Bilinear (linear) interpolation — blends the four nearest texels.
    /// Produces smooth results. Standard for photographic textures.
    Linear,
}

/// Texture wrap (addressing) mode.
///
/// Controls what happens when texture coordinates fall outside the
/// normalised [0, 1] range.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgWrap {
    /// Tile the texture by repeating it at integer boundaries. The default
    /// for most tiling textures (floors, walls, …).
    Repeat,
    /// Clamp to the edge texel. Pixels beyond the texture edge are filled
    /// with the nearest border texel. Useful for skyboxes and HUD elements.
    Extend,
    /// Clamp to edge but make out-of-range coordinates transparent (zero
    /// alpha). Useful for decals and overlays where the border should be
    /// invisible.
    Clip,
}

/// Vertex attribute data type.
///
/// Specifies the component type for vertex buffer data (positions, normals,
/// UVs, etc.). The GPU reads attributes according to this type when
/// interpreting raw vertex buffer bytes.
#[derive(Clone, Debug, PartialEq)]
pub enum ATTRType {
    /// Unsigned 8-bit integer (0–255).
    U8,
    /// Signed 8-bit integer (−128–127).
    I8,
    /// Unsigned 16-bit integer (0–65535).
    U16,
    /// Signed 16-bit integer (−32768–32767).
    I16,
    /// Unsigned 32-bit integer.
    U32,
    /// Signed 32-bit integer.
    I32,
    /// 32-bit IEEE 754 floating point.
    F32,
    /// 64-bit IEEE 754 floating point.
    F64,
}
