//! Sanitary file I/O and cached-path resolution for the Optic engine.
//!
//! All fallible functions return [`OpticResult`] wrapping [`OpticErrorKind::File`]
//! errors with descriptive messages.
//!
//! # Cache path convention
//!
//! Assets are cached alongside the source file in an `optc/` subdirectory:
//!
//! ```ignore
//! assets/tex/foo.png     → assets/tex/optc/foo.otxtr
//! models/cube.obj        → models/optc/cube.omesh
//! shaders/main.glsl      → shaders/optc/main.oshdr
//! ```

use optic_core::{OpticError, OpticErrorKind, OpticResult};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

/// Extract the file stem (name without extension) from a path.
///
/// ```
/// use optic_file::name;
/// assert_eq!(name("foo.txt"), Some("foo".into()));
/// assert_eq!(name("/path/to/bar.txt"), Some("bar".into()));
/// ```
pub fn name(path: &str) -> Option<String> {
    let path = PathBuf::from(path);
    path.file_stem()
        .map(|n| n.to_string_lossy().to_string())
}

/// Extract the file extension from a path.
///
/// ```
/// use optic_file::extension;
/// assert_eq!(extension("foo.txt"), Some("txt".into()));
/// assert_eq!(extension("Makefile"), None);
/// ```
pub fn extension(path: &str) -> Option<String> {
    let path = PathBuf::from(path);
    path.extension()
        .map(|n| n.to_string_lossy().to_string())
}

/// Check whether a file or directory exists at the given path.
pub fn exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

/// Read a file as raw bytes.
///
/// # Errors
///
/// Returns [`OpticErrorKind::File`] if the file is missing, unreadable, or
/// permission is denied.
pub fn read_bytes(path: &str) -> OpticResult<Vec<u8>> {
    match fs::read(path) {
        Ok(data) => Ok(data),
        Err(e) => {
            let kind = match e.kind() {
                ErrorKind::NotFound | ErrorKind::InvalidInput => "file not found or invalid",
                ErrorKind::PermissionDenied => "permission denied",
                _ => "unknown file error",
            };
            Err(OpticError::new(
                OpticErrorKind::File,
                &format!("{kind}: {path}"),
            ))
        }
    }
}

/// Read a file as a UTF-8 string.
///
/// # Errors
///
/// Returns [`OpticErrorKind::File`] if the file does not exist, is not
/// valid UTF-8, or permission is denied.
pub fn read_string(path: &str) -> OpticResult<String> {
    match fs::read_to_string(path) {
        Ok(data) => Ok(data),
        Err(e) => {
            let kind = match e.kind() {
                ErrorKind::NotFound | ErrorKind::InvalidInput => "file not found or invalid",
                ErrorKind::PermissionDenied => "permission denied",
                _ => "unknown file error",
            };
            Err(OpticError::new(
                OpticErrorKind::File,
                &format!("{kind}: {path}"),
            ))
        }
    }
}

/// Write raw bytes to a file, creating parent directories if needed.
///
/// # Errors
///
/// Returns [`OpticErrorKind::File`] if the directory cannot be created or
/// the file cannot be written.
pub fn write_bytes(path: &str, data: &[u8]) -> OpticResult<()> {
    let pathbuf = PathBuf::from(path);
    if let Some(parent) = pathbuf.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| {
                OpticError::new(
                    OpticErrorKind::File,
                    &format!("could not create directory {}: {e}", parent.display()),
                )
            })?;
        }
    }
    fs::write(path, data).map_err(|e| {
        OpticError::new(
            OpticErrorKind::File,
            &format!("could not write {path}: {e}"),
        )
    })
}

/// Write a UTF-8 string to a file, creating parent directories if needed.
///
/// Equivalent to [`write_bytes`] with `data.as_bytes()`.
pub fn write_string(path: &str, data: &str) -> OpticResult<()> {
    write_bytes(path, data.as_bytes())
}

/// Compute the cache path for a source asset.
///
/// The cache file is placed in an `optc/` subdirectory next to the source
/// file, with the given extension replacing the original:
///
/// ```
/// use optic_file::cached_path;
///
/// assert_eq!(cached_path("assets/tex/foo.png", "otxtr"),
///            "assets/tex/optc/foo.otxtr");
/// assert_eq!(cached_path("foo.png", "omesh"),
///            "optc/foo.omesh");
/// ```
pub fn cached_path(source: &str, ext: &str) -> String {
    let pb = PathBuf::from(source);
    let parent = pb.parent().and_then(|p| {
        let s = p.to_string_lossy().to_string();
        if s.is_empty() || s == "." { None } else { Some(s) }
    });
    let stem = pb.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
    match parent {
        Some(dir) => format!("{dir}/optc/{stem}.{ext}"),
        None => format!("optc/{stem}.{ext}"),
    }
}

/// Create a directory and all parent directories (like `mkdir -p`).
pub fn create_dir(path: &str) -> OpticResult<()> {
    fs::create_dir_all(path).map_err(|e| {
        OpticError::new(
            OpticErrorKind::File,
            &format!("could not create directory {path}: {e}"),
        )
    })
}
