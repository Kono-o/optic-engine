use optic_core::{ImgFilter, ImgFormat, ImgWrap, Size2D};

use crate::GL;

/// A handle to an OpenGL 2D texture object.
///
/// Stores the GL texture ID, size, pixel format, filtering, and wrap mode.
/// Created by [`create_texture`] or via [`TextureFile::ship`](crate::asset::TextureFile::ship).
#[derive(Clone, Debug)]
pub struct Texture2D {
    pub id: u32,
    pub size: Size2D,
    pub fmt: ImgFormat,
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}

impl Texture2D {
    /// Creates a new texture handle from a raw GL texture ID.
    pub fn new(
        id: u32,
        size: Size2D,
        fmt: ImgFormat,
        filter: ImgFilter,
        wrap: ImgWrap,
    ) -> Self {
        Self { id, size, fmt, filter, wrap }
    }

    /// Returns the texture dimensions.
    pub fn size(&self) -> Size2D { self.size }
    /// Returns the current wrap mode.
    pub fn wrap(&self) -> ImgWrap { self.wrap }
    /// Overrides the stored wrap mode.
    pub fn set_wrap(&mut self, wrap: ImgWrap) { self.wrap = wrap; }
    /// Returns the current filter mode.
    pub fn filter(&self) -> ImgFilter { self.filter }
    /// Overrides the stored filter mode.
    pub fn set_filter(&mut self, filter: ImgFilter) { self.filter = filter; }

    /// Deletes the underlying OpenGL texture.
    pub fn delete(self) { delete_texture(self.id); }
}

/// Creates a new OpenGL 2D texture from raw pixel data.
///
/// Generates mipmaps automatically. The texture is left bound to texture unit 0
/// after creation.
pub fn create_texture(
    bytes: &[u8],
    size: Size2D,
    fmt: &ImgFormat,
    filter: &ImgFilter,
    wrap: &ImgWrap,
) -> u32 {
    let mut id = 0u32;
    unsafe {
        gl::GenTextures(1, &mut id);
        GL::bind_texture_at(id, 0);

        let wrap_gl = match wrap {
            ImgWrap::Repeat => gl::REPEAT,
            ImgWrap::Extend => gl::CLAMP_TO_EDGE,
            ImgWrap::Clip => gl::CLAMP_TO_BORDER,
        };
        let (min_fil, mag_fil) = match filter {
            ImgFilter::Closest => (gl::NEAREST_MIPMAP_NEAREST as i32, gl::NEAREST as i32),
            ImgFilter::Linear => (gl::LINEAR_MIPMAP_LINEAR as i32, gl::LINEAR as i32),
        };

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_gl as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_gl as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_fil);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_fil);

        let (base, sized, pix_type) = match fmt {
            ImgFormat::R(bd) => match bd {
                32 => (gl::RED, gl::R32F, gl::FLOAT),
                16 => (gl::RED, gl::R16, gl::UNSIGNED_SHORT),
                _  => (gl::RED, gl::R8, gl::UNSIGNED_BYTE),
            },
            ImgFormat::RG(bd) => match bd {
                32 => (gl::RG, gl::RG32F, gl::FLOAT),
                16 => (gl::RG, gl::RG16, gl::UNSIGNED_SHORT),
                _  => (gl::RG, gl::RG8, gl::UNSIGNED_BYTE),
            },
            ImgFormat::RGB(bd) => match bd {
                32 => (gl::RGB, gl::RGB32F, gl::FLOAT),
                16 => (gl::RGB, gl::RGB16, gl::UNSIGNED_SHORT),
                _  => (gl::RGB, gl::RGB8, gl::UNSIGNED_BYTE),
            },
            ImgFormat::RGBA(bd) => match bd {
                32 => (gl::RGBA, gl::RGBA32F, gl::FLOAT),
                16 => (gl::RGBA, gl::RGBA16, gl::UNSIGNED_SHORT),
                _  => (gl::RGBA, gl::RGBA8, gl::UNSIGNED_BYTE),
            },
        };

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            sized as i32,
            size.w as i32,
            size.h as i32,
            0,
            base,
            pix_type,
            bytes.as_ptr() as *const std::ffi::c_void,
        );
        gl::GenerateMipmap(gl::TEXTURE_2D);
        GL::unbind_texture();
    }
    id
}

/// Deletes an OpenGL 2D texture by its ID.
pub fn delete_texture(id: u32) {
    unsafe { gl::DeleteTextures(1, &id); }
}
