use optic_core::{OpticError, OpticErrorKind, OpticResult};

use crate::handles::shader::{link_compute_program, link_program, Shader};

pub enum ShaderType {
    Pipeline,
    Compute,
}

enum GLSL {
    ParsedCompute(String),
    ParsedPipeline { v_src: String, f_src: String },
    FailedPipeline { v_missing: bool, _f_missing: bool },
}

impl GLSL {
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

impl ShaderType {
    pub fn is_compute(&self) -> bool {
        matches!(self, ShaderType::Compute)
    }
}

pub struct ShaderAsset {
    pub v_src: String,
    pub f_src: String,
    pub is_compute: bool,
}

impl ShaderAsset {
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

    pub fn from_path(path: &str, typ: ShaderType) -> OpticResult<Self> {
        let src = optic_file::read_string(path)?;
        Self::from_src(&src, typ)
    }

    pub fn from_vert_frag(v_src: &str, f_src: &str) -> Self {
        Self {
            v_src: v_src.to_string(),
            f_src: f_src.to_string(),
            is_compute: false,
        }
    }

    pub fn from_path_cached(path: &str, typ: ShaderType) -> OpticResult<Self> {
        let cached = optic_file::cached_path(path, "oshdr");
        if optic_file::exists(&cached) {
            return Self::from_cached(&cached, typ);
        }
        let shader = Self::from_path(path, typ)?;
        if let Some(parent) = std::path::Path::new(&cached).parent() {
            let _ = optic_file::create_dir(&parent.to_string_lossy());
        }
        shader.save_cached(&cached)?;
        Ok(shader)
    }

    pub fn save_cached(&self, path: &str) -> OpticResult<()> {
        if self.is_compute {
            optic_file::write_string(path, &self.v_src)
        } else {
            let src = format!("// VERTEX\n{}\n// FRAGMENT\n{}", self.v_src.trim_end(), self.f_src.trim_end());
            optic_file::write_string(path, &src)
        }
    }

    pub fn from_cached(path: &str, typ: ShaderType) -> OpticResult<Self> {
        let src = optic_file::read_string(path)?;
        Self::from_src(&src, typ)
    }

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

impl ShaderAsset {
    pub fn default_3d() -> OpticResult<Self> {
        Self::from_path("optic/assets/shdr/fallback3d.glsl", ShaderType::Pipeline)
    }

    pub fn default_2d() -> OpticResult<Self> {
        Self::from_path("optic/assets/shdr/fallback2d.glsl", ShaderType::Pipeline)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_compute_shader() {
        let src = "#version 430\nvoid main() {}";
        let asset = ShaderAsset::from_src(src, ShaderType::Compute).unwrap();
        assert!(asset.is_compute);
        assert_eq!(asset.v_src, src);
        assert!(asset.f_src.is_empty());
    }

    #[test]
    fn parse_pipeline_shader() {
        let src = "// v\nvoid vertex_main() {}\n// f\nvoid fragment_main() {}";
        let asset = ShaderAsset::from_src(src, ShaderType::Pipeline).unwrap();
        assert!(!asset.is_compute);
        assert!(asset.v_src.contains("vertex_main"));
        assert!(asset.f_src.contains("fragment_main"));
    }

    #[test]
    fn parse_pipeline_missing_vertex() {
        let src = "// f\nvoid fragment_main() {}";
        let result = ShaderAsset::from_src(src, ShaderType::Pipeline);
        assert!(result.is_err());
    }

    #[test]
    fn parse_pipeline_missing_fragment() {
        let src = "// v\nvoid vertex_main() {}";
        let result = ShaderAsset::from_src(src, ShaderType::Pipeline);
        assert!(result.is_err());
    }

    #[test]
    fn parse_pipeline_empty_source() {
        let result = ShaderAsset::from_src("", ShaderType::Pipeline);
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
            let asset = ShaderAsset::from_src(src, ShaderType::Pipeline).unwrap();
            assert!(asset.v_src.trim().contains(v_exp));
            assert!(asset.f_src.trim().contains(f_exp));
        }
    }

    #[test]
    fn shader_cached_roundtrip_pipeline() {
        let src = "// VERTEX\nvoid main() {}\n// FRAGMENT\nvoid main() {}";
        let asset = ShaderAsset::from_src(src, ShaderType::Pipeline).unwrap();
        let path = "/tmp/optic_test_shdr_pipe.oshdr";
        asset.save_cached(path).unwrap();
        let loaded = ShaderAsset::from_cached(path, ShaderType::Pipeline).unwrap();
        assert!(!loaded.is_compute);
        assert_eq!(loaded.v_src, asset.v_src);
        assert_eq!(loaded.f_src, asset.f_src);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn shader_cached_roundtrip_compute() {
        let src = "#version 430\nvoid main() {}";
        let asset = ShaderAsset::from_src(src, ShaderType::Compute).unwrap();
        let path = "/tmp/optic_test_shdr_comp.oshdr";
        asset.save_cached(path).unwrap();
        let loaded = ShaderAsset::from_cached(path, ShaderType::Compute).unwrap();
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
        let asset = ShaderAsset::from_vert_frag("v_src", "f_src");
        assert!(!asset.is_compute);
        assert_eq!(asset.v_src, "v_src");
        assert_eq!(asset.f_src, "f_src");
    }
}
