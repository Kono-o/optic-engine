use optic_core::{Cull, PolyMode, RGBA, Size2D};

/// Low-level, stateless OpenGL 4.6 wrappers.
///
/// Every method is a thin wrapper around a single `gl::*` call — no state tracking,
/// no error handling. Use [`GPU`](crate::GPU) for stateful rendering.
///
/// # Example
///
/// ```ignore
/// optic_render::GL::clear();
/// optic_render::GL::set_clear(RGBA::grey(0.2));
/// ```
pub struct GL;

impl GL {
    /// Clears the colour and depth buffer of the currently bound framebuffer.
    pub fn clear() {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }
    }

    /// Sets the clear colour for subsequent [`clear`](Self::clear) calls.
    pub fn set_clear(color: RGBA) {
        unsafe { gl::ClearColor(color.0, color.1, color.2, color.3); }
    }

    /// Sets the OpenGL viewport to span `(0, 0, size.w, size.h)`.
    pub fn resize(size: Size2D) {
        unsafe { gl::Viewport(0, 0, size.w as i32, size.h as i32); }
    }

    /// Sets polygon rasterisation mode (filled, wireframe, or points).
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

    /// Enables or disables multisample anti-aliasing.
    pub fn enable_msaa(enable: bool) {
        unsafe {
            match enable {
                true => gl::Enable(gl::MULTISAMPLE),
                false => gl::Disable(gl::MULTISAMPLE),
            }
        }
    }

    /// Enables or disables depth testing.
    pub fn enable_depth(enable: bool) {
        unsafe {
            match enable {
                true => gl::Enable(gl::DEPTH_TEST),
                false => gl::Disable(gl::DEPTH_TEST),
            }
        }
    }

    /// Enables or disables alpha blending (`SRC_ALPHA`, `ONE_MINUS_SRC_ALPHA`).
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

    /// Enables or disables back-face culling.
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

    /// Sets the front face winding order (clockwise or counter-clockwise).
    pub fn set_cull_face(face: Cull) {
        unsafe {
            match face {
                Cull::Clock => gl::FrontFace(gl::CW),
                Cull::AntiClock => gl::FrontFace(gl::CCW),
            }
        }
    }

    /// Sets the point size (used with [`PolyMode::Points`]).
    pub fn set_point_size(size: f32) {
        unsafe { gl::PointSize(size); }
    }

    /// Sets the line width (used with [`PolyMode::WireFrame`]).
    pub fn set_wire_width(width: f32) {
        unsafe { gl::LineWidth(width); }
    }

    /// Binds a shader program (`glUseProgram`).
    pub fn bind_shader(id: u32) {
        unsafe { gl::UseProgram(id); }
    }

    /// Unbinds the current shader program (binds 0).
    pub fn unbind_shader() {
        unsafe { gl::UseProgram(0); }
    }

    /// Binds a 2D texture to the given texture unit slot.
    pub fn bind_texture_at(tex_id: u32, slot: u32) {
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0 + slot);
            gl::BindTexture(gl::TEXTURE_2D, tex_id);
        }
    }

    /// Unbinds the currently bound 2D texture (binds 0).
    pub fn unbind_texture() {
        unsafe { gl::BindTexture(gl::TEXTURE_2D, 0); }
    }

    /// Binds a vertex array object.
    pub fn bind_vao(id: u32) {
        unsafe { gl::BindVertexArray(id); }
    }

    /// Unbinds the current VAO (binds 0).
    pub fn unbind_vao() {
        unsafe { gl::BindVertexArray(0); }
    }

    /// Binds an array buffer (`GL_ARRAY_BUFFER`).
    pub fn bind_buffer(id: u32) {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, id); }
    }

    /// Unbinds the current array buffer (binds 0).
    pub fn unbind_buffer() {
        unsafe { gl::BindBuffer(gl::ARRAY_BUFFER, 0); }
    }

    /// Binds an element array buffer (`GL_ELEMENT_ARRAY_BUFFER`).
    pub fn bind_ebo(id: u32) {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, id); }
    }

    /// Unbinds the current element array buffer (binds 0).
    pub fn unbind_ebo() {
        unsafe { gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, 0); }
    }

    /// Binds a shader storage buffer (`GL_SHADER_STORAGE_BUFFER`).
    pub fn bind_ssbo(id: u32) {
        unsafe { gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id); }
    }

    /// Unbinds the current SSBO (binds 0).
    pub fn unbind_ssbo() {
        unsafe { gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, 0); }
    }
}
