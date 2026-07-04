use optic_core::consts::{OPTIC_CACHE_VERSION, OPTIC_MAGIC, SHADER_COMPUTE, SHADER_PIPELINE};
use optic_core::{OpticError, OpticErrorKind, OpticResult};

use crate::handles::shader::{link_compute_program, link_program, Shader};

/// Whether a shader source is a vertex+fragment pipeline or a compute shader.
pub enum ShaderType {
    /// Vertex + fragment shader pair.
    Pipeline,
    /// Compute shader.
    Compute,
}

impl ShaderType {
    /// Returns `true` for the compute variant.
    pub fn is_compute(&self) -> bool {
        matches!(self, ShaderType::Compute)
    }
}

/// Internal GLSL parsing result.
enum GLSL {
    ParsedCompute(String),
    ParsedPipeline { v_src: String, f_src: String },
    FailedPipeline { v_missing: bool, _f_missing: bool },
}

impl GLSL {
    /// Parses a combined GLSL source into vertex/fragment or compute.
    ///
    /// Pipeline shaders use comment markers to delimit sections:
    /// - `//V`, `//VERT`, `//vertex`, etc. start the vertex section
    /// - `//F`, `//FRAG`, `//fragment`, etc. start the fragment section
    ///
    /// Compute shaders use the entire source as one stage.
    fn parse(src: &str, typ: &ShaderType) -> Self {
        if typ.is_compute() {
            return GLSL::ParsedCompute(src.to_string());
        }

        let mut v_src = String::new();
        let mut f_src = String::new();
        let mut v_found = false;
        let mut f_found = false;
        let mut cur = &mut v_src;

        for line in src.lines() {
            let line = line.trim();
            match line {
                "//v" | "//V" | "//vert" | "//VERT" | "//vertex" | "//VERTEX"
                | "// v" | "// V" | "// vert" | "// VERT" | "// vertex" | "// VERTEX" => {
                    cur = &mut v_src;
                    v_found = true;
                }
                "//f" | "//F" | "//frag" | "//FRAG" | "//fragment" | "//FRAGMENT"
                | "// f" | "// F" | "// frag" | "// FRAG" | "// fragment" | "// FRAGMENT" => {
                    cur = &mut f_src;
                    f_found = true;
                }
                _ => {
                    cur.push_str(line);
                    cur.push('\n');
                }
            }
        }

        let v_missing = v_src.is_empty() || !v_found;
        let f_missing = f_src.is_empty() || !f_found;

        if v_missing || f_missing {
            GLSL::FailedPipeline { v_missing, _f_missing: f_missing }
        } else {
            GLSL::ParsedPipeline { v_src, f_src }
        }
    }
}

/// A shader loaded from disk (or cache), ready to compile.
///
/// # Loading
///
/// ```ignore
/// use optic_render::asset::{ShaderFile, ShaderType};
///
/// let sf = ShaderFile::from_disk("shaders/example.glsl", ShaderType::Pipeline)?;
/// let shader = sf.compile()?; // returns a Shader handle
/// ```
///
/// # Shader format
///
/// Pipeline shaders use comment markers to separate vertex and fragment stages
/// within a single `.glsl` file:
///
/// ```glsl
/// // V
/// void main() {
///     gl_Position = vec4(0.0);
/// }
/// // F
/// void main() {
///     outColor = vec4(1.0);
/// }
/// ```
pub struct ShaderFile {
    pub v_src: String,
    pub f_src: String,
    pub is_compute: bool,
}

impl ShaderFile {
    /// Creates a `ShaderFile` by parsing a combined GLSL source string.
    pub fn from_src(src: &str, typ: ShaderType) -> OpticResult<Self> {
        match GLSL::parse(src, &typ) {
            GLSL::ParsedCompute(src) => Ok(Self {
                v_src: src.clone(),
                f_src: String::new(),
                is_compute: true,
            }),
            GLSL::ParsedPipeline { v_src, f_src } => Ok(Self {
                v_src,
                f_src,
                is_compute: false,
            }),
            GLSL::FailedPipeline { v_missing, _f_missing: _ } => {
                if v_missing {
                    Err(OpticError::new(OpticErrorKind::Shader, "vertex shader section missing"))
                } else {
                    Err(OpticError::new(OpticErrorKind::Shader, "fragment shader section missing"))
                }
            }
        }
    }

    /// Creates a pipeline shader from separate vertex and fragment source strings.
    pub fn from_vert_frag(v_src: &str, f_src: &str) -> Self {
        Self {
            v_src: v_src.to_string(),
            f_src: f_src.to_string(),
            is_compute: false,
        }
    }

    /// Compiles this shader and returns a [`Shader`](crate::handles::Shader) handle.
    pub fn compile(&self) -> OpticResult<Shader> {
        if self.is_compute {
            let id = link_compute_program(&self.v_src)?;
            Ok(Shader::new(id, true))
        } else {
            let id = link_program(&self.v_src, &self.f_src)?;
            Ok(Shader::new(id, false))
        }
    }
}

// --- from_disk: debug loads source + overwrites cache; release loads cache only ---
#[cfg(debug_assertions)]
impl ShaderFile {
    /// Loads a shader from disk, caching it for release builds.
    pub fn from_disk(path: &str, typ: ShaderType) -> OpticResult<Self> {
        let src = optic_file::read_string(path)?;
        let shader = Self::from_src(&src, typ)?;
        let cache = optic_file::cached_path(path, "oshdr");
        shader.save_cached(&cache)?;
        Ok(shader)
    }
}

#[cfg(not(debug_assertions))]
impl ShaderFile {
    /// Loads a shader from the binary cache (release only).
    pub fn from_disk(path: &str, _typ: ShaderType) -> OpticResult<Self> {
        let cache = optic_file::cached_path(path, "oshdr");
        Self::from_cached(&cache)
    }
}

// --- binary cache read/write (internal) ---
impl ShaderFile {
    /// Saves this shader to a binary cache file.
    pub fn save_cached(&self, path: &str) -> OpticResult<()> {
        let typ_byte = if self.is_compute { SHADER_COMPUTE } else { SHADER_PIPELINE };
        let v_bytes = self.v_src.as_bytes();
        let f_bytes = self.f_src.as_bytes();
        let mut data = Vec::with_capacity(13 + v_bytes.len() + f_bytes.len());
        data.extend_from_slice(&OPTIC_MAGIC);
        data.extend_from_slice(&OPTIC_CACHE_VERSION.to_le_bytes());
        data.push(typ_byte);
        data.extend_from_slice(&(v_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(v_bytes);
        data.extend_from_slice(&(f_bytes.len() as u32).to_le_bytes());
        data.extend_from_slice(f_bytes);
        optic_file::write_bytes(path, &data)
    }

    /// Loads a shader from a binary cache file.
    #[cfg_attr(debug_assertions, allow(dead_code))]
    fn from_cached(path: &str) -> OpticResult<Self> {
        let data = optic_file::read_bytes(path)?;
        if data.len() < 15 {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("cached shader too short: {path}")));
        }
        if data[0..8] != OPTIC_MAGIC {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("not a valid Optic cache file (bad magic): {path}")));
        }
        let version = u16::from_le_bytes([data[8], data[9]]);
        if version != OPTIC_CACHE_VERSION {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!(
                "cache file version {version} is not supported (expected {OPTIC_CACHE_VERSION}): {path}"
            )));
        }
        let is_compute = data[10] == SHADER_COMPUTE;

        let mut off = 11usize;
        let v_len = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        off += 4;
        if off + v_len > data.len() {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached shader (vertex section): {path}")));
        }
        let v_src = String::from_utf8(data[off..off + v_len].to_vec())
            .map_err(|_| OpticError::new(OpticErrorKind::Asset, &format!("invalid UTF-8 in cached shader: {path}")))?;
        off += v_len;

        if off + 4 > data.len() {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached shader (fragment length): {path}")));
        }
        let f_len = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        off += 4;
        if off + f_len > data.len() {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached shader (fragment section): {path}")));
        }
        let f_src = if f_len > 0 {
            String::from_utf8(data[off..off + f_len].to_vec())
                .map_err(|_| OpticError::new(OpticErrorKind::Asset, &format!("invalid UTF-8 in cached shader: {path}")))?
        } else {
            String::new()
        };

        Ok(Self { v_src, f_src, is_compute })
    }
}

impl ShaderFile {
    /// Loads the default 3D pipeline shader from `optic/assets/shdr/fallback3d.glsl`.
    pub fn default_3d() -> OpticResult<Self> {
        Self::from_disk("optic/assets/shdr/fallback3d.glsl", ShaderType::Pipeline)
    }

    /// Loads the default 2D pipeline shader from `optic/assets/shdr/fallback2d.glsl`.
    pub fn default_2d() -> OpticResult<Self> {
        Self::from_disk("optic/assets/shdr/fallback2d.glsl", ShaderType::Pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_compute_shader() {
        let src = "#version 430\nvoid main() {}";
        let asset = ShaderFile::from_src(src, ShaderType::Compute).unwrap();
        assert!(asset.is_compute);
        assert_eq!(asset.v_src, src);
        assert!(asset.f_src.is_empty());
    }

    #[test]
    fn parse_pipeline_shader() {
        let src = "// v\nvoid vertex_main() {}\n// f\nvoid fragment_main() {}";
        let asset = ShaderFile::from_src(src, ShaderType::Pipeline).unwrap();
        assert!(!asset.is_compute);
        assert!(asset.v_src.contains("vertex_main"));
        assert!(asset.f_src.contains("fragment_main"));
    }

    #[test]
    fn parse_pipeline_missing_vertex() {
        let src = "// f\nvoid fragment_main() {}";
        let result = ShaderFile::from_src(src, ShaderType::Pipeline);
        assert!(result.is_err());
    }

    #[test]
    fn parse_pipeline_missing_fragment() {
        let src = "// v\nvoid vertex_main() {}";
        let result = ShaderFile::from_src(src, ShaderType::Pipeline);
        assert!(result.is_err());
    }

    #[test]
    fn parse_pipeline_empty_source() {
        let result = ShaderFile::from_src("", ShaderType::Pipeline);
        assert!(result.is_err());
    }

    #[test]
    fn parse_pipeline_various_markers() {
        let cases = vec![
            ("//VERT\nv\n//FRAG\nf", "v", "f"),
            ("//vertex\nv\n//fragment\nf", "v", "f"),
            ("// V\nv\n// F\nf", "v", "f"),
        ];
        for (src, v_exp, f_exp) in cases {
            let asset = ShaderFile::from_src(src, ShaderType::Pipeline).unwrap();
            assert!(asset.v_src.trim().contains(v_exp));
            assert!(asset.f_src.trim().contains(f_exp));
        }
    }

    #[test]
    fn shader_cached_roundtrip_pipeline() {
        let src = "// VERTEX\nvoid main() {}\n// FRAGMENT\nvoid main() {}";
        let asset = ShaderFile::from_src(src, ShaderType::Pipeline).unwrap();
        let path = "/tmp/optic_test_shdr_pipe.oshdr";
        asset.save_cached(path).unwrap();
        let loaded = ShaderFile::from_cached(path).unwrap();
        assert!(!loaded.is_compute);
        assert_eq!(loaded.v_src, asset.v_src);
        assert_eq!(loaded.f_src, asset.f_src);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn shader_cached_roundtrip_compute() {
        let src = "#version 430\nvoid main() {}";
        let asset = ShaderFile::from_src(src, ShaderType::Compute).unwrap();
        let path = "/tmp/optic_test_shdr_comp.oshdr";
        asset.save_cached(path).unwrap();
        let loaded = ShaderFile::from_cached(path).unwrap();
        assert!(loaded.is_compute);
        assert_eq!(loaded.v_src, src);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn shader_type_is_compute() {
        assert!(ShaderType::Compute.is_compute());
        assert!(!ShaderType::Pipeline.is_compute());
    }

    #[test]
    fn from_vert_frag() {
        let asset = ShaderFile::from_vert_frag("v_src", "f_src");
        assert!(!asset.is_compute);
        assert_eq!(asset.v_src, "v_src");
        assert_eq!(asset.f_src, "f_src");
    }
}
