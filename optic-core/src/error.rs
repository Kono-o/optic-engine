use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum OpticErrorKind {
    Init,
    OpenGL,
    Shader,
    Asset,
    File,
    Custom,
}

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
    pub fn new(kind: OpticErrorKind, msg: &str) -> Self {
        Self {
            kind,
            msg: msg.to_string(),
        }
    }
    pub fn custom(msg: &str) -> Self {
        Self {
            kind: OpticErrorKind::Custom,
            msg: msg.to_string(),
        }
    }
}

pub type OpticResult<T> = Result<T, OpticError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_new() {
        let e = OpticError::new(OpticErrorKind::File, "test error");
        assert_eq!(e.kind, OpticErrorKind::File);
        assert_eq!(e.msg, "test error");
    }

    #[test]
    fn error_custom() {
        let e = OpticError::custom("custom msg");
        assert_eq!(e.kind, OpticErrorKind::Custom);
        assert_eq!(e.msg, "custom msg");
    }

    #[test]
    fn error_display() {
        let e = OpticError::new(OpticErrorKind::Shader, "compile failed");
        let s = format!("{e}");
        assert!(s.contains("optic error"));
        assert!(s.contains("compile failed"));
    }

    #[test]
    fn error_is_send() {
        fn assert_send<T: Send>() {}
        assert_send::<OpticError>();
    }
}
