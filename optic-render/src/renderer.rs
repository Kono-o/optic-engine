use optic_core::{log_info, Cull, DrawMode, PolyMode, RGBA, Size2D};

use crate::context::RenderContext;
use crate::glraw::GL;
use crate::handles::{Mesh2D, Mesh3D, Shader, Texture2D};
use crate::util::{Transform2D, Transform3D};
use crate::{asset, Camera};

pub struct GPU {
    pub ctx: RenderContext,
    pub poly_mode: PolyMode,
    pub cull_face: Cull,
    pub bg_color: RGBA,
    pub msaa: bool,
    pub msaa_samples: u32,
    pub culling: bool,
    pub fallback_shader2d: Shader,
    pub fallback_shader3d: Shader,
    pub fallback_texture: Texture2D,
    pub canvas_size: Size2D,
}

impl GPU {
    pub fn new_headless() -> optic_core::OpticResult<Self> {
        let ctx = RenderContext::new_headless()?;
        Ok(Self::from_ctx(ctx))
    }

    pub fn new_windowed(
        raw_handle: raw_window_handle::RawWindowHandle,
        size: Size2D,
    ) -> optic_core::OpticResult<Self> {
        let ctx = RenderContext::new_windowed(raw_handle, size)?;
        Ok(Self::from_ctx(ctx))
    }

    fn from_ctx(ctx: RenderContext) -> Self {
        let bg_color = RGBA::grey(0.5);
        GL::enable_depth(true);

        let mut gpu = Self {
            ctx,
            bg_color,
            msaa: true,
            culling: true,
            msaa_samples: 4,
            cull_face: Cull::AntiClock,
            poly_mode: PolyMode::Filled,
            fallback_shader2d: Shader::new(0, false),
            fallback_shader3d: Shader::new(0, false),
            fallback_texture: Texture2D::new(0, Size2D::empty(), optic_core::ImgFormat::RGBA(8), optic_core::ImgFilter::Closest, optic_core::ImgWrap::Repeat),
            canvas_size: Size2D::from(1, 1),
        };

        // Load fallback assets
        if let Ok(fallback_tex) = asset::Image::fallback() {
            let mut tex = fallback_tex;
            tex.set_wrap(optic_core::ImgWrap::Repeat);
            gpu.fallback_texture = gpu.ship_texture(&tex);
        }
        if let Ok(shader_asset) = asset::ShaderAsset::default_3d() {
            if let Some(shader) = gpu.ship_shader(&shader_asset) {
                gpu.fallback_shader3d = shader;
                let mut s = gpu.fallback_shader3d.clone();
                s.attach_tex(&gpu.fallback_texture);
                gpu.fallback_shader3d = s;
            }
        }
        if let Ok(shader_asset) = asset::ShaderAsset::default_2d() {
            if let Some(shader) = gpu.ship_shader(&shader_asset) {
                gpu.fallback_shader2d = shader;
                let mut s = gpu.fallback_shader2d.clone();
                s.attach_tex(&gpu.fallback_texture);
                gpu.fallback_shader2d = s;
            }
        }

        gpu.set_msaa(true);
        gpu.set_culling(true);
        gpu.set_wire_width(2.0);
        gpu.set_bg_color(bg_color);
        GL::enable_alpha(true);
        gpu
    }

    pub fn version(&self) -> &str { &self.ctx.gl_ver }
    pub fn lang_version(&self) -> &str { &self.ctx.glsl_ver }
    pub fn name(&self) -> &str { &self.ctx.device }

    pub fn clear(&self) {
        self.ctx.clear();
    }

    pub fn set_msaa_samples(&mut self, samples: u32) { self.msaa_samples = samples; }

    pub fn set_bg_color(&mut self, color: RGBA) {
        self.bg_color = color;
        self.ctx.set_clear_color(color);
    }

    pub fn set_poly_mode(&mut self, mode: PolyMode) {
        self.poly_mode = mode;
        GL::poly_mode(mode);
    }

    pub fn toggle_wireframe(&mut self) {
        let mode = match self.poly_mode {
            PolyMode::WireFrame => PolyMode::Filled,
            _ => PolyMode::WireFrame,
        };
        self.set_poly_mode(mode);
    }

    pub fn set_msaa(&mut self, enable: bool) {
        self.msaa = enable;
        GL::enable_msaa(enable);
    }

    pub fn toggle_msaa(&mut self) {
        self.msaa = !self.msaa;
        GL::enable_msaa(self.msaa);
    }

    pub fn set_culling(&mut self, enable: bool) {
        self.culling = enable;
        GL::enable_cull(enable);
    }

    pub fn toggle_culling(&mut self) {
        self.culling = !self.culling;
        GL::enable_cull(self.culling);
    }

    pub fn set_cull_face(&mut self, cull_face: Cull) {
        self.cull_face = cull_face;
        GL::set_cull_face(cull_face);
    }

    pub fn flip_cull_face(&mut self) {
        self.cull_face = match self.cull_face {
            Cull::Clock => Cull::AntiClock,
            Cull::AntiClock => Cull::Clock,
        };
        GL::set_cull_face(self.cull_face);
    }

    pub fn set_canvas_size(&mut self, size: Size2D) {
        self.canvas_size = size;
    }

    pub fn set_wire_width(&mut self, width: f32) {
        GL::set_wire_width(width);
    }

    pub fn set_point_size(&self, size: f32) {
        GL::set_point_size(size);
    }

    pub fn log_backend_info(&self) {
        log_info!("BACKEND: {} (GLSL {})", self.ctx.gl_ver, self.ctx.glsl_ver);
    }

    pub fn log_info(&self) {
        log_info!("RENDERER");
        log_info!(
            "> mode: {}",
            match self.poly_mode {
                PolyMode::Points => "POINTS",
                PolyMode::WireFrame => "WIREFRAME",
                PolyMode::Filled => "RASTERIZE",
            }
        );
        log_info!(
            "> cull: {}",
            if self.culling {
                let face = match self.cull_face {
                    Cull::Clock => "clockwise",
                    Cull::AntiClock => "anti-clock",
                };
                format!("ON [{face}]")
            } else {
                "OFF".to_string()
            }
        );
        log_info!(
            "> msaa: {}",
            if self.msaa {
                format!("ON [{} samples]", self.msaa_samples)
            } else {
                "OFF".to_string()
            }
        );
    }

    pub fn fallback_shader3d(&self) -> Shader {
        self.fallback_shader3d.clone()
    }

    pub fn fallback_shader2d(&self) -> Shader {
        self.fallback_shader2d.clone()
    }

    pub fn ship_mesh3d(&self, file: &asset::Mesh3DFile) -> Mesh3D {
        Mesh3D {
            visibility: true,
            handle: file.ship(),
            shader: Some(self.fallback_shader3d()),
            transform: Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        }
    }

    pub fn ship_mesh2d(&self, file: &asset::Mesh2DFile) -> Mesh2D {
        Mesh2D {
            visibility: true,
            handle: file.ship(),
            shader: Some(self.fallback_shader2d()),
            transform: Transform2D::default(),
            draw_mode: DrawMode::Triangles,
        }
    }

    pub fn ship_shader(&self, asset: &asset::ShaderAsset) -> Option<Shader> {
        asset.compile().ok()
    }

    pub fn ship_texture(&self, image: &asset::Image) -> Texture2D {
        image.ship()
    }

    pub fn render3d(&self, mesh: &Mesh3D, camera: &Camera) {
        mesh.render(&camera.transform.view_matrix(), &camera.transform.proj_matrix());
    }

    pub fn render2d(&self, mesh: &Mesh2D) {
        let aspect = if self.canvas_size.w > 0 && self.canvas_size.h > 0 {
            self.canvas_size.w as f32 / self.canvas_size.h as f32
        } else {
            1.0
        };
        let proj = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        mesh.render(&proj);
    }
}
