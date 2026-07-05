use image::{ColorType, GenericImageView};
use optic_core::consts::{OPTIC_CACHE_VERSION, OPTIC_MAGIC};
use optic_core::{ImgFilter, ImgFormat, ImgWrap, OpticError, OpticErrorKind, OpticResult, Size2D};

use crate::handles::texture::{create_texture, Texture2D};

/// A texture loaded from disk (or cache) with metadata.
///
/// # Loading
///
/// ```ignore
/// use optic_render::asset::TextureFile;
///
/// let tex = TextureFile::from_disk("textures/wood.png")?;
/// let gpu_tex = tex.upload(); // uploads to GPU
/// ```
///
/// # Caching
///
/// In debug builds, `from_disk` loads the source image and writes a binary
/// cache (`.otxtr`). In release builds, it reads the cache directly for
/// faster startup.
pub struct TextureFile {
    pub bytes: Vec<u8>,
    pub size: Size2D,
    pub fmt: ImgFormat,
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}

impl TextureFile {
    /// Returns the total pixel count (width × height).
    pub fn pixel_count(&self) -> usize {
        self.size.w as usize * self.size.h as usize
    }
    /// Overrides the wrap mode (used before [`upload`](TextureFile::upload)).
    pub fn set_wrap(&mut self, wrap: ImgWrap) { self.wrap = wrap; }
    /// Overrides the filter mode (used before [`upload`](TextureFile::upload)).
    pub fn set_filter(&mut self, filter: ImgFilter) { self.filter = filter; }

    /// Uploads this texture to the GPU and returns a [`Texture2D`] handle.
    pub fn upload(&self) -> Texture2D {
        let id = create_texture(&self.bytes, self.size, &self.fmt, &self.filter, &self.wrap);
        Texture2D::new(id, self.size, self.fmt, self.filter, self.wrap)
    }

    /// Loads the fallback texture from `optic/assets/txtr/fallback.png`.
    pub fn fallback() -> OpticResult<Self> {
        Self::from_disk("optic/assets/txtr/fallback.png")
    }
}

// --- from_disk: debug loads source + overwrites cache; release loads cache only ---
#[cfg(debug_assertions)]
impl TextureFile {
    /// Loads a texture from disk, caching it for release builds.
    pub fn from_disk(path: &str) -> OpticResult<Self> {
        let img = image::open(path)
            .map_err(|e| OpticError::new(OpticErrorKind::File, &format!("failed to load image {path}: {e}")))?;

        let (w, h) = img.dimensions();
        let color = img.color();
        let bytes = img.as_bytes().to_vec();

        let fmt = match color {
            ColorType::L8 => ImgFormat::R(8),
            ColorType::La8 => ImgFormat::RG(8),
            ColorType::Rgb8 => ImgFormat::RGB(8),
            ColorType::Rgba8 => ImgFormat::RGBA(8),
            ColorType::L16 => ImgFormat::R(16),
            ColorType::La16 => ImgFormat::RG(16),
            ColorType::Rgb16 => ImgFormat::RGB(16),
            ColorType::Rgba16 => ImgFormat::RGBA(16),
            ColorType::Rgb32F => ImgFormat::RGB(32),
            ColorType::Rgba32F => ImgFormat::RGBA(32),
            _ => ImgFormat::RGBA(8),
        };

        let tex = Self {
            bytes,
            size: Size2D::new(w, h),
            fmt,
            filter: ImgFilter::Closest,
            wrap: ImgWrap::Clip,
        };

        let cache = optic_file::cached_path(path, "otxtr");
        tex.save_cached(&cache)?;
        Ok(tex)
    }
}

#[cfg(not(debug_assertions))]
impl TextureFile {
    /// Loads a texture from the binary cache (release only).
    pub fn from_disk(path: &str) -> OpticResult<Self> {
        let cache = optic_file::cached_path(path, "otxtr");
        Self::from_cached(&cache)
    }
}

// --- binary cache read/write (internal) ---
impl TextureFile {
    /// Saves this texture to a binary cache file.
    pub fn save_cached(&self, path: &str) -> OpticResult<()> {
        let mut data = Vec::with_capacity(22 + self.bytes.len());
        data.extend_from_slice(&OPTIC_MAGIC);
        data.extend_from_slice(&OPTIC_CACHE_VERSION.to_le_bytes());
        data.push(self.fmt.channels());
        data.push(self.fmt.bit_depth());
        data.extend_from_slice(&(self.size.w as u32).to_le_bytes());
        data.extend_from_slice(&(self.size.h as u32).to_le_bytes());
        data.push(match self.filter {
            ImgFilter::Closest => 0u8,
            ImgFilter::Linear => 1u8,
        });
        data.push(match self.wrap {
            ImgWrap::Repeat => 0u8,
            ImgWrap::Extend => 1u8,
            ImgWrap::Clip => 2u8,
        });
        data.extend_from_slice(&self.bytes);
        optic_file::write_bytes(path, &data)
    }

    /// Loads a texture from a binary cache file.
    #[cfg_attr(debug_assertions, allow(dead_code))]
    pub(crate) fn from_cached(path: &str) -> OpticResult<Self> {
        let data = optic_file::read_bytes(path)?;
        if data.len() < 22 {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("cached texture too short: {path}")));
        }
        if data[0..8] != OPTIC_MAGIC {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("not a valid Optic cache file (bad magic): {path}")));
        }
        let version = u16::from_le_bytes([data[8], data[9]]);
        if version != OPTIC_CACHE_VERSION {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!(
                "cache file version {version} is not supported (expected {OPTIC_CACHE_VERSION}): {path}"
            )));
        }
        let channels = data[10];
        let bit_depth = data[11];
        let w = u32::from_le_bytes([data[12], data[13], data[14], data[15]]);
        let h = u32::from_le_bytes([data[16], data[17], data[18], data[19]]);
        let filter = match data[20] {
            0 => ImgFilter::Closest,
            1 => ImgFilter::Linear,
            _ => ImgFilter::Closest,
        };
        let wrap = match data[21] {
            0 => ImgWrap::Repeat,
            1 => ImgWrap::Extend,
            _ => ImgWrap::Clip,
        };
        let bytes = data[22..].to_vec();
        let expected = w as usize * h as usize * channels as usize * (bit_depth as usize / 8);
        if bytes.len() != expected {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!(
                "cached texture size mismatch: expected {expected} bytes, got {} for {path}", bytes.len()
            )));
        }
        Ok(Self {
            bytes,
            size: Size2D::new(w, h),
            fmt: ImgFormat::new(channels, bit_depth),
            filter,
            wrap,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_count() {
        let img = TextureFile {
            bytes: vec![0u8; 1920 * 1080 * 4],
            size: Size2D::new(1920, 1080),
            fmt: ImgFormat::RGBA(8),
            filter: ImgFilter::Closest,
            wrap: ImgWrap::Clip,
        };
        assert_eq!(img.pixel_count(), 1920 * 1080);
    }

    #[test]
    fn pixel_count_zero() {
        let img = TextureFile {
            bytes: vec![],
            size: Size2D::zero(),
            fmt: ImgFormat::RGBA(8),
            filter: ImgFilter::Closest,
            wrap: ImgWrap::Clip,
        };
        assert_eq!(img.pixel_count(), 0);
    }

    #[test]
    fn image_cached_roundtrip() {
        let img = TextureFile {
            bytes: vec![128u8; 16 * 16 * 4],
            size: Size2D::new(16, 16),
            fmt: ImgFormat::RGBA(8),
            filter: ImgFilter::Linear,
            wrap: ImgWrap::Repeat,
        };
        let path = "/tmp/optic_test_img_cache.otxtr";
        img.save_cached(path).unwrap();
        let loaded = TextureFile::from_cached(path).unwrap();
        assert_eq!(loaded.bytes, img.bytes);
        assert_eq!(loaded.size, img.size);
        assert_eq!(loaded.fmt, img.fmt);
        assert_eq!(loaded.filter, img.filter);
        assert_eq!(loaded.wrap, img.wrap);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn image_from_cached_bad_magic() {
        let path = "/tmp/optic_test_img_badmagic.bin";
        optic_file::write_bytes(path, &[0u8; 30]).unwrap();
        let result = TextureFile::from_cached(path);
        assert!(result.is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn image_from_cached_too_short() {
        let path = "/tmp/optic_test_img_short.bin";
        optic_file::write_bytes(path, b"tooshrt").unwrap();
        let result = TextureFile::from_cached(path);
        assert!(result.is_err());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn set_wrap_filter() {
        let mut img = TextureFile {
            bytes: vec![],
            size: Size2D::new(1, 1),
            fmt: ImgFormat::RGBA(8),
            filter: ImgFilter::Closest,
            wrap: ImgWrap::Clip,
        };
        assert_eq!(img.filter, ImgFilter::Closest);
        assert_eq!(img.wrap, ImgWrap::Clip);

        img.set_filter(ImgFilter::Linear);
        img.set_wrap(ImgWrap::Repeat);
        assert_eq!(img.filter, ImgFilter::Linear);
        assert_eq!(img.wrap, ImgWrap::Repeat);
    }
}
