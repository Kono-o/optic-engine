//! Error types for the Optic engine.
//!
//! Every fallible operation in Optic returns [`OpticResult<T>`], which is
//! a type alias for `Result<T, OpticError>`. Errors carry a [`OpticErrorKind`]
//! category and a human-readable message. Use [`OpticError::kind`] to match
//! on the category, or handle them generically with `?` propagation.
//!
//! The quickest way to create an error is through one of the kind-specific
//! constructors:
//!
//! ```ignore
//! let err = OpticError::shader("failed to compile fragment shader");
//! let err = OpticError::new(OpticErrorKind::Asset, "texture not found");
//! ```

use std::fmt;

/// Broad category of an error.
///
/// Used with [`OpticError`] to classify failures into one of several
/// well-known buckets. This allows callers to handle errors by category
/// (e.g. retry on [`File`](OpticErrorKind::File), abort on
/// [`Init`](OpticErrorKind::Init)) without parsing messages.
#[derive(Debug, Clone, PartialEq)]
pub enum OpticErrorKind {
    /// Engine or subsystem initialization failed (e.g. window creation,
    /// OpenGL context setup, driver incompatibility).
    Init,
    /// An OpenGL API call returned an error, or the GL context is in an
    /// invalid state.
    OpenGL,
    /// Shader compilation or linking failed. The accompanying message
    /// typically contains the driver's error log.
    Shader,
    /// An asset (texture, mesh, model, font, …) could not be loaded or
    /// parsed.
    Asset,
    /// A file-system operation failed (open, read, write, path resolution).
    File,
    /// Framebuffer creation, attachment, or completeness check failed.
    Framebuffer,
    /// A catch-all for errors that do not fit the other categories.
    Custom,
}

/// The primary error type for the Optic engine.
///
/// All fallible functions in the engine return [`OpticResult<T>`].
/// Errors propagate upward via `?` and can be handled generically
/// by inspecting [`kind`](OpticError::kind).
///
/// ```ignore
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
    /// Construct an error with a specific [`OpticErrorKind`] and message.
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
