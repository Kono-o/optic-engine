use khronos_egl as egl;
use optic_core::{OpticError, OpticErrorKind, OpticResult, Size2D};
use raw_window_handle::RawWindowHandle;
use std::ffi::c_void;
use std::ptr;

pub struct WindowSurface {
    pub surface: egl::Surface,
    pub size: Size2D,
}

pub struct RenderContext {
    pub display: egl::Display,
    pub context: egl::Context,
    config: egl::Config,
    pub surfaces: Vec<WindowSurface>,
    pub active_index: Option<usize>,
    pub gl_ver: String,
    pub glsl_ver: String,
    pub device: String,
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
        let pbuffer_surface = WindowSurface { surface: pbuffer, size: Size2D::from(1, 1) };

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

    pub fn new_windowed(
        raw_handle: RawWindowHandle,
        size: Size2D,
    ) -> OpticResult<Self> {
        let mut ctx = Self::new_headless()?;
        ctx.attach_window(raw_handle, size)?;
        ctx.make_current(1)?;
        Ok(ctx)
    }

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

    pub fn resize_window(&mut self, index: usize, size: Size2D) {
        if let Some(ws) = self.surfaces.get_mut(index) {
            ws.size = size;
        }
    }

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

    pub fn swap_buffers(&self, index: usize) -> OpticResult<()> {
        let egl_instance = egl::Instance::new(egl::Static);
        let ws = self.surfaces.get(index).ok_or_else(|| {
            OpticError::new(OpticErrorKind::OpenGL, "invalid surface index")
        })?;

        egl_instance.swap_buffers(self.display, ws.surface)
            .map_err(|e| OpticError::new(OpticErrorKind::OpenGL, &format!("swap buffers failed: {e}")))
    }

    pub fn clear(&self) {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT); }
    }

    pub fn set_vsync(&self, enable: bool) {
        let egl_instance = egl::Instance::new(egl::Static);
        let interval = if enable { 1 } else { 0 };
        let _ = egl_instance.swap_interval(self.display, interval);
    }

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
