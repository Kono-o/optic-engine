use khronos_egl as egl;
use optic_core::{OpticError, OpticErrorKind, OpticResult, Size2D};
use raw_window_handle::RawWindowHandle;
use std::ffi::c_void;
use std::ptr;

/// An EGL window surface and its dimensions.
///
/// Created by [`RenderContext::new_windowed`] or [`RenderContext::attach_window`].
/// Each surface corresponds to one native window.
pub struct WindowSurface {
    pub(crate) surface: egl::Surface,
    size: Size2D,
}

impl WindowSurface {
    /// Returns the surface size.
    pub fn size(&self) -> Size2D { self.size }
}

/// EGL + OpenGL 4.6 context with support for multiple window surfaces.
///
/// # Initialisation
///
/// Create a headless context for off-screen rendering:
///
/// ```ignore
/// let ctx = RenderContext::new_headless()?;
/// ```
///
/// Or create a windowed context from a raw window handle:
///
/// ```ignore
/// let ctx = RenderContext::new_windowed(raw_handle, display_handle, size)?;
/// ```
///
/// # Multi-window
///
/// Additional windows can be attached with [`attach_window`](RenderContext::attach_window)
/// and made current with [`make_current`](RenderContext::make_current).
pub struct RenderContext {
    pub(crate) display: egl::Display,
    pub(crate) context: egl::Context,
    config: egl::Config,
    pub(crate) surfaces: Vec<WindowSurface>,
    active_index: Option<usize>,
    gl_ver: String,
    glsl_ver: String,
    device: String,
}

const GL_ATTRIBS: [i32; 7] = [
    egl::CONTEXT_MAJOR_VERSION as i32, 4,
    egl::CONTEXT_MINOR_VERSION as i32, 6,
    egl::CONTEXT_OPENGL_PROFILE_MASK as i32,
    egl::CONTEXT_OPENGL_CORE_PROFILE_BIT as i32,
    egl::NONE as i32,
];

const PBUFFER_ATTRIBS: [i32; 5] = [
    egl::WIDTH as i32, 1,
    egl::HEIGHT as i32, 1,
    egl::NONE as i32,
];

const CONFIG_ATTRIBS: [i32; 15] = [
    egl::SURFACE_TYPE as i32, egl::PBUFFER_BIT as i32 | egl::WINDOW_BIT as i32,
    egl::RENDERABLE_TYPE as i32, egl::OPENGL_BIT as i32,
    egl::RED_SIZE as i32, 8,
    egl::GREEN_SIZE as i32, 8,
    egl::BLUE_SIZE as i32, 8,
    egl::ALPHA_SIZE as i32, 8,
    egl::DEPTH_SIZE as i32, 24,
    egl::NONE as i32,
];

fn create_display_and_context() -> OpticResult<(egl::Instance<egl::Static>, egl::Display, egl::Context, egl::Config)> {
    let egl_instance = egl::Instance::new(egl::Static);

    let display = unsafe {
        egl_instance.get_display(egl::DEFAULT_DISPLAY)
            .ok_or_else(|| OpticError::new(OpticErrorKind::OpenGL, "no EGL display found"))?
    };

    egl_instance.initialize(display)
        .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL init failed: {e}")))?;

    let mut configs = Vec::with_capacity(1);
    egl_instance.choose_config(display, &CONFIG_ATTRIBS, &mut configs)
        .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL config failed: {e}")))?;

    let config = *configs.first()
        .ok_or_else(|| OpticError::new(OpticErrorKind::OpenGL, "no suitable EGL config"))?;

    egl_instance.bind_api(egl::OPENGL_API)
        .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL bind API failed: {e}")))?;

    let context = egl_instance.create_context(display, config, None, &GL_ATTRIBS)
        .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL context creation failed: {e}")))?;

    Ok((egl_instance, display, context, config))
}

fn load_gl_info() -> (String, String, String) {
    let gl_ver = unsafe {
        let ptr = gl::GetString(gl::VERSION);
        if ptr.is_null() {
            return ("unknown".into(), "unknown".into(), "unknown".into());
        }
        std::ffi::CStr::from_ptr(ptr as *const i8)
            .to_string_lossy()
            .to_string()
    };

    let glsl_ver = unsafe {
        let ptr = gl::GetString(gl::SHADING_LANGUAGE_VERSION);
        if ptr.is_null() {
            return (gl_ver, "unknown".into(), "unknown".into());
        }
        std::ffi::CStr::from_ptr(ptr as *const i8)
            .to_string_lossy()
            .to_string()
    };

    let device = unsafe {
        let ptr = gl::GetString(gl::RENDERER);
        if ptr.is_null() {
            return (gl_ver, glsl_ver, "unknown".into());
        }
        format!(
            "OPENGL {}",
            std::ffi::CStr::from_ptr(ptr as *const i8)
                .to_string_lossy()
        )
    };

    (gl_ver, glsl_ver, device)
}

fn raw_handle_to_native(handle: RawWindowHandle) -> OpticResult<*mut c_void> {
    match handle {
        RawWindowHandle::Xlib(h) => Ok(h.window as usize as *mut c_void),
        RawWindowHandle::Xcb(h) => Ok(h.window.get() as usize as *mut c_void),
        RawWindowHandle::Wayland(h) => Ok(h.surface.as_ptr() as *mut c_void),
        RawWindowHandle::Win32(h) => Ok(h.hwnd.get() as *mut c_void),
        _ => Err(OpticError::new(
            OpticErrorKind::OpenGL,
            "unsupported platform for EGL window surface",
        )),
    }
}

impl RenderContext {
    /// Returns the OpenGL version string.
    pub fn gl_ver(&self) -> &str { &self.gl_ver }
    /// Returns the GLSL version string.
    pub fn glsl_ver(&self) -> &str { &self.glsl_ver }
    /// Returns the GPU device name.
    pub fn device(&self) -> &str { &self.device }
    /// Returns the active surface index, if any.
    pub fn active_index(&self) -> Option<usize> { self.active_index }
    /// Returns a reference to the window surfaces.
    pub fn surfaces(&self) -> &Vec<WindowSurface> { &self.surfaces }

    /// Creates a headless EGL context with a 1×1 pbuffer surface.
    ///
    /// This is useful for off-screen rendering or when no window is available.
    /// Loads OpenGL function pointers via EGL and queries driver info.
    pub fn new_headless() -> OpticResult<Self> {
        let (egl_instance, display, context, config) = create_display_and_context()?;

        let pbuffer = egl_instance.create_pbuffer_surface(display, config, &PBUFFER_ATTRIBS)
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("pbuffer creation failed: {e}")))?;

        egl_instance.make_current(display, Some(pbuffer), Some(pbuffer), Some(context))
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("make current failed: {e}")))?;

        gl::load_with(|s| {
            egl_instance.get_proc_address(s)
                .map(|p| p as *const _)
                .unwrap_or(ptr::null())
        });

        let (gl_ver, glsl_ver, device) = load_gl_info();
        let pbuffer_surface = WindowSurface { surface: pbuffer, size: Size2D::new(1, 1) };

        Ok(Self {
            display,
            context,
            config,
            surfaces: vec![pbuffer_surface],
            active_index: Some(0),
            gl_ver,
            glsl_ver,
            device,
        })
    }

    /// Creates a windowed EGL context from raw window and display handles.
    ///
    /// Supports X11 (with optional visual ID matching), Wayland, and Win32
    /// platforms. Falls back to the default EGL display if platform-specific
    /// display creation fails.
    pub fn new_windowed(
        raw_handle: RawWindowHandle,
        display_handle: raw_window_handle::RawDisplayHandle,
        size: Size2D,
    ) -> OpticResult<Self> {
        let egl_instance = egl::Instance::new(egl::Static);

        // Use platform-specific display for better compatibility
        let display: egl::Display = match display_handle {
            raw_window_handle::RawDisplayHandle::Xlib(h) => {
                let platform = 0x31D5; // EGL_PLATFORM_X11_EXT
                let native_display = h.display.map_or(std::ptr::null_mut(), |d| d.as_ptr());
                let r = unsafe {
                    egl_instance.get_platform_display(platform, native_display, &[])
                };
                match r {
                    Ok(d) => d,
                    Err(_) => unsafe {
                        egl_instance.get_display(egl::DEFAULT_DISPLAY)
                            .ok_or_else(|| OpticError::new(OpticErrorKind::OpenGL, "no EGL display found"))?
                    },
                }
            }
            raw_window_handle::RawDisplayHandle::Wayland(h) => {
                let platform = 0x31D6; // EGL_PLATFORM_WAYLAND_EXT
                let native_display = h.display.as_ptr() as *mut c_void;
                let r = unsafe {
                    egl_instance.get_platform_display(platform, native_display, &[])
                };
                match r {
                    Ok(d) => d,
                    Err(_) => unsafe {
                        egl_instance.get_display(egl::DEFAULT_DISPLAY)
                            .ok_or_else(|| OpticError::new(OpticErrorKind::OpenGL, "no EGL display found"))?
                    },
                }
            }
            _ => unsafe {
                egl_instance.get_display(egl::DEFAULT_DISPLAY)
                    .ok_or_else(|| OpticError::new(OpticErrorKind::OpenGL, "no EGL display found"))?
            },
        };

        egl_instance.initialize(display)
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL init failed: {e}")))?;

        let native = raw_handle_to_native(raw_handle)?;

        let visual_id: Option<u32> = match raw_handle {
            RawWindowHandle::Xlib(h) => Some(h.visual_id as u32),
            RawWindowHandle::Xcb(h) => h.visual_id.map(|v| v.get()),
            _ => None,
        };

        let result = if let Some(vid) = visual_id {
            Self::try_create_windowed(&egl_instance, display, native, size, Some(vid))
        } else {
            Self::try_create_windowed(&egl_instance, display, native, size, None)
        };

        match result {
            Ok(ctx) => Ok(ctx),
            Err(_) if visual_id.is_some() => {
                Self::try_create_windowed(&egl_instance, display, native, size, None)
            }
            Err(e) => Err(e),
        }
    }

    fn try_create_windowed(
        egl_instance: &egl::Instance<egl::Static>,
        display: egl::Display,
        native: *mut c_void,
        size: Size2D,
        visual_id: Option<u32>,
    ) -> OpticResult<RenderContext> {
        let mut cfg_attribs = Vec::new();
        cfg_attribs.push(egl::SURFACE_TYPE as i32);
        cfg_attribs.push(egl::WINDOW_BIT as i32);
        cfg_attribs.push(egl::RENDERABLE_TYPE as i32);
        cfg_attribs.push(egl::OPENGL_BIT as i32);
        cfg_attribs.push(egl::RED_SIZE as i32); cfg_attribs.push(8);
        cfg_attribs.push(egl::GREEN_SIZE as i32); cfg_attribs.push(8);
        cfg_attribs.push(egl::BLUE_SIZE as i32); cfg_attribs.push(8);
        cfg_attribs.push(egl::ALPHA_SIZE as i32); cfg_attribs.push(8);
        cfg_attribs.push(egl::DEPTH_SIZE as i32); cfg_attribs.push(24);
        if let Some(vid) = visual_id {
            cfg_attribs.push(egl::NATIVE_VISUAL_ID as i32);
            cfg_attribs.push(vid as i32);
        }
        cfg_attribs.push(egl::NONE as i32);

        let mut configs = Vec::with_capacity(1);
        egl_instance.choose_config(display, &cfg_attribs, &mut configs)
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL config failed: {e}")))?;

        let config = *configs.first()
            .ok_or_else(|| OpticError::new(OpticErrorKind::OpenGL, "no suitable EGL config"))?;

        egl_instance.bind_api(egl::OPENGL_API)
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL bind API failed: {e}")))?;

        let context = egl_instance.create_context(display, config, None, &GL_ATTRIBS)
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("EGL context creation failed: {e}")))?;

        let surface = unsafe { egl_instance.create_window_surface(display, config, native, None)
            .map_err(|e| OpticError::new(
                OpticErrorKind::OpenGL,
                &format!("EGL window surface creation failed: {e}"),
            ))? };

        egl_instance.make_current(display, Some(surface), Some(surface), Some(context))
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("make current failed: {e}")))?;

        gl::load_with(|s| {
            egl_instance.get_proc_address(s)
                .map(|p| p as *const _)
                .unwrap_or(ptr::null())
        });

        let (gl_ver, glsl_ver, device) = load_gl_info();
        let window_surface = WindowSurface { surface, size };

        Ok(Self {
            display,
            context,
            config,
            surfaces: vec![window_surface],
            active_index: Some(0),
            gl_ver,
            glsl_ver,
            device,
        })
    }

    /// Attaches a new window surface to this context.
    ///
    /// Returns the index of the new surface, which can be used with
    /// [`make_current`](RenderContext::make_current).
    pub fn attach_window(
        &mut self,
        raw_handle: RawWindowHandle,
        size: Size2D,
    ) -> OpticResult<usize> {
        let native = raw_handle_to_native(raw_handle)?;
        let egl = egl::Instance::new(egl::Static);

        let surface = unsafe { egl.create_window_surface(
            self.display,
            self.config,
            native,
            None,
        ).map_err(|e| OpticError::new(
            OpticErrorKind::OpenGL,
            &format!("EGL window surface creation failed: {e}"),
        ))? };

        let index = self.surfaces.len();
        self.surfaces.push(WindowSurface { surface, size });
        Ok(index)
    }

    /// Resizes the tracked size for a window surface.
    ///
    /// Does **not** call EGL surface resize — just updates the stored
    /// dimensions used by [`make_current`](RenderContext::make_current)
    /// for the viewport call.
    pub fn resize_window(&mut self, index: usize, size: Size2D) {
        if let Some(ws) = self.surfaces.get_mut(index) {
            ws.size = size;
        }
    }

    /// Makes the given surface current and sets the viewport.
    pub fn make_current(&self, index: usize) -> OpticResult<()> {
        let egl_instance = egl::Instance::new(egl::Static);
        let ws = self.surfaces.get(index).ok_or_else(|| {
            OpticError::new(OpticErrorKind::OpenGL, "invalid surface index")
        })?;

        egl_instance.make_current(self.display, Some(ws.surface), Some(ws.surface), Some(self.context))
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("make current failed: {e}")))?;

        unsafe { gl::Viewport(0, 0, ws.size.w as i32, ws.size.h as i32); }
        Ok(())
    }

    /// Swaps front and back buffers for the given surface (double-buffering).
    pub fn swap_buffers(&self, index: usize) -> OpticResult<()> {
        let egl_instance = egl::Instance::new(egl::Static);
        let ws = self.surfaces.get(index).ok_or_else(|| {
            OpticError::new(OpticErrorKind::OpenGL, "invalid surface index")
        })?;

        egl_instance.swap_buffers(self.display, ws.surface)
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("swap buffers failed: {e}")))
    }

    /// Clears the colour and depth buffers of the currently bound framebuffer.
    pub fn clear(&self) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }
    }

    /// Enables or disables vertical sync (swap interval).
    pub fn set_vsync(&self, enable: bool) {
        let egl_instance = egl::Instance::new(egl::Static);
        let interval = if enable { 1 } else { 0 };
        let _ = egl_instance.swap_interval(self.display, interval);
    }

    /// Sets the clear colour used by [`clear`](RenderContext::clear).
    pub fn set_clear_color(&self, color: optic_core::RGBA) {
        unsafe { gl::ClearColor(color.0, color.1, color.2, color.3); }
    }
}

impl Drop for RenderContext {
    fn drop(&mut self) {
        let egl_instance = egl::Instance::new(egl::Static);
        let _ = egl_instance.make_current(self.display, None, None, None);
        for ws in self.surfaces.drain(..) {
            let _ = egl_instance.destroy_surface(self.display, ws.surface);
        }
        let _ = egl_instance.destroy_context(self.display, self.context);
        let _ = egl_instance.terminate(self.display);
    }
}
