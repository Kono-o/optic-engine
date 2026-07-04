use optic_core::{log_info, ColorInfo, Cull, DrawMode, Gradient, PolyMode, RGBA, Size2D};

use crate::context::RenderContext;
use crate::glraw::GL;
use crate::handles::{Canvas, Mesh2D, Mesh3D, RenderTarget, Shader, Texture2D};
use crate::util::{Transform2D, Transform3D};
use crate::{asset, Camera};

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
/// fallback texture. [`ship_mesh3d`](Self::ship_mesh3d) and
/// [`ship_mesh2d`](Self::ship_mesh2d) attach these automatically, so you can
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
    pub(crate) current_target_size: Size2D,
    pub(crate) max_color_attachments: i32,
    pub(crate) max_draw_buffers: i32,
    pub(crate) max_samples: i32,
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
    /// (e.g. `winit`). See [`optic_window`] for how to acquire them.
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

        let canvas_size = Size2D::from(1, 1);
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
            canvas_size,
            current_target_size: canvas_size,
            max_color_attachments,
            max_draw_buffers,
            max_samples,
        };

        if let Ok(fallback_tex) = asset::TextureFile::fallback() {
            let mut tex = fallback_tex;
            tex.set_wrap(optic_core::ImgWrap::Repeat);
            gpu.fallback_texture = gpu.ship_texture(&tex);
        }
        if let Ok(shader_asset) = asset::ShaderFile::default_3d() {
            if let Some(shader) = gpu.ship_shader(&shader_asset) {
                gpu.fallback_shader3d = shader;
                let mut s = gpu.fallback_shader3d.clone();
                s.attach_tex(&gpu.fallback_texture);
                gpu.fallback_shader3d = s;
            }
        }
        if let Ok(shader_asset) = asset::ShaderFile::default_2d() {
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

    // ── Backend info ─────────────────────────────────────────────────────

    /// Returns the OpenGL version string, e.g. `"4.6.0 NVIDIA 545.84"`.
    pub fn version(&self) -> &str { &self.ctx.gl_ver }

    /// Returns the GLSL version string, e.g. `"4.60"`.
    pub fn lang_version(&self) -> &str { &self.ctx.glsl_ver }

    /// Returns the GPU device name, e.g. `"GeForce RTX 3080"`.
    pub fn name(&self) -> &str { &self.ctx.device }

    /// Logs the OpenGL and GLSL version at the `INFO` level.
    pub fn log_backend_info(&self) {
        log_info!("BACKEND: {} (GLSL {})", self.ctx.gl_ver, self.ctx.glsl_ver);
    }

    /// Logs the current pipeline configuration (polygon mode, culling, MSAA)
    /// at the `INFO` level.
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
    /// | [`PolyMode::Filled`] | Solid triangles (default) |
    /// | [`PolyMode::WireFrame`] | Triangle outlines |
    /// | [`PolyMode::Points`] | Vertex points |
    pub fn set_poly_mode(&mut self, mode: PolyMode) {
        self.poly_mode = mode;
        GL::poly_mode(mode);
    }

    /// Toggles between filled and wireframe mode.
    pub fn toggle_wireframe(&mut self) {
        let mode = match self.poly_mode {
            PolyMode::WireFrame => PolyMode::Filled,
            _ => PolyMode::WireFrame,
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

    /// Sets the point size used when [`PolyMode::Points`] is active.
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
    /// is available at [`GPU::max_samples`] at runtime.
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
    /// front-facing. Set to [`Cull::Clock`] to cull the front face instead.
    pub fn set_cull_face(&mut self, cull_face: Cull) {
        self.cull_face = cull_face;
        GL::set_cull_face(cull_face);
    }

    /// Flips the cull face between clockwise and counter-clockwise.
    pub fn flip_cull_face(&mut self) {
        self.cull_face = match self.cull_face {
            Cull::Clock => Cull::AntiClock,
            Cull::AntiClock => Cull::Clock,
        };
        GL::set_cull_face(self.cull_face);
    }

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

    // ── Asset shipping ───────────────────────────────────────────────────

    /// Returns a clone of the 3D fallback shader (with the fallback texture
    /// pre-bound).
    ///
    /// Useful when you want to render a mesh without writing a custom shader:
    ///
    /// ```ignore
    /// let mut mesh = gpu.ship_mesh3d(&my_mesh_file);
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
    /// let mesh = gpu.ship_mesh3d(&cube);
    /// gpu.render3d(&mesh, &camera);
    /// ```
    pub fn ship_mesh3d(&self, file: &asset::Mesh3DFile) -> Mesh3D {
        Mesh3D {
            visibility: true,
            handle: file.ship(),
            shader: Some(self.fallback_shader3d()),
            transform: Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        }
    }

    /// Uploads a [`Mesh2DFile`](asset::Mesh2DFile) to the GPU and returns a
    /// [`Mesh2D`] with the fallback 2D shader attached.
    pub fn ship_mesh2d(&self, file: &asset::Mesh2DFile) -> Mesh2D {
        Mesh2D {
            visibility: true,
            handle: file.ship(),
            shader: Some(self.fallback_shader2d()),
            transform: Transform2D::default(),
            draw_mode: DrawMode::Triangles,
        }
    }

    /// Compiles a [`ShaderFile`](asset::ShaderFile) into a usable [`Shader`].
    ///
    /// Returns `None` if compilation or linking fails (errors are logged
    /// internally by the GLSL compiler). A returned shader is ready to bind
    /// uniforms and attach textures.
    pub fn ship_shader(&self, asset: &asset::ShaderFile) -> Option<Shader> {
        asset.compile().ok()
    }

    /// Uploads a [`TextureFile`](asset::TextureFile) to the GPU.
    ///
    /// ```ignore
    /// let tex_file = TextureFile::from_disk("assets/grass.png")?;
    /// let tex: Texture2D = gpu.ship_texture(&tex_file);
    /// ```
    pub fn ship_texture(&self, image: &asset::TextureFile) -> Texture2D {
        image.ship()
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
    /// let ramp: Texture2D = gpu.ship_gradient(&grad, 256);
    /// // Use as a lookup texture in a shader
    /// ```
    pub fn ship_gradient(&self, gradient: &Gradient, resolution: u32) -> Texture2D {
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
        let size = Size2D::from(res, 1);
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
    pub fn ship_canvas(&mut self, desc: &crate::handles::CanvasDesc) -> optic_core::OpticResult<Canvas> {
        if desc.color_formats.len() as i32 > self.max_color_attachments {
            return Err(optic_core::OpticError::new(
                optic_core::OpticErrorKind::Custom,
                &format!(
                    "ship_canvas: {} color attachments exceeds GL_MAX_COLOR_ATTACHMENTS ({})",
                    desc.color_formats.len(), self.max_color_attachments,
                ),
            ));
        }
        if desc.color_formats.len() as i32 > self.max_draw_buffers {
            return Err(optic_core::OpticError::new(
                optic_core::OpticErrorKind::Custom,
                &format!(
                    "ship_canvas: {} color attachments exceeds GL_MAX_DRAW_BUFFERS ({})",
                    desc.color_formats.len(), self.max_draw_buffers,
                ),
            ));
        }
        if desc.samples as i32 > self.max_samples {
            return Err(optic_core::OpticError::new(
                optic_core::OpticErrorKind::Custom,
                &format!(
                    "ship_canvas: {} samples exceeds GL_MAX_SAMPLES ({})",
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
    /// in [`current_target_size`](Self::current_target_size).
    ///
    /// ```ignore
    /// // Render to a canvas
    /// let canvas = gpu.ship_canvas(&my_desc)?;
    /// gpu.set_render_target(&RenderTarget::Canvas(&canvas))?;
    /// gpu.clear();
    ///
    /// // Switch back to screen
    /// gpu.set_render_target(&RenderTarget::Screen)?;
    /// canvas.blit_to_screen(window_size);
    /// ```
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
    /// let mesh = gpu.ship_mesh3d(&cube_file);
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
    /// let mesh = gpu.ship_mesh2d(&quad_file);
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
}
