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

/// Screen-space (HUD / UI) text rendered with instanced quads.
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

    pub fn set_text(&mut self, text: &str) -> OpticResult<()> {
        self.raw_text = text.to_string();
        self.rebuild_layout()
    }

    pub fn text(&self) -> &str {
        &self.raw_text
    }

    pub fn set_font(&mut self, font: FontFamily) -> OpticResult<()> {
        self.font = font;
        self.rebuild_layout()
    }

    pub fn set_shader(&mut self, shader: Shader) {
        self.shader = Some(shader);
    }

    pub fn remove_shader(&mut self) {
        self.shader = None;
    }

    pub fn shader(&self) -> Option<&Shader> {
        self.shader.as_ref()
    }

    pub fn set_base_size(&mut self, size: f32) -> OpticResult<()> {
        self.base_size = size;
        self.rebuild_layout()
    }

    pub fn set_wrap_width(&mut self, width: f32) -> OpticResult<()> {
        self.wrap_width = width;
        self.rebuild_layout()
    }

    pub fn transform(&self) -> &Transform2D {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform2D {
        &mut self.transform
    }

    pub fn update(&mut self, time: f32) -> OpticResult<()> {
        self.time = time;
        if !self.is_dynamic {
            return Ok(());
        }
        self.write_buffers()
    }

    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.visibility = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visibility
    }

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

/// World-space billboard text rendered via instanced quads.
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

    pub fn set_text(&mut self, text: &str) -> OpticResult<()> {
        self.raw_text = text.to_string();
        self.rebuild_layout()
    }

    pub fn text(&self) -> &str {
        &self.raw_text
    }

    pub fn set_font(&mut self, font: FontFamily) -> OpticResult<()> {
        self.font = font;
        self.rebuild_layout()
    }

    pub fn set_shader(&mut self, shader: Shader) {
        self.shader = Some(shader);
    }

    pub fn remove_shader(&mut self) {
        self.shader = None;
    }

    pub fn shader(&self) -> Option<&Shader> {
        self.shader.as_ref()
    }

    pub fn set_base_size(&mut self, size: f32) -> OpticResult<()> {
        self.base_size = size;
        self.rebuild_layout()
    }

    pub fn set_wrap_width(&mut self, width: f32) -> OpticResult<()> {
        self.wrap_width = width;
        self.rebuild_layout()
    }

    pub fn transform(&self) -> &Transform3D {
        &self.transform
    }

    pub fn transform_mut(&mut self) -> &mut Transform3D {
        &mut self.transform
    }

    pub fn update(&mut self, time: f32) -> OpticResult<()> {
        self.time = time;
        if !self.is_dynamic {
            return Ok(());
        }
        self.write_buffers()
    }

    pub fn is_dynamic(&self) -> bool {
        self.is_dynamic
    }

    pub fn set_visibility(&mut self, visible: bool) {
        self.visibility = visible;
    }

    pub fn is_visible(&self) -> bool {
        self.visibility
    }

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
