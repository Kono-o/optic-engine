use optic_core::{Cull, PolyMode, RGBA, Size2D};

pub struct GL;

impl GL {
    pub fn clear() {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }
    }

    pub fn set_clear(color: RGBA) {
        unsafe { gl::ClearColor(color.0, color.1, color.2, color.3); }
    }

    pub fn resize(size: Size2D) {
        unsafe { gl::Viewport(0, 0, size.w as i32, size.h as i32); }
    }

    pub fn poly_mode(mode: PolyMode) {
        unsafe {
            match mode {
                PolyMode::WireFrame => gl::PolygonMode(gl::FRONT_AND_BACK, gl::LINE),
                PolyMode::Filled => gl::PolygonMode(gl::FRONT_AND_BACK, gl::FILL),
                PolyMode::Points => {
                    gl::PointSize(10.0);
                    gl::PolygonMode(gl::FRONT_AND_BACK, gl::POINT);
                }
            }
        }
    }

    pub fn enable_msaa(enable: bool) {
        unsafe {
            match enable {
                true => gl::Enable(gl::MULTISAMPLE),
                false => gl::Disable(gl::MULTISAMPLE),
            }
        }
    }

    pub fn enable_depth(enable: bool) {
        unsafe {
            match enable {
                true => gl::Enable(gl::DEPTH_TEST),
                false => gl::Disable(gl::DEPTH_TEST),
            }
        }
    }

    pub fn enable_alpha(enable: bool) {
        unsafe {
            match enable {
                true => {
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                }
                false => gl::Disable(gl::BLEND),
            }
        }
    }

    pub fn enable_cull(enable: bool) {
        unsafe {
            match enable {
                true => {
                    gl::Enable(gl::CULL_FACE);
                    gl::CullFace(gl::BACK);
                }
                false => gl::Disable(gl::CULL_FACE),
            }
        }
    }

    pub fn set_cull_face(face: Cull) {
        unsafe {
            match face {
                Cull::Clock => gl::FrontFace(gl::CW),
                Cull::AntiClock => gl::FrontFace(gl::CCW),
            }
        }
    }

    pub fn set_point_size(size: f32) {
        unsafe { gl::PointSize(size); }
    }

    pub fn set_wire_width(width: f32) {
        unsafe { gl::LineWidth(width); }
    }

    pub fn bind_shader(id: u32) {
        unsafe { gl::UseProgram(id); }
    }

    pub fn unbind_shader() {
        unsafe { gl::UseProgram(0); }
    }

    pub fn bind_texture_at(tex_id: u32, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);
        }
    }

    pub fn unbind_texture() {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0); }
    }

    pub fn bind_vao(id: u32) {
        unsafe { gl::BindVertexArray(id); }
    }

    pub fn unbind_vao() {
        unsafe { gl::BindVertexArray(0); }
    }

    pub fn bind_buffer(id: u32) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, id); }
    }

    pub fn unbind_buffer() {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0); }
    }

    pub fn bind_ebo(id: u32) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id); }
    }

    pub fn unbind_ebo() {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); }
    }

    pub fn bind_ssbo(id: u32) {
        unsafe { gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id); }
    }

    pub fn unbind_ssbo() {
        unsafe { gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0); }
    }
}
