/// Polygon rasterization mode.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PolygonMode {
    Points,
    WireFrame,
    Filled,
}

/// Face culling direction.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CullFace {
    Clock,
    AntiClock,
}

/// Primitive topology for draw calls.
#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum DrawMode {
    Points,
    Lines,
    #[default]
    Triangles,
    Strip,
}

/// Texture image format, encoding channel count and bit depth.
///
/// Each variant carries the per-channel bit depth (e.g. `RGBA(8)` = 4×8-bit).
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImgFormat {
    R(u8),
    RG(u8),
    RGB(u8),
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgFilter {
    Closest,
    Linear,
}

/// Texture wrap (addressing) mode.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgWrap {
    Repeat,
    Extend,
    Clip,
}

/// Vertex attribute data type.
#[derive(Clone, Debug, PartialEq)]
pub enum ATTRType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
    F64,
}
