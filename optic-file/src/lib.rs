use optic_core::{OpticError, OpticErrorKind, OpticResult};
use std::fs;
use std::io::ErrorKind;
use std::path::PathBuf;

pub fn name(path: &str) -> Option<String> {
    let path = PathBuf::from(path);
    path.file_stem()
        .map(|n| n.to_string_lossy().to_string())
}

pub fn extension(path: &str) -> Option<String> {
    let path = PathBuf::from(path);
    path.extension()
        .map(|n| n.to_string_lossy().to_string())
}

pub fn exists(path: &str) -> bool {
    PathBuf::from(path).exists()
}

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

pub fn write_string(path: &str, data: &str) -> OpticResult<()> {
    write_bytes(path, data.as_bytes())
}

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

pub fn create_dir(path: &str) -> OpticResult<()> {
    fs::create_dir_all(path).map_err(|e| {
        OpticError::new(
            OpticErrorKind::File,
            &format!("could not create directory {path}: {e}"),
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn tmp_path() -> String {
        static COUNTER: AtomicUsize = AtomicUsize::new(0);
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        format!("/tmp/optic_test_{id}")
    }

    #[test]
    fn name_no_extension() {
        let n = name("hello").unwrap();
        assert_eq!(n, "hello");
    }

    #[test]
    fn name_with_path() {
        let n = name("/some/dir/file.txt").unwrap();
        assert_eq!(n, "file");
    }

    #[test]
    fn name_empty_path() {
        let n = name("");
        assert!(n.is_none() || n.unwrap().is_empty());
    }

    #[test]
    fn extension_basic() {
        let e = extension("data.obj").unwrap();
        assert_eq!(e, "obj");
    }

    #[test]
    fn extension_no_ext() {
        let e = extension("Makefile");
        assert!(e.is_none());
    }

    #[test]
    fn exists_false() {
        assert!(!exists("/tmp/nonexistent_file_xyz123"));
    }

    #[test]
    fn write_and_read_bytes() {
        let path = tmp_path();
        let data = b"hello world";
        write_bytes(&path, data).unwrap();
        let read = read_bytes(&path).unwrap();
        assert_eq!(read, data);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn write_and_read_string() {
        let path = tmp_path();
        let data = "hello world";
        write_string(&path, data).unwrap();
        let read = read_string(&path).unwrap();
        assert_eq!(read, data);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn write_bytes_creates_dir() {
        let path = format!("/tmp/optic_test_dir/subdir/file.txt");
        let data = b"test";
        write_bytes(&path, data).unwrap();
        assert!(exists(&path));
        let read = read_bytes(&path).unwrap();
        assert_eq!(read, data);
        let _ = fs::remove_dir_all("/tmp/optic_test_dir");
    }

    #[test]
    fn read_bytes_not_found() {
        let result = read_bytes("/tmp/nonexistent_file_xyz456");
        assert!(result.is_err());
    }

    #[test]
    fn read_string_not_found() {
        let result = read_string("/tmp/nonexistent_file_xyz789");
        assert!(result.is_err());
    }

    #[test]
    fn create_dir_twice() {
        let path = format!("/tmp/optic_test_createdir");
        create_dir(&path).unwrap();
        create_dir(&path).unwrap(); // should be idempotent
        assert!(exists(&path));
        let _ = fs::remove_dir(&path);
    }

    #[test]
    fn cached_path_basic() {
        let c = cached_path("assets/tex/foo.png", "otxtr");
        assert_eq!(c, "assets/tex/optc/foo.otxtr");
    }

    #[test]
    fn cached_path_no_dir() {
        let c = cached_path("foo.png", "otxtr");
        assert_eq!(c, "optc/foo.otxtr");
    }

    #[test]
    fn cached_path_omesh() {
        let c = cached_path("models/cube.obj", "omesh");
        assert_eq!(c, "models/optc/cube.omesh");
    }
}
