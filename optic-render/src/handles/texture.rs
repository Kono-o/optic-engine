use optic_core::{ImgFilter, ImgFormat, ImgWrap, Size2D};

use crate::GL;

#[derive(Clone, Debug)]
pub struct Texture2D {
    pub id: u32,
    pub size: Size2D,
    pub fmt: ImgFormat,
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}

impl Texture2D {
    pub fn new(
        id: u32,
        size: Size2D,
        fmt: ImgFormat,
        filter: ImgFilter,
        wrap: ImgWrap,
    ) -> Self {
        Self { id, size, fmt, filter, wrap }
    }

    pub fn size(&self) -> Size2D { self.size }
    pub fn wrap(&self) -> ImgWrap { self.wrap }
    pub fn set_wrap(&mut self, wrap: ImgWrap) { self.wrap = wrap; }
    pub fn filter(&self) -> ImgFilter { self.filter }
    pub fn set_filter(&mut self, filter: ImgFilter) { self.filter = filter; }

    pub fn delete(self) { delete_texture(self.id); }
}

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

pub fn delete_texture(id: u32) {
    unsafe { gl::DeleteTextures(1, &id); }
}
