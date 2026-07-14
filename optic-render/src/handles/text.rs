use cgmath::Matrix4;
use optic_core::{OpticResult, DrawMode};

use crate::handles::font::FontFamily;
use crate::handles::instance::InstanceBuffer;
use crate::handles::Shader;
use crate::text::layout::{
    build_decoration_desc_2d, build_decoration_desc_3d, build_glyph_desc_2d, build_glyph_desc_3d,
    TextLayout,
};
use crate::util::{Transform2D, Transform3D};

fn build_quad_mesh() -> crate::handles::MeshHandle {
    use crate::asset::attr::{Pos2DATTR, ColorATTR, UVMapATTR, IndicesATTR};
    let desc = crate::asset::Mesh2DFile {
        pos_attr: Pos2DATTR::from_array(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ]),
        layer: 0,
        aspect: 1.0,
        col_attr: ColorATTR::from_array(&[[1.0, 1.0, 1.0, 1.0]; 4]),
        uvm_attr: UVMapATTR::from_array(&[
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
            [0.0, 1.0],
        ]),
        ind_attr: IndicesATTR::from_array(&[0u32, 1, 2, 0, 2, 3]),
        custom_attrs: vec![],
    };
    desc.upload()
}

/// Screen-space text renderer using BBCode markup, MSDF atlas fonts, and instanced quad drawing.
///
/// Each glyph is an instanced quad sampling from the font's MSDF atlas texture.
/// `Text2D` is the engine's HUD/UI text primitive — use it for score displays, dialogue,
/// menus, and any screen-aligned text. Supports BBCode markup, word wrapping, and dynamic
/// effects (wave, shake, rainbow, pulse) animated via a time value.
///
/// # Rendering
///
/// 1. Call [`set_text`](Text2D::set_text) to set/update the BBCode string.
/// 2. Assign a shader via [`set_shader`](Text2D::set_shader) (the shader
///    must accept `uProj`, `uLayer`, and texture sampler uniforms).
/// 3. Call [`update`](Text2D::update) each frame with `game.time.elapsed()`
///    to animate dynamic effects.
/// 4. Call [`render`](Text2D::render) with the camera's projection matrix.
pub struct Text2D {
    raw_text: String,
    font: FontFamily,
    shader: Option<Shader>,
    base_size: f32,
    wrap_width: f32,
    transform: Transform2D,
    quad_mesh: crate::handles::MeshHandle,
    glyph_instances: Option<InstanceBuffer>,
    decoration_instances: Option<InstanceBuffer>,
    layout: Option<TextLayout>,
    is_dynamic: bool,
    time: f32,
    visibility: bool,
}

impl Text2D {
    /// Creates a new text object with the given font.
    pub fn new(font: FontFamily) -> Self {
        let quad = build_quad_mesh();

        Text2D {
            raw_text: String::new(),
            font,
            shader: None,
            base_size: 16.0,
            wrap_width: 0.0,
            transform: Transform2D::default(),
            quad_mesh: quad,
            glyph_instances: None,
            decoration_instances: None,
            layout: None,
            is_dynamic: false,
            time: 0.0,
            visibility: true,
        }
    }

    /// Sets the BBCode text and rebuilds the layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_text(&mut self, text: &str) -> OpticResult<()> {
        self.raw_text = text.to_string();
        self.rebuild_layout()
    }

    /// Returns the raw BBCode text.
    pub fn text(&self) -> &str {
        &self.raw_text
    }

    /// Replaces the font and rebuilds the layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_font(&mut self, font: FontFamily) -> OpticResult<()> {
        self.font = font;
        self.rebuild_layout()
    }

    /// Assigns a custom shader for rendering.
    pub fn set_shader(&mut self, shader: Shader) {
        self.shader = Some(shader);
    }

    /// Removes the custom shader, disabling rendering.
    pub fn remove_shader(&mut self) {
        self.shader = None;
    }

    /// Returns a reference to the current shader, if set.
    pub fn shader(&self) -> Option<&Shader> {
        self.shader.as_ref()
    }

    /// Sets the base font size in pixels and rebuilds the layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_base_size(&mut self, size: f32) -> OpticResult<()> {
        self.base_size = size;
        self.rebuild_layout()
    }

    /// Sets the word-wrap width in pixels. `0` disables wrapping.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_wrap_width(&mut self, width: f32) -> OpticResult<()> {
        self.wrap_width = width;
        self.rebuild_layout()
    }

    /// Returns a reference to the 2D transform.
    pub fn transform(&self) -> &Transform2D {
        &self.transform
    }

    /// Returns a mutable reference to the 2D transform.
    pub fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    /// Updates dynamic effects (wave, shake, rainbow, pulse) with the given time.
    ///
    /// Call this each frame with `game.time.elapsed()` to animate effects.
    /// No-op for static text.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU instance buffers fail to upload.
    pub fn update(&mut self, time: f32) -> OpticResult<()> {
        self.time = time;
        if !self.is_dynamic {
            return Ok(());
        }
        self.write_buffers()
    }

    /// Returns `true` if the text contains any dynamic effects.
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    /// Shows or hides the text.
    pub fn set_visibility(&mut self, visible: bool) {
        self.visibility = visible;
    }

    /// Returns whether the text is visible.
    pub fn is_visible(&self) -> bool {
        self.visibility
    }

    /// Renders the text with the given projection matrix.
    ///
    /// Does nothing if invisible, has no glyphs, or no shader is assigned.
    pub fn render(&mut self, proj: &Matrix4<f32>) {
        if !self.visibility {
            return;
        }
        let has_glyphs = self.glyph_instances.as_ref().map_or(false, |gi| gi.count() > 0);
        if !has_glyphs {
            return;
        }

        let shader = match &self.shader {
            Some(sh) => sh,
            None => return,
        };

        shader.bind();
        shader.set_m4_f32("uProj", *proj);
        shader.set_u32("uLayer", self.transform.layer() as u32);

        self.bind_textures_and_draw(proj);
    }

    fn bind_textures_and_draw(&mut self, _proj: &Matrix4<f32>) {
        if let Some(shader) = &mut self.shader {
            let tex = self.font.primary_atlas();
            shader.attach_texture(tex);
            shader.bind_textures();
            shader.bind_storages();
        }

        if let Some(gi) = &self.glyph_instances {
            self.quad_mesh.set_instances(gi);
            self.quad_mesh.draw_as(DrawMode::Triangles);
        }
        if let Some(di) = &self.decoration_instances {
            if di.count() > 0 {
                self.quad_mesh.set_instances(di);
                self.quad_mesh.draw_as(DrawMode::Triangles);
            }
        }
    }

    /// Deletes the GPU mesh and font resources.
    pub fn delete(self) {
        self.quad_mesh.delete();
        self.font.delete();
    }

    fn rebuild_layout(&mut self) -> OpticResult<()> {
        let layout = crate::text::layout::layout_text(
            &self.raw_text,
            &self.font,
            self.base_size,
            self.wrap_width,
        )?;
        self.is_dynamic = layout.is_dynamic;
        self.layout = Some(layout);
        self.write_buffers()
    }

    fn write_buffers(&mut self) -> OpticResult<()> {
        let layout = self.layout.as_ref().expect("layout cache");
        let glyph_desc = build_glyph_desc_2d(layout, self.time);
        let deco_desc = build_decoration_desc_2d(layout, self.time);

        self.glyph_instances = if !glyph_desc.pos_attr.is_empty() {
            Some(glyph_desc.upload()?)
        } else {
            None
        };
        self.decoration_instances = if !deco_desc.pos_attr.is_empty() {
            Some(deco_desc.upload()?)
        } else {
            None
        };
        Ok(())
    }
}

/// World-space billboard text renderer, same API as Text2D but with 3D positioning.
///
/// Renders text as camera-facing quads in 3D space using the same MSDF instanced
/// pipeline as [`Text2D`]. Position and orient the text via its [`Transform3D`] and
/// render through a camera's view/projection matrices. Use for floating labels,
/// damage numbers, or any text that must exist in the 3D scene.
pub struct Text3D {
    raw_text: String,
    font: FontFamily,
    shader: Option<Shader>,
    base_size: f32,
    wrap_width: f32,
    transform: Transform3D,
    quad_mesh: crate::handles::MeshHandle,
    glyph_instances: Option<InstanceBuffer>,
    decoration_instances: Option<InstanceBuffer>,
    layout: Option<TextLayout>,
    is_dynamic: bool,
    time: f32,
    visibility: bool,
}

impl Text3D {
    /// Creates a new 3D text object with the given font.
    pub fn new(font: FontFamily) -> Self {
        let quad = build_quad_mesh();

        Text3D {
            raw_text: String::new(),
            font,
            shader: None,
            base_size: 16.0,
            wrap_width: 0.0,
            transform: Transform3D::default(),
            quad_mesh: quad,
            glyph_instances: None,
            decoration_instances: None,
            layout: None,
            is_dynamic: false,
            time: 0.0,
            visibility: true,
        }
    }

    /// Sets the BBCode text and rebuilds the layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_text(&mut self, text: &str) -> OpticResult<()> {
        self.raw_text = text.to_string();
        self.rebuild_layout()
    }

    /// Returns the raw BBCode text.
    pub fn text(&self) -> &str {
        &self.raw_text
    }

    /// Replaces the font and rebuilds the layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_font(&mut self, font: FontFamily) -> OpticResult<()> {
        self.font = font;
        self.rebuild_layout()
    }

    /// Assigns a custom shader for rendering.
    pub fn set_shader(&mut self, shader: Shader) {
        self.shader = Some(shader);
    }

    /// Removes the custom shader, disabling rendering.
    pub fn remove_shader(&mut self) {
        self.shader = None;
    }

    /// Returns a reference to the current shader, if set.
    pub fn shader(&self) -> Option<&Shader> {
        self.shader.as_ref()
    }

    /// Sets the base font size in pixels and rebuilds the layout.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_base_size(&mut self, size: f32) -> OpticResult<()> {
        self.base_size = size;
        self.rebuild_layout()
    }

    /// Sets the word-wrap width in pixels. `0` disables wrapping.
    ///
    /// # Errors
    ///
    /// Returns an error if the BBCode is malformed or GPU instance buffers
    /// fail to upload.
    pub fn set_wrap_width(&mut self, width: f32) -> OpticResult<()> {
        self.wrap_width = width;
        self.rebuild_layout()
    }

    /// Returns a reference to the 3D transform.
    pub fn transform(&self) -> &Transform3D {
        &self.transform
    }

    /// Returns a mutable reference to the 3D transform.
    pub fn transform_mut(&mut self) -> &mut Transform3D {
        &mut self.transform
    }

    /// Updates dynamic effects with the given time value.
    ///
    /// # Errors
    ///
    /// Returns an error if GPU instance buffers fail to upload.
    pub fn update(&mut self, time: f32) -> OpticResult<()> {
        self.time = time;
        if !self.is_dynamic {
            return Ok(());
        }
        self.write_buffers()
    }

    /// Returns `true` if the text contains any dynamic effects.
    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    /// Shows or hides the text.
    pub fn set_visibility(&mut self, visible: bool) {
        self.visibility = visible;
    }

    /// Returns whether the text is visible.
    pub fn is_visible(&self) -> bool {
        self.visibility
    }

    /// Renders the text with view and projection matrices.
    pub fn render(&mut self, view: &Matrix4<f32>, proj: &Matrix4<f32>) {
        if !self.visibility {
            return;
        }
        let has_glyphs = self.glyph_instances.as_ref().map_or(false, |gi| gi.count() > 0);
        if !has_glyphs {
            return;
        }

        let shader = match &self.shader {
            Some(sh) => sh,
            None => return,
        };

        shader.bind();
        shader.set_m4_f32("uView", *view);
        shader.set_m4_f32("uProj", *proj);
        shader.set_m4_f32("uTfm", self.transform.matrix());

        if let Some(shader) = &mut self.shader {
            let tex = self.font.primary_atlas();
            shader.attach_texture(tex);
            shader.bind_textures();
            shader.bind_storages();
        }

        if let Some(gi) = &self.glyph_instances {
            self.quad_mesh.set_instances(gi);
            self.quad_mesh.draw_as(DrawMode::Triangles);
        }
        if let Some(di) = &self.decoration_instances {
            if di.count() > 0 {
                self.quad_mesh.set_instances(di);
                self.quad_mesh.draw_as(DrawMode::Triangles);
            }
        }
    }

    pub fn delete(self) {
        self.quad_mesh.delete();
        self.font.delete();
    }

    fn rebuild_layout(&mut self) -> OpticResult<()> {
        let layout = crate::text::layout::layout_text(
            &self.raw_text,
            &self.font,
            self.base_size,
            self.wrap_width,
        )?;
        self.is_dynamic = layout.is_dynamic;
        self.layout = Some(layout);
        self.write_buffers()
    }

    fn write_buffers(&mut self) -> OpticResult<()> {
        let layout = self.layout.as_ref().expect("layout cache");
        let glyph_desc = build_glyph_desc_3d(layout, self.time);
        let deco_desc = build_decoration_desc_3d(layout, self.time);

        self.glyph_instances = if !glyph_desc.pos_attr.is_empty() {
            Some(glyph_desc.upload()?)
        } else {
            None
        };
        self.decoration_instances = if !deco_desc.pos_attr.is_empty() {
            Some(deco_desc.upload()?)
        } else {
            None
        };
        Ok(())
    }
}
