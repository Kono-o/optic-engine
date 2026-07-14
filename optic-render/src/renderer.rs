//! High-level GPU renderer and pipeline state management.
//!
//! [`GPU`] is the primary entry point for all rendering operations. It owns an
//! EGL-backed [`RenderContext`], pre-loaded fallback assets (shaders, textures,
//! fonts), and the global rasterization pipeline state (polygon mode, culling,
//! MSAA, clear colour).
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────┐
//! │  GPU                                            │
//! │  ┌───────────────┐  ┌────────────────────────┐  │
//! │  │ RenderContext  │  │  Fallback assets       │  │
//! │  │ (EGL display,  │  │  • 2D / 3D shaders     │  │
//! │  │  surfaces,     │  │  • checkerboard tex    │  │
//! │  │  vsync)        │  │  • built-in font       │  │
//! │  └───────────────┘  └────────────────────────┘  │
//! │  ┌─────────────────────────────────────────────┐ │
//! │  │  Pipeline state                             │ │
//! │  │  polygon mode · culling · MSAA · clear colour│ │
//! │  └─────────────────────────────────────────────┘ │
//! └─────────────────────────────────────────────────┘
//! ```
//!
//! # Draw calls
//!
//! | Method | Geometry | Projection |
//! |--------|----------|------------|
//! | [`render3d`](GPU::render3d) | [`Mesh3D`] | Camera perspective/ortho |
//! | [`render2d`](GPU::render2d) | [`Mesh2D`] | Canvas orthographic |
//! | [`render_text2d`](GPU::render_text2d) | [`Text2D`] | Canvas orthographic |
//! | [`render_text3d`](GPU::render_text3d) | [`Text3D`] | Camera perspective/ortho |
//!
//! # Thread safety
//!
//! `GPU` is **not** `Send` — OpenGL contexts are bound to a single OS thread.
//! All rendering must happen on the thread that created the context.

use optic_core::{log_info, ColorInfo, CullFace, Gradient, OpticResult, PolygonMode, RGBA, Size2D};

use crate::context::RenderContext;
use crate::glraw::GL;
use crate::handles::{Canvas, FontFamily, Mesh2D, Mesh3D, RenderTarget, Shader, Text2D, Text3D, Texture2D};
use crate::{asset, Camera};

/// A rectangular region of the render target, measured in pixels from the lower-left corner.
///
/// Describes the viewport or scissor rectangle that the OpenGL pipeline uses to map
/// NDC coordinates to window pixels. Used by [`GPU::viewport`] and [`GPU::set_viewport`]
/// to restrict rendering to a sub-area of the canvas, and by canvases to set renderable
/// regions for split-screen or picture-in-picture effects.
pub struct Viewport {
    /// Horizontal offset from the left edge of the window, in pixels.
    pub x: i32,
    /// Vertical offset from the bottom edge of the window, in pixels.
    pub y: i32,
    /// Width of the viewport rectangle, in pixels.
    pub width: i32,
    /// Height of the viewport rectangle, in pixels.
    pub height: i32,
}

/// The primary renderer — owns the GL context, fallback assets, and global
/// pipeline state.
///
/// `GPU` is the central rendezvous point between CPU and GPU. It owns:
///
/// | Owner | Type | Purpose |
/// |---|---|---|
/// | GL context | [`RenderContext`] | EGL/GLX/WGL surface and function pointers |
/// | Fallback shaders | [`Shader`] | Built-in 2D and 3D shaders (used when no custom shader is provided) |
/// | Fallback texture | [`Texture2D`] | Checkerboard texture for untextured meshes |
/// | Pipeline config | — | Polygon mode, culling, MSAA, clear colour |
///
/// # Lifecycle
///
/// A `GPU` is created once at startup and lives for the entire application
/// session. It is **not** `Send` — OpenGL contexts are thread-bound on most
/// platforms.
///
/// # Creating a GPU
///
/// | Constructor | Use case |
/// |---|---|
/// | [`GPU::new_headless`](Self::new_headless) | Off-screen / compute-only (no window) |
/// | [`GPU::new_windowed`](Self::new_windowed) | On-screen rendering |
///
/// # Fallback assets
///
/// On construction, `GPU` loads built-in default shaders and a checkerboard
/// fallback texture. [`upload_mesh3d`](Self::upload_mesh3d) and
/// [`upload_mesh2d`](Self::upload_mesh2d) attach these automatically, so you can
/// render something immediately without providing custom shaders.
///
/// # Example
///
/// ```ignore
/// use optic_render::GPU;
///
/// let gpu = GPU::new_headless()?;
/// gpu.log_backend_info();
/// // → "BACKEND: 4.6.0 NVIDIA ... (GLSL 4.60)"
/// ```
pub struct GPU {
    pub(crate) ctx: RenderContext,
    poly_mode: PolygonMode,
    cull_face: CullFace,
    bg_color: RGBA,
    msaa: bool,
    msaa_samples: u32,
    culling: bool,
    fallback_shader2d: Shader,
    fallback_shader3d: Shader,
    fallback_texture: Texture2D,
    fallback_font: FontFamily,
    canvas_size: Size2D,
    pub(crate) current_target_size: Size2D,
    pub(crate) max_color_attachments: i32,
    pub(crate) max_draw_buffers: i32,
    pub(crate) max_samples: i32,
    fallback_shader_text2d: Shader,
    fallback_shader_text3d: Shader,
}

impl GPU {
    /// Creates a headless GPU context (no window).
    ///
    /// Useful for:
    /// - Off-screen rendering (compute FBOs)
    /// - Automated testing
    /// - Server-side rendering
    ///
    /// The context uses an EGL `PBuffer` surface (or platform equivalent).
    pub fn new_headless() -> optic_core::OpticResult<Self> {
        let ctx = RenderContext::new_headless()?;
        Ok(Self::from_ctx(ctx))
    }

    /// Creates a GPU context backed by an on-screen window.
    ///
    /// `raw_handle` and `display_handle` come from the windowing system
    /// (e.g. `winit`). See the `optic_window` crate for how to acquire them.
    ///
    /// # Errors
    ///
    /// Returns an error if the EGL/GLX surface cannot be created.
    pub fn new_windowed(
        raw_handle: raw_window_handle::RawWindowHandle,
        display_handle: raw_window_handle::RawDisplayHandle,
        size: Size2D,
    ) -> optic_core::OpticResult<Self> {
        let ctx = RenderContext::new_windowed(raw_handle, display_handle, size)?;
        Ok(Self::from_ctx(ctx))
    }

    fn from_ctx(ctx: RenderContext) -> Self {
        let bg_color = RGBA::grey(0.5);
        GL::enable_depth(true);

        let max_color_attachments = unsafe {
            let mut v = 0i32;
            gl::GetIntegerv(gl::MAX_COLOR_ATTACHMENTS, &mut v);
            v
        };
        let max_draw_buffers = unsafe {
            let mut v = 0i32;
            gl::GetIntegerv(gl::MAX_DRAW_BUFFERS, &mut v);
            v
        };
        let max_samples = unsafe {
            let mut v = 0i32;
            gl::GetIntegerv(gl::MAX_SAMPLES, &mut v);
            v
        };

        let canvas_size = Size2D::new(1, 1);
        let mut gpu = Self {
            ctx,
            bg_color,
            msaa: true,
            culling: true,
            msaa_samples: 4,
            cull_face: CullFace::AntiClock,
            poly_mode: PolygonMode::Filled,
            fallback_shader2d: Shader::new(0, false),
            fallback_shader3d: Shader::new(0, false),
            fallback_texture: Texture2D::new(0, Size2D::zero(), optic_core::ImgFormat::RGBA(8), optic_core::ImgFilter::Closest, optic_core::ImgWrap::Repeat),
            fallback_font: FontFamily::default(),
            canvas_size,
            current_target_size: canvas_size,
            max_color_attachments,
            max_draw_buffers,
            max_samples,
            fallback_shader_text2d: Shader::new(0, false),
            fallback_shader_text3d: Shader::new(0, false),
        };

        if let Ok(fallback_tex) = asset::TextureFile::fallback() {
            let mut tex = fallback_tex;
            tex.set_wrap(optic_core::ImgWrap::Repeat);
            gpu.fallback_texture = gpu.upload_texture(&tex);
        }
        if let Ok(shader_asset) = asset::ShaderFile::default_3d() {
            if let Ok(shader) = gpu.upload_shader(&shader_asset) {
                gpu.fallback_shader3d = shader;
                let mut s = gpu.fallback_shader3d.clone();
                s.attach_texture(&gpu.fallback_texture);
                gpu.fallback_shader3d = s;
            }
        }
        if let Ok(shader_asset) = asset::ShaderFile::default_2d() {
            if let Ok(shader) = gpu.upload_shader(&shader_asset) {
                gpu.fallback_shader2d = shader;
                let mut s = gpu.fallback_shader2d.clone();
                s.attach_texture(&gpu.fallback_texture);
                gpu.fallback_shader2d = s;
            }
        }
        if let Ok(shader_asset) = asset::ShaderFile::from_disk("assets/shdr/fallback_text2d.glsl", asset::ShaderType::Pipeline) {
            if let Ok(shader) = gpu.upload_shader(&shader_asset) {
                gpu.fallback_shader_text2d = shader;
            }
        }
        if let Ok(shader_asset) = asset::ShaderFile::from_disk("assets/shdr/fallback_text3d.glsl", asset::ShaderType::Pipeline) {
            if let Ok(shader) = gpu.upload_shader(&shader_asset) {
                gpu.fallback_shader_text3d = shader;
            }
        }

        gpu.set_msaa(true);
        gpu.set_culling(true);
        gpu.set_wire_width(2.0);
        gpu.set_bg_color(bg_color);
        GL::enable_alpha(true);
        gpu
    }

    // ── Backend info ─────────────────────────────────────────────────────

    /// Returns the OpenGL version string, e.g. `"4.6.0 NVIDIA 545.84"`.
    pub fn version(&self) -> &str { self.ctx.gl_ver() }

    /// Returns the GLSL version string, e.g. `"4.60"`.
    pub fn lang_version(&self) -> &str { self.ctx.glsl_ver() }

    /// Returns the GPU device name, e.g. `"GeForce RTX 3080"`.
    pub fn name(&self) -> &str { self.ctx.device() }

    /// Logs the OpenGL and GLSL version at the `INFO` level.
    pub fn log_backend_info(&self) {
        log_info!("BACKEND: {} (GLSL {})", self.ctx.gl_ver(), self.ctx.glsl_ver());
    }

    /// Logs the current pipeline configuration (polygon mode, culling, MSAA)
    /// at the `INFO` level.
    pub fn log_info(&self) {
        log_info!("RENDERER");
        log_info!(
            "> mode: {}",
            match self.poly_mode {
                PolygonMode::Points => "POINTS",
                PolygonMode::WireFrame => "WIREFRAME",
                PolygonMode::Filled => "RASTERIZE",
            }
        );
        log_info!(
            "> cull: {}",
            if self.culling {
                let face = match self.cull_face {
        CullFace::Clock => "clockwise",
        CullFace::AntiClock => "anti-clock",
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

    // ── Clearing ─────────────────────────────────────────────────────────

    /// Clears the current render target using the configured background colour.
    ///
    /// Equivalent to `clear_target(None, true)` — preserves `bg_color` and
    /// clears depth.
    pub fn clear(&self) {
        self.ctx.clear();
    }

    /// Clears the currently-bound render target with explicit control.
    ///
    /// | Parameter | `Some` / `true` | `None` / `false` |
    /// |---|---|---|
    /// | `color` | Sets `bg_color` and clears colour buffer | Leaves colour buffer untouched |
    /// | `depth` | Clears depth buffer | Leaves depth buffer untouched |
    ///
    /// ```ignore
    /// // Clear colour to red, leave depth as-is
    /// gpu.clear_target(Some(RED.into()), false);
    ///
    /// // Clear both with current bg_color
    /// gpu.clear_target(None, true);
    /// ```
    pub fn clear_target(&mut self, color: Option<RGBA>, depth: bool) {
        let mut mask = 0u32;
        if let Some(c) = color {
            self.ctx.set_clear_color(c);
            self.bg_color = c;
            mask |= gl::COLOR_BUFFER_BIT;
        }
        if depth {
            mask |= gl::DEPTH_BUFFER_BIT;
        }
        if mask != 0 {
            unsafe { gl::Clear(mask); }
            }
        }


    // ── Pipeline state ───────────────────────────────────────────────────

    /// Sets the background clear colour.
    ///
    /// Applied the next time [`clear`](Self::clear) or
    /// [`clear_target`](Self::clear_target) is called.
    pub fn set_bg_color(&mut self, color: RGBA) {
        self.bg_color = color;
        self.ctx.set_clear_color(color);
    }

    /// Sets the polygon rasterization mode.
    ///
    /// | Mode | Effect |
    /// |---|---|
    /// | [`PolygonMode::Filled`] | Solid triangles (default) |
    /// | [`PolygonMode::WireFrame`] | Triangle outlines |
    /// | [`PolygonMode::Points`] | Vertex points |
    pub fn set_poly_mode(&mut self, mode: PolygonMode) {
        self.poly_mode = mode;
        GL::poly_mode(mode);
    }

    /// Toggles between filled and wireframe mode.
    pub fn toggle_wireframe(&mut self) {
        let mode = match self.poly_mode {
            PolygonMode::WireFrame => PolygonMode::Filled,
            _ => PolygonMode::WireFrame,
        };
        self.set_poly_mode(mode);
    }

    /// Sets the line width used in wireframe mode.
    ///
    /// > **Note**: `glLineWidth` is deprecated in core OpenGL and may not
    /// > work on all drivers. Prefer a geometry shader for thick lines.
    pub fn set_wire_width(&mut self, width: f32) {
        GL::set_wire_width(width);
    }

    /// Sets the point size used when [`PolygonMode::Points`] is active.
    pub fn set_point_size(&self, size: f32) {
        GL::set_point_size(size);
    }

    // ── MSAA ─────────────────────────────────────────────────────────────

    /// Enables or disables multisample anti-aliasing.
    pub fn set_msaa(&mut self, enable: bool) {
        self.msaa = enable;
        GL::enable_msaa(enable);
    }

    /// Toggles MSAA on/off.
    pub fn toggle_msaa(&mut self) {
        self.msaa = !self.msaa;
        GL::enable_msaa(self.msaa);
    }

    /// Sets the MSAA sample count for newly created canvases.
    ///
    /// Existing canvases are not affected. The driver's maximum sample count
    /// is available at runtime via the driver's `GL_MAX_SAMPLES` query.
    pub fn set_msaa_samples(&mut self, samples: u32) { self.msaa_samples = samples; }

    // ── Culling ──────────────────────────────────────────────────────────

    /// Enables or disables back-face culling.
    pub fn set_culling(&mut self, enable: bool) {
        self.culling = enable;
        GL::enable_cull(enable);
    }

    /// Toggles back-face culling on/off.
    pub fn toggle_culling(&mut self) {
        self.culling = !self.culling;
        GL::enable_cull(self.culling);
    }

    /// Sets which face is culled (clockwise or counter-clockwise).
    ///
    /// Vertices in counter-clockwise order (the default in OpenGL) are
    /// front-facing. Set to [`CullFace::Clock`] to cull the front face instead.
    pub fn set_cull_face(&mut self, cull_face: CullFace) {
        self.cull_face = cull_face;
        GL::set_cull_face(cull_face);
    }

    /// Flips the cull face between clockwise and counter-clockwise.
    pub fn flip_cull_face(&mut self) {
        self.cull_face = match self.cull_face {
        CullFace::Clock => CullFace::AntiClock,
        CullFace::AntiClock => CullFace::Clock,
        };
        GL::set_cull_face(self.cull_face);
    }

    /// Returns the background clear colour.
    pub fn bg_color(&self) -> &RGBA { &self.bg_color }
    /// Returns the polygon rasterization mode.
    pub fn poly_mode(&self) -> PolygonMode { self.poly_mode }
    /// Returns `true` if MSAA is enabled.
    pub fn msaa(&self) -> bool { self.msaa }
    /// Returns the MSAA sample count.
    pub fn msaa_samples(&self) -> u32 { self.msaa_samples }
    /// Returns `true` if back-face culling is enabled.
    pub fn culling(&self) -> bool { self.culling }
    /// Returns which face is culled.
    pub fn cull_face(&self) -> CullFace { self.cull_face }
    /// Returns a reference to the render context.
    pub fn ctx(&self) -> &RenderContext { &self.ctx }
    /// Returns a mutable reference to the render context.
    pub fn ctx_mut(&mut self) -> &mut RenderContext { &mut self.ctx }
    /// Returns a reference to the fallback texture.
    pub fn fallback_texture(&self) -> &Texture2D { &self.fallback_texture }
    /// Returns the logical canvas size.
    pub fn canvas_size(&self) -> Size2D { self.canvas_size }

    // ─── Canvas / viewport ───────────────────────────────────────────────

    /// Sets the logical canvas size (defines the 2D orthographic projection).
    ///
    /// This is the size used by [`render2d`](Self::render2d) to compute its
    /// aspect-correct orthographic matrix.
    pub fn set_canvas_size(&mut self, size: Size2D) {
        self.canvas_size = size;
    }

    /// Returns the size of the currently-bound render target.
    ///
    /// Updated whenever [`set_render_target`](Self::set_render_target) is
    /// called.
    pub fn current_render_target_size(&self) -> Size2D {
        self.current_target_size
    }

    /// Returns the current OpenGL viewport rect.
    pub fn viewport(&self) -> Viewport {
        let mut vp = [0i32; 4];
        unsafe { gl::GetIntegerv(gl::VIEWPORT, vp.as_mut_ptr()); }
        Viewport { x: vp[0], y: vp[1], width: vp[2], height: vp[3] }
    }

    /// Sets the OpenGL viewport rect.
    pub fn set_viewport(&self, vp: Viewport) {
        unsafe { gl::Viewport(vp.x, vp.y, vp.width, vp.height); }
    }

    /// Flushes OpenGL commands (non-blocking).
    pub fn flush(&self) {
        unsafe { gl::Flush(); }
    }

    /// Blocks until all OpenGL commands are complete.
    pub fn finish(&self) {
        unsafe { gl::Finish(); }
    }

    /// Resets the GPU back to default state.
    ///
    /// This clears all cached state on the GPU and resets the renderer's
    /// internal state to match defaults.
    pub fn reset_state(&mut self) {
        self.bg_color = RGBA::new(0.0, 0.0, 0.0, 1.0);
        self.poly_mode = PolygonMode::Filled;
        self.msaa = false;
        self.culling = false;
        self.cull_face = CullFace::AntiClock;
        self.current_target_size = self.canvas_size;
        let _ = self.set_render_target(&RenderTarget::Screen);
    }

    // ── Asset upload ─────────────────────────────────────────────────────

    /// Returns a clone of the 3D fallback shader (with the fallback texture
    /// pre-bound).
    ///
    /// Useful when you want to render a mesh without writing a custom shader:
    ///
    /// ```ignore
    /// let mut mesh = gpu.upload_mesh3d(&my_mesh_file);
    /// mesh.shader = Some(gpu.fallback_shader3d());
    /// ```
    pub fn fallback_shader3d(&self) -> Shader {
        self.fallback_shader3d.clone()
    }

    /// Returns a clone of the 2D fallback shader (with the fallback texture
    /// pre-bound).
    pub fn fallback_shader2d(&self) -> Shader {
        self.fallback_shader2d.clone()
    }

    /// Uploads a [`Mesh3DFile`](asset::Mesh3DFile) to the GPU and returns a
    /// [`Mesh3D`] with the fallback 3D shader attached.
    ///
    /// The returned mesh is immediately renderable:
    ///
    /// ```ignore
    /// let cube = Mesh3DFile::cube(2.0);
    /// let mesh = gpu.upload_mesh3d(&cube);
    /// gpu.render3d(&mesh, &camera);
    /// ```
    pub fn upload_mesh3d(&self, file: &asset::Mesh3DFile) -> Mesh3D {
        let mut mesh = Mesh3D::new(file.upload());
        mesh.set_shader(self.fallback_shader3d());
        mesh
    }

    /// Uploads a [`Mesh2DFile`](asset::Mesh2DFile) to the GPU and returns a
    /// [`Mesh2D`] with the fallback 2D shader attached.
    pub fn upload_mesh2d(&self, file: &asset::Mesh2DFile) -> Mesh2D {
        let mut mesh = Mesh2D::new(file.upload());
        mesh.set_shader(self.fallback_shader2d());
        mesh
    }

    /// Uploads a [`FontFamilyFile`](asset::FontFamilyFile) to the GPU and
    /// returns a [`FontFamily`].
    ///
    /// This uploads the MSDF atlas textures for each populated style variant,
    /// making the font ready for use with [`Text2D`] or [`Text3D`].
    ///
    /// # Errors
    ///
    /// Returns [`OpticError`](optic_core::OpticError) if the atlas texture
    /// upload fails (e.g. the GPU rejects the image dimensions or format).
    pub fn upload_font_family(&self, file: &asset::FontFamilyFile) -> OpticResult<FontFamily> {
        FontFamily::new(file)
    }

    /// Returns a clone of the fallback font, used by [`Text2D`] / [`Text3D`]
    /// when no explicit font is set.
    pub fn fallback_font(&self) -> FontFamily {
        self.fallback_font.clone()
    }

    /// Returns a clone of the fallback 2D text shader.
    pub fn fallback_shader_text2d(&self) -> Shader {
        let mut sh = self.fallback_shader_text2d.clone();
        sh.attach_texture(&self.fallback_texture);
        sh
    }

    /// Returns a clone of the fallback 3D text shader.
    pub fn fallback_shader_text3d(&self) -> Shader {
        self.fallback_shader_text3d.clone()
    }

    /// Compiles a [`ShaderFile`](asset::ShaderFile) into a usable [`Shader`].
    ///
    /// Returns `Err` if compilation or linking fails (errors are logged
    /// internally by the GLSL compiler). A returned shader is ready to bind
    /// uniforms and attach textures.
    ///
    /// # Errors
    ///
    /// Returns an [`OpticError`](optic_core::OpticError) with kind
    /// [`Shader`](optic_core::OpticErrorKind::Shader) if GLSL compilation or
    /// program linking fails. The error message contains the driver's info log.
    pub fn upload_shader(&self, asset: &asset::ShaderFile) -> optic_core::OpticResult<Shader> {
        asset.compile()
    }

    /// Uploads a [`TextureFile`](asset::TextureFile) to the GPU.
    ///
    /// ```ignore
    /// let tex_file = TextureFile::from_disk("assets/grass.png")?;
    /// let tex: Texture2D = gpu.upload_texture(&tex_file);
    /// ```
    pub fn upload_texture(&self, image: &asset::TextureFile) -> Texture2D {
        image.upload()
    }

    /// Bakes a [`Gradient`] into a 1-pixel-high 2D texture (colour ramp).
    ///
    /// `resolution` controls the width of the texture (number of samples).
    /// Each sample is linearly interpolated from the gradient and stored as
    /// `RGBA8`.
    ///
    /// ```ignore
    /// use optic_core::Gradient;
    ///
    /// let grad = Gradient::rainbow();
    /// let ramp: Texture2D = gpu.upload_gradient(&grad, 256);
    /// // Use as a lookup texture in a shader
    /// ```
    pub fn upload_gradient(&self, gradient: &Gradient, resolution: u32) -> Texture2D {
        let res = resolution.max(1);
        let colors = gradient.sample_n(res as usize);
        let mut bytes = Vec::with_capacity(res as usize * 4);
        for c in &colors {
            let (r, g, b, a) = c.to_bytes();
            bytes.push(r);
            bytes.push(g);
            bytes.push(b);
            bytes.push(a);
        }
        let size = Size2D::new(res, 1);
        let id = crate::handles::texture::create_texture(
            &bytes,
            size,
            &optic_core::ImgFormat::RGBA(8),
            &optic_core::ImgFilter::Linear,
            &optic_core::ImgWrap::Clip,
        );
        Texture2D::new(id, size, optic_core::ImgFormat::RGBA(8), optic_core::ImgFilter::Linear, optic_core::ImgWrap::Clip)
    }

    /// Creates a [`Canvas`] (FBO) with hardware capability validation.
    ///
    /// Checks the descriptor against the driver's maximum colour attachments,
    /// draw buffers, and MSAA samples before creating the FBO.
    ///
    /// # Errors
    ///
    /// | Condition | Error |
    /// |---|---|
    /// | Too many colour attachments | `"exceeds GL_MAX_COLOR_ATTACHMENTS"` |
    /// | Too many draw buffers | `"exceeds GL_MAX_DRAW_BUFFERS"` |
    /// | Too many MSAA samples | `"exceeds GL_MAX_SAMPLES"` |
    /// | FBO incomplete | Framebuffer status error |
    pub fn upload_canvas(&mut self, desc: &crate::handles::CanvasDesc) -> optic_core::OpticResult<Canvas> {
        if desc.color_formats.len() as i32 > self.max_color_attachments {
            return Err(optic_core::OpticError::new(
                optic_core::OpticErrorKind::Custom,
                &format!(
                    "upload_canvas: {} color attachments exceeds GL_MAX_COLOR_ATTACHMENTS ({})",
                    desc.color_formats.len(), self.max_color_attachments,
                ),
            ));
        }
        if desc.color_formats.len() as i32 > self.max_draw_buffers {
            return Err(optic_core::OpticError::new(
                optic_core::OpticErrorKind::Custom,
                &format!(
                    "upload_canvas: {} color attachments exceeds GL_MAX_DRAW_BUFFERS ({})",
                    desc.color_formats.len(), self.max_draw_buffers,
                ),
            ));
        }
        if desc.samples as i32 > self.max_samples {
            return Err(optic_core::OpticError::new(
                optic_core::OpticErrorKind::Custom,
                &format!(
                    "upload_canvas: {} samples exceeds GL_MAX_SAMPLES ({})",
                    desc.samples, self.max_samples,
                ),
            ));
        }
        Canvas::new(desc)
    }

    // ── Render target ────────────────────────────────────────────────────

    /// Binds a render target (screen or canvas) for subsequent draw calls.
    ///
    /// Updates the viewport to match the target's size and records the size
    /// in [`current_render_target_size`](Self::current_render_target_size).
    ///
    /// ```ignore
    /// // Render to a canvas
    /// let canvas = gpu.upload_canvas(&my_desc)?;
    /// gpu.set_render_target(&RenderTarget::Canvas(&canvas))?;
    /// gpu.clear();
    ///
    /// // Switch back to screen
    /// gpu.set_render_target(&RenderTarget::Screen)?;
    /// canvas.blit_to_screen(window_size);
    /// ```
    ///
    /// # Errors
    ///
    /// Currently always returns `Ok(())`. This method returns
    /// [`OpticResult`] to preserve a uniform interface for future error
    /// conditions (e.g. invalid FBO state).
    pub fn set_render_target(&mut self, target: &RenderTarget) -> optic_core::OpticResult<()> {
        match target {
            RenderTarget::Screen => {
                unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, 0); }
                GL::resize(self.canvas_size);
                self.current_target_size = self.canvas_size;
            }
            RenderTarget::Canvas(canvas) => {
                let fb = canvas.fbo_id;
                unsafe { gl::BindFramebuffer(gl::FRAMEBUFFER, fb); }
                GL::resize(canvas.size);
                self.current_target_size = canvas.size;
            }
        }
        Ok(())
    }

    // ── Rendering ────────────────────────────────────────────────────────

    /// Draws a 3D mesh through the given camera.
    ///
    /// Internally computes `MVP = proj × view × model` and sends it to the
    /// shader before issuing the draw call.
    ///
    /// ```ignore
    /// let cube_file = Mesh3DFile::cube(2.0);
    /// let mesh = gpu.upload_mesh3d(&cube_file);
    ///
    /// gpu.render3d(&mesh, &camera);
    /// ```
    pub fn render3d(&self, mesh: &Mesh3D, camera: &Camera) {
        mesh.render(&camera.transform.view_matrix(), &camera.transform.proj_matrix());
    }

    /// Draws a 2D mesh using an orthographic projection derived from the
    /// canvas size.
    ///
    /// The projection maps `[-aspect, aspect]` on X and `[-1, 1]` on Y
    /// where `aspect = canvas_width / canvas_height`.
    ///
    /// ```ignore
    /// let quad_file = Mesh2DFile::quad(&(800, 600).into());
    /// let mesh = gpu.upload_mesh2d(&quad_file);
    ///
    /// gpu.render2d(&mesh);
    /// ```
    pub fn render2d(&self, mesh: &Mesh2D) {
        let aspect = if self.canvas_size.w > 0 && self.canvas_size.h > 0 {
            self.canvas_size.w as f32 / self.canvas_size.h as f32
        } else {
            1.0
        };
        let proj = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        mesh.render(&proj);
    }

    /// Draws a [`Text2D`] using the same orthographic projection as
    /// [`render2d`](Self::render2d).
    pub fn render_text2d(&self, text: &mut Text2D) {
        let aspect = if self.canvas_size.w > 0 && self.canvas_size.h > 0 {
            self.canvas_size.w as f32 / self.canvas_size.h as f32
        } else {
            1.0
        };
        let proj = cgmath::ortho(-aspect, aspect, -1.0, 1.0, -1.0, 1.0);
        text.render(&proj);
    }

    /// Draws a [`Text3D`] through the given camera.
    pub fn render_text3d(&self, text: &mut Text3D, camera: &Camera) {
        let view = camera.transform.view_matrix();
        let proj = camera.transform.proj_matrix();
        text.render(&view, &proj);
    }
}
