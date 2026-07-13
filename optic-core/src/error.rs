use std::fmt;

/// Broad category of an error.
#[derive(Debug, Clone, PartialEq)]
pub enum OpticErrorKind {
    Init,
    OpenGL,
    Shader,
    Asset,
    File,
    Framebuffer,
    Custom,
}

/// The primary error type for the Optic engine.
///
/// All fallible functions in the engine return [`OpticResult<T>`].
/// Errors propagate upward via `?` and can be handled generically
/// by inspecting [`kind`](OpticError::kind).
///
/// ```
/// use optic_core::*;
///
/// fn load_shader(path: &str) -> OpticResult<()> {
///     Err(OpticError::new(OpticErrorKind::Shader, "compile failed"))
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OpticError {
    pub kind: OpticErrorKind,
    pub msg: String,
}

impl fmt::Display for OpticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "optic error: {}", self.msg)
    }
}

impl OpticError {
    /// Construct an error with a specific kind and message.
    pub fn new(kind: OpticErrorKind, msg: &str) -> Self {
        Self {
            kind,
            msg: msg.to_string(),
        }
    }
    /// Construct a [`Custom`](OpticErrorKind::Custom) error.
    pub fn custom(msg: &str) -> Self {
        OpticError::new(OpticErrorKind::Custom, msg)
    }
    /// Construct a [`Shader`](OpticErrorKind::Shader) error.
    pub fn shader(msg: &str) -> Self {
        OpticError::new(OpticErrorKind::Shader, msg)
    }
    /// Construct an [`Asset`](OpticErrorKind::Asset) error.
    pub fn asset(msg: &str) -> Self {
        OpticError::new(OpticErrorKind::Asset, msg)
    }
    /// Construct a [`File`](OpticErrorKind::File) error.
    pub fn file(msg: &str) -> Self {
        OpticError::new(OpticErrorKind::File, msg)
    }
    /// Construct an [`Init`](OpticErrorKind::Init) error.
    pub fn init(msg: &str) -> Self {
        OpticError::new(OpticErrorKind::Init, msg)
    }
    /// Construct an [`OpenGL`](OpticErrorKind::OpenGL) error.
    pub fn opengl(msg: &str) -> Self {
        OpticError::new(OpticErrorKind::OpenGL, msg)
    }
    /// Construct a [`Framebuffer`](OpticErrorKind::Framebuffer) error.
    pub fn framebuffer(msg: &str) -> Self {
        OpticError::new(OpticErrorKind::Framebuffer, msg)
    }
}

/// Convenience alias for `Result<T, OpticError>`.
pub type OpticResult<T> = Result<T, OpticError>;
