use optic_core::{ImgFilter, ImgFormat, ImgWrap, OpticError, OpticErrorKind, OpticResult, Rect, Size2D};

use crate::handles::texture::{Texture2D, delete_texture};

#[derive(Clone, Debug)]
pub struct CanvasDesc {
    pub size: Size2D,
    pub color_formats: Vec<ImgFormat>,
    pub depth: bool,
    pub depth_as_texture: bool,
    pub depth_compare: bool,
    pub stencil: bool,
    pub samples: u32,
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}

impl Default for CanvasDesc {
    fn default() -> Self {
        Self {
            size: Size2D::from(512, 512),
            color_formats: vec![ImgFormat::RGBA(8)],
            depth: true,
            depth_as_texture: true,
            depth_compare: false,
            stencil: false,
            samples: 0,
            filter: ImgFilter::Linear,
            wrap: ImgWrap::Extend,
        }
    }
}

pub struct Canvas {
    pub(crate) fbo_id: u32,
    pub(crate) resolve_fbo_id: u32,
    pub(crate) msaa_rbos: Vec<u32>,
    pub(crate) depth_stencil_rbo: u32,
    pub(crate) color_texs: Vec<Texture2D>,
    pub(crate) depth_tex: Option<Texture2D>,
    pub(crate) size: Size2D,
    #[allow(dead_code)]
    pub(crate) samples: u32,
    pub(crate) has_stencil: bool,
    pub(crate) has_depth: bool,
    #[allow(dead_code)]
    pub(crate) depth_as_texture: bool,
    pub(crate) desc: CanvasDesc,
}

impl Canvas {
    pub fn new(desc: &CanvasDesc) -> OpticResult<Self> {
        if desc.color_formats.is_empty() && !desc.depth {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "Canvas: at least one color format or depth must be specified",
            ));
        }
        if desc.stencil && !desc.depth {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "Canvas: stencil requires depth to be enabled",
            ));
        }
        if desc.depth_compare && !desc.depth_as_texture {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "Canvas: depth_compare requires depth_as_texture",
            ));
        }

        let size = desc.size;
        let has_msaa = desc.samples > 1;
        let msaa_s = if has_msaa { desc.samples } else { 0 };

        let fbo_id = unsafe {
            let mut id = 0;
            gl::GenFramebuffers(1, &mut id);
            gl::BindFramebuffer(gl::FRAMEBUFFER, id);
            id
        };

        let mut color_texs: Vec<Texture2D> = Vec::new();
        let mut msaa_rbos: Vec<u32> = Vec::new();
        let mut depth_stencil_rbo = 0u32;
        let mut depth_tex: Option<Texture2D> = None;
        let mut resolve_fbo_id = 0u32;

        for (i, fmt) in desc.color_formats.iter().enumerate() {
            let attachment = gl::COLOR_ATTACHMENT0 + i as u32;
            if has_msaa {
                let rbo = create_rbo_storage_msaa(size, msaa_s, fmt, desc.stencil);
                unsafe {
                    gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, attachment, gl::RENDERBUFFER, rbo);
                }
                msaa_rbos.push(rbo);
            } else {
                let tex_id = create_empty_tex(size, fmt, desc.filter, desc.wrap);
                unsafe {
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, attachment, gl::TEXTURE_2D, tex_id, 0);
                }
                color_texs.push(Texture2D::new(tex_id, size, *fmt, desc.filter, desc.wrap));
            }
        }

        if !desc.color_formats.is_empty() {
            let attachments: Vec<u32> = (0..desc.color_formats.len() as u32)
                .map(|i| gl::COLOR_ATTACHMENT0 + i)
                .collect();
            unsafe {
                gl::DrawBuffers(desc.color_formats.len() as i32, attachments.as_ptr());
            }
        } else {
            unsafe { gl::DrawBuffer(gl::NONE); }
        }

        if desc.depth {
            if has_msaa {
                let (internal, att) = depth_rbo_params(desc.stencil);
                let rbo = unsafe {
                    let mut id = 0;
                    gl::GenRenderbuffers(1, &mut id);
                    gl::BindRenderbuffer(gl::RENDERBUFFER, id);
                    gl::RenderbufferStorageMultisample(
                        gl::RENDERBUFFER, msaa_s as i32, internal as u32, size.w as i32, size.h as i32,
                    );
                    gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, att, gl::RENDERBUFFER, id);
                    gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
                    id
                };
                depth_stencil_rbo = rbo;
            } else if desc.depth_as_texture {
                let tex_id = create_depth_tex(size, desc.stencil, desc.depth_compare);
                let att = if desc.stencil {
                    gl::DEPTH_STENCIL_ATTACHMENT
                } else {
                    gl::DEPTH_ATTACHMENT
                };
                unsafe {
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, att, gl::TEXTURE_2D, tex_id, 0);
                }
                depth_tex = Some(Texture2D::new(
                    tex_id, size, ImgFormat::R(32), ImgFilter::Closest, ImgWrap::Extend,
                ));
            } else {
                let (internal, att) = depth_rbo_params(desc.stencil);
                let rbo = unsafe {
                    let mut id = 0;
                    gl::GenRenderbuffers(1, &mut id);
                    gl::BindRenderbuffer(gl::RENDERBUFFER, id);
                    gl::RenderbufferStorage(gl::RENDERBUFFER, internal as u32, size.w as i32, size.h as i32);
                    gl::FramebufferRenderbuffer(gl::FRAMEBUFFER, att, gl::RENDERBUFFER, id);
                    gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
                    id
                };
                depth_stencil_rbo = rbo;
            }
        }

        if has_msaa {
            resolve_fbo_id = unsafe {
                let mut id = 0;
                gl::GenFramebuffers(1, &mut id);
                gl::BindFramebuffer(gl::FRAMEBUFFER, id);
                id
            };

            for (i, fmt) in desc.color_formats.iter().enumerate() {
                let tex_id = create_empty_tex(size, fmt, desc.filter, desc.wrap);
                unsafe {
                    gl::FramebufferTexture2D(
                        gl::FRAMEBUFFER,
                        gl::COLOR_ATTACHMENT0 + i as u32,
                        gl::TEXTURE_2D, tex_id, 0,
                    );
                }
                color_texs.push(Texture2D::new(tex_id, size, *fmt, desc.filter, desc.wrap));
            }

            if !desc.color_formats.is_empty() {
                let attachments: Vec<u32> = (0..desc.color_formats.len() as u32)
                    .map(|i| gl::COLOR_ATTACHMENT0 + i)
                    .collect();
                unsafe {
                    gl::DrawBuffers(desc.color_formats.len() as i32, attachments.as_ptr());
                }
            } else {
                unsafe { gl::DrawBuffer(gl::NONE); }
            }

            if desc.depth && desc.depth_as_texture {
                let tex_id = create_depth_tex(size, desc.stencil, desc.depth_compare);
                let att = if desc.stencil {
                    gl::DEPTH_STENCIL_ATTACHMENT
                } else {
                    gl::DEPTH_ATTACHMENT
                };
                unsafe {
                    gl::FramebufferTexture2D(gl::FRAMEBUFFER, att, gl::TEXTURE_2D, tex_id, 0);
                }
                depth_tex = Some(Texture2D::new(
                    tex_id, size, ImgFormat::R(32), ImgFilter::Closest, ImgWrap::Extend,
                ));
            }

            unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, fbo_id); }
        }

        let complete = unsafe { gl::CheckFramebufferStatus(gl::FRAMEBUFFER) };
        unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }

        if complete != gl::FRAMEBUFFER_COMPLETE {
            unsafe {
                gl::DeleteFramebuffers(1, &fbo_id);
                if resolve_fbo_id != 0 {
                    gl::DeleteFramebuffers(1, &resolve_fbo_id);
                }
                for &rbo in &msaa_rbos {
                    gl::DeleteRenderbuffers(1, &rbo);
                }
                if depth_stencil_rbo != 0 {
                    gl::DeleteRenderbuffers(1, &depth_stencil_rbo);
                }
            }
            for tex in &color_texs {
                delete_texture(tex.id);
            }
            if let Some(ref tex) = depth_tex {
                delete_texture(tex.id);
            }
            return Err(OpticError::new(
                OpticErrorKind::Framebuffer,
                &format!("framebuffer incomplete: status={complete:#x}"),
            ));
        }

        Ok(Self {
            fbo_id,
            resolve_fbo_id,
            msaa_rbos,
            depth_stencil_rbo,
            color_texs,
            depth_tex,
            size,
            samples: desc.samples,
            has_stencil: desc.stencil,
            has_depth: desc.depth,
            depth_as_texture: desc.depth_as_texture,
            desc: desc.clone(),
        })
    }

    pub fn size(&self) -> Size2D {
        self.size
    }

    pub fn color_tex(&self, index: usize) -> OpticResult<&Texture2D> {
        self.color_texs.get(index).ok_or_else(|| {
            OpticError::new(
                OpticErrorKind::Custom,
                &format!("Canvas color attachment index {index} out of range ({} attachments)", self.color_texs.len()),
            )
        })
    }

    pub fn depth_tex(&self) -> Option<&Texture2D> {
        self.depth_tex.as_ref()
    }

    pub fn set_size(&mut self, new_size: Size2D) -> OpticResult<()> {
        let mut new_desc = self.desc.clone();
        new_desc.size = new_size;
        let new_canvas = Canvas::new(&new_desc)?;
        *self = new_canvas;
        Ok(())
    }

    pub fn resolve(&self) {
        if self.resolve_fbo_id == 0 {
            return;
        }
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, self.fbo_id);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, self.resolve_fbo_id);
            let mut mask = gl::COLOR_BUFFER_BIT;
            if self.has_depth {
                mask |= gl::DEPTH_BUFFER_BIT;
            }
            if self.has_stencil {
                mask |= gl::STENCIL_BUFFER_BIT;
            }
            gl::BlitFramebuffer(
                0, 0, self.size.w as i32, self.size.h as i32,
                0, 0, self.size.w as i32, self.size.h as i32,
                mask, gl::NEAREST,
            );
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
        }
    }

    pub fn blit_to_screen(&self, window_size: Size2D) {
        self.resolve_if_needed();
        let src = if self.resolve_fbo_id != 0 {
            self.resolve_fbo_id
        } else {
            self.fbo_id
        };
        unsafe {
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, src);
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, 0);
            gl::BlitFramebuffer(
                0, 0, self.size.w as i32, self.size.h as i32,
                0, 0, window_size.w as i32, window_size.h as i32,
                gl::COLOR_BUFFER_BIT, gl::LINEAR,
            );
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, 0);
        }
    }

    pub fn set_renderable_area(&self, area: Rect) -> OpticResult<()> {
        if area.x < 0
            || area.y < 0
            || area.w <= 0
            || area.h <= 0
            || area.x + area.w > self.size.w as i32
            || area.y + area.h > self.size.h as i32
        {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!(
                    "Canvas::set_renderable_area rect ({},{},{},{}) exceeds canvas size ({},{})",
                    area.x, area.y, area.w, area.h, self.size.w, self.size.h,
                ),
            ));
        }
        unsafe {
            gl::Viewport(area.x, area.y, area.w, area.h);
        }
        Ok(())
    }

    pub fn read_pixels(&self, index: usize) -> OpticResult<Vec<u8>> {
        self.resolve_if_needed();
        let tex = self.color_tex(index)?;
        let src = if self.resolve_fbo_id != 0 {
            self.resolve_fbo_id
        } else {
            self.fbo_id
        };
        let mut prev_fbo = 0i32;
        unsafe {
            gl::GetIntegerv(gl::FRAMEBUFFER_BINDING, &mut prev_fbo);
            gl::BindFramebuffer(gl::FRAMEBUFFER, src);
        }
        let (format, pix_type, pixel_size) = fmt_gl_params(&tex.fmt);
        let total = (self.size.w as usize) * (self.size.h as usize) * pixel_size;
        let mut pixels = vec![0u8; total];
        unsafe {
            gl::ReadPixels(
                0, 0, self.size.w as i32, self.size.h as i32,
                format, pix_type, pixels.as_mut_ptr() as *mut _,
            );
            gl::BindFramebuffer(gl::FRAMEBUFFER, prev_fbo as u32);
        }
        Ok(pixels)
    }

    pub fn save_to_file(&self, index: usize, path: &str) -> OpticResult<()> {
        let data = self.read_pixels(index)?;
        let tex = self.color_tex(index)?;
        let channels = tex.fmt.channels() as u8;
        let bit_depth = tex.fmt.bit_depth();
        let ct = match (channels, bit_depth) {
            (1, 8) => image::ColorType::L8,
            (2, 8) => image::ColorType::La8,
            (3, 8) => image::ColorType::Rgb8,
            (4, 8) => image::ColorType::Rgba8,
            (1, 16) => image::ColorType::L16,
            (2, 16) => image::ColorType::La16,
            (3, 16) => image::ColorType::Rgb16,
            (4, 16) => image::ColorType::Rgba16,
            (3, 32) => image::ColorType::Rgb32F,
            (4, 32) => image::ColorType::Rgba32F,
            _ => {
                return Err(OpticError::new(
                    OpticErrorKind::Custom,
                    &format!("unsupported format for save_to_file: {}x{}bpp", channels, bit_depth),
                ))
            }
        };
        image::save_buffer(path, &data, self.size.w, self.size.h, ct).map_err(|e| {
            OpticError::new(OpticErrorKind::File, &format!("failed to save image: {e}"))
        })
    }

    pub fn delete(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.fbo_id);
            if self.resolve_fbo_id != 0 {
                gl::DeleteFramebuffers(1, &self.resolve_fbo_id);
            }
            for &rbo in &self.msaa_rbos {
                gl::DeleteRenderbuffers(1, &rbo);
            }
            if self.depth_stencil_rbo != 0 {
                gl::DeleteRenderbuffers(1, &self.depth_stencil_rbo);
            }
        }
        for tex in std::mem::take(&mut self.color_texs) {
            tex.delete();
        }
        if let Some(tex) = self.depth_tex.take() {
            tex.delete();
        }
    }

    fn resolve_if_needed(&self) {
        if self.resolve_fbo_id != 0 {
            self.resolve();
        }
    }
}

fn depth_rbo_params(stencil: bool) -> (i32, u32) {
    if stencil {
        (gl::DEPTH24_STENCIL8 as i32, gl::DEPTH_STENCIL_ATTACHMENT)
    } else {
        (gl::DEPTH_COMPONENT24 as i32, gl::DEPTH_ATTACHMENT)
    }
}

fn create_empty_tex(size: Size2D, fmt: &ImgFormat, filter: ImgFilter, wrap: ImgWrap) -> u32 {
    unsafe {
        let mut id = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);

        let (min_fil, mag_fil) = match filter {
            ImgFilter::Closest => (gl::NEAREST as i32, gl::NEAREST as i32),
            ImgFilter::Linear => (gl::LINEAR as i32, gl::LINEAR as i32),
        };
        let wrap_gl = match wrap {
            ImgWrap::Repeat => gl::REPEAT,
            ImgWrap::Extend => gl::CLAMP_TO_EDGE,
            ImgWrap::Clip => gl::CLAMP_TO_BORDER,
        };
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_fil);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_fil);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, wrap_gl as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, wrap_gl as i32);

        let (base, sized, pix_type) = fmt_to_gl(fmt);
        gl::TexImage2D(
            gl::TEXTURE_2D, 0, sized as i32,
            size.w as i32, size.h as i32, 0,
            base, pix_type, std::ptr::null(),
        );
        gl::BindTexture(gl::TEXTURE_2D, 0);
        id
    }
}

fn create_rbo_storage_msaa(size: Size2D, samples: u32, fmt: &ImgFormat, _stencil: bool) -> u32 {
    unsafe {
        let mut id = 0;
        gl::GenRenderbuffers(1, &mut id);
        gl::BindRenderbuffer(gl::RENDERBUFFER, id);
        let (_base, sized, _pix_type) = fmt_to_gl(fmt);
        gl::RenderbufferStorageMultisample(
            gl::RENDERBUFFER, samples as i32, sized as u32,
            size.w as i32, size.h as i32,
        );
        gl::BindRenderbuffer(gl::RENDERBUFFER, 0);
        id
    }
}

fn create_depth_tex(size: Size2D, stencil: bool, compare: bool) -> u32 {
    unsafe {
        let mut id = 0;
        gl::GenTextures(1, &mut id);
        gl::BindTexture(gl::TEXTURE_2D, id);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

        let (internal, format, pix_type) = if stencil {
            (gl::DEPTH24_STENCIL8 as i32, gl::DEPTH_STENCIL, gl::UNSIGNED_INT_24_8)
        } else {
            (gl::DEPTH_COMPONENT24 as i32, gl::DEPTH_COMPONENT, gl::FLOAT)
        };

        gl::TexImage2D(
            gl::TEXTURE_2D, 0, internal,
            size.w as i32, size.h as i32, 0,
            format, pix_type, std::ptr::null(),
        );

        if compare {
            gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_COMPARE_MODE,
                gl::COMPARE_REF_TO_TEXTURE as i32,
            );
            gl::TexParameteri(
                gl::TEXTURE_2D, gl::TEXTURE_COMPARE_FUNC,
                gl::LEQUAL as i32,
            );
        }

        gl::BindTexture(gl::TEXTURE_2D, 0);
        id
    }
}

fn fmt_to_gl(fmt: &ImgFormat) -> (u32, u32, u32) {
    match fmt {
        ImgFormat::R(bd) => match bd {
            32 => (gl::RED, gl::R32F, gl::FLOAT),
            16 => (gl::RED, gl::R16, gl::UNSIGNED_SHORT),
            _ => (gl::RED, gl::R8, gl::UNSIGNED_BYTE),
        },
        ImgFormat::RG(bd) => match bd {
            32 => (gl::RG, gl::RG32F, gl::FLOAT),
            16 => (gl::RG, gl::RG16, gl::UNSIGNED_SHORT),
            _ => (gl::RG, gl::RG8, gl::UNSIGNED_BYTE),
        },
        ImgFormat::RGB(bd) => match bd {
            32 => (gl::RGB, gl::RGB32F, gl::FLOAT),
            16 => (gl::RGB, gl::RGB16, gl::UNSIGNED_SHORT),
            _ => (gl::RGB, gl::RGB8, gl::UNSIGNED_BYTE),
        },
        ImgFormat::RGBA(bd) => match bd {
            32 => (gl::RGBA, gl::RGBA32F, gl::FLOAT),
            16 => (gl::RGBA, gl::RGBA16, gl::UNSIGNED_SHORT),
            _ => (gl::RGBA, gl::RGBA8, gl::UNSIGNED_BYTE),
        },
    }
}

fn fmt_gl_params(fmt: &ImgFormat) -> (u32, u32, usize) {
    let (base, _, pix_type) = fmt_to_gl(fmt);
    let channels = fmt.channels() as usize;
    let bytes_per_channel = (fmt.bit_depth() / 8) as usize;
    (base, pix_type, channels * bytes_per_channel)
}

pub enum RenderTarget<'a> {
    Screen,
    Canvas(&'a Canvas),
}
