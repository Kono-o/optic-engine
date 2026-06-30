use optic_core::OpticError;
use optic_core::{OpticErrorKind, OpticResult};
use cgmath::{Matrix, Matrix2, Matrix3, Matrix4, Vector2, Vector3, Vector4};
use gl::types::GLint;
use std::ffi::CString;
use std::ptr;

use crate::handles::{StorageBuffer, Texture2D};
use crate::GL;

#[derive(Clone, Debug)]
pub enum Slot {
    S0, S1, S2, S3, S4, S5, S6, S7,
    S8, S9, S10, S11, S12, S13, S14, S15,
}

impl Slot {
    pub fn as_index(&self) -> usize {
        match self {
            Slot::S0 => 0, Slot::S1 => 1, Slot::S2 => 2, Slot::S3 => 3,
            Slot::S4 => 4, Slot::S5 => 5, Slot::S6 => 6, Slot::S7 => 7,
            Slot::S8 => 8, Slot::S9 => 9, Slot::S10 => 10, Slot::S11 => 11,
            Slot::S12 => 12, Slot::S13 => 13, Slot::S14 => 14, Slot::S15 => 15,
        }
    }
    pub fn total_slots() -> usize { 16 }
}

#[derive(Clone, Debug)]
pub struct Workers {
    pub group_x: u32,
    pub group_y: u32,
    pub group_z: u32,
}

impl Workers {
    pub fn empty() -> Self { Self { group_x: 0, group_y: 0, group_z: 0 } }
    pub fn set_groups(&mut self, x: u32, y: u32, z: u32) {
        self.group_x = x; self.group_y = y; self.group_z = z;
    }
    pub fn groups(&self) -> (u32, u32, u32) { (self.group_x, self.group_y, self.group_z) }
    pub fn set_group_x(&mut self, x: u32) { self.group_x = x; }
    pub fn set_group_y(&mut self, y: u32) { self.group_y = y; }
    pub fn set_group_z(&mut self, z: u32) { self.group_z = z; }
}

#[derive(Clone, Debug)]
pub struct Shader {
    pub workers: Workers,
    pub id: u32,
    pub is_compute: bool,
    pub tex_ids: Vec<Option<u32>>,
    pub sbo_ids: Vec<Option<u32>>,
}

impl Shader {
    pub fn new(id: u32, is_compute: bool) -> Self {
        Self {
            workers: Workers::empty(),
            id,
            is_compute,
            tex_ids: vec![None; Slot::total_slots()],
            sbo_ids: vec![None; Slot::total_slots()],
        }
    }

    pub fn set_tex_at_slot(&mut self, tex: &Texture2D, slot: Slot) {
        self.tex_ids[slot.as_index()] = Some(tex.id);
    }

    pub fn set_sbo_at_slot(&mut self, sbo: &StorageBuffer, slot: Slot) {
        self.sbo_ids[slot.as_index()] = Some(sbo.id);
    }

    pub fn delete(self) { delete_program(self.id); }

    pub fn bind(&self) { unsafe { gl::UseProgram(self.id); } }
    pub fn unbind(&self) { unsafe { gl::UseProgram(0); } }

    pub fn compute(&self) {
        self.bind();
        self.bind_textures();
        self.bind_storages();
        let (x, y, z) = self.workers.groups();
        unsafe {
            gl::DispatchCompute(x, y, z);
            gl::MemoryBarrier(
                gl::SHADER_IMAGE_ACCESS_BARRIER_BIT | gl::SHADER_STORAGE_BARRIER_BIT,
            );
        }
    }

    pub fn uniform_location(&self, name: &str) -> Option<u32> {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let loc = gl::GetUniformLocation(self.id, c_name.as_ptr());
            if loc == -1 { None } else { Some(loc as u32) }
        }
    }

    fn uni_loc(&self, name: &str) -> GLint {
        unsafe {
            let c_name = CString::new(name).unwrap();
            let loc = gl::GetUniformLocation(self.id, c_name.as_ptr());
            if loc == -1 {
                panic!("uniform '{name}' does not exist in shader {}", self.id);
            }
            loc
        }
    }

    pub fn texture_binds(&self) -> Vec<(u32, u32)> {
        self.tex_ids.iter().enumerate()
            .filter_map(|(slot, id)| id.map(|tid| (slot as u32, tid)))
            .collect()
    }
    pub fn storage_binds(&self) -> Vec<(u32, u32)> {
        self.sbo_ids.iter().enumerate()
            .filter_map(|(slot, id)| id.map(|sid| (slot as u32, sid)))
            .collect()
    }
    pub fn bind_textures(&self) {
        for (slot, tex_id) in self.tex_ids.iter().enumerate() {
            if let Some(id) = tex_id {
                if self.is_compute {
                    unsafe {
                        gl::BindImageTexture(
                            slot as u32, *id, 0, gl::FALSE, 0, gl::READ_WRITE, gl::RGBA8,
                        );
                    }
                } else {
                    GL::bind_texture_at(*id, slot as u32);
                }
            }
        }
    }

    pub fn bind_storages(&self) {
        for (slot, sbo_id) in self.sbo_ids.iter().enumerate() {
            if let Some(id) = sbo_id {
                unsafe {
                    gl::BindBufferBase(gl::SHADER_STORAGE_BUFFER, slot as u32, *id);
                }
            }
        }
    }

    pub fn set_i32(&self, name: &str, v: i32) {
        unsafe { gl::Uniform1i(self.uni_loc(name), v); }
    }
    pub fn set_u32(&self, name: &str, v: u32) {
        unsafe { gl::Uniform1ui(self.uni_loc(name), v); }
    }
    pub fn set_f32(&self, name: &str, v: f32) {
        unsafe { gl::Uniform1f(self.uni_loc(name), v); }
    }
    pub fn set_vec2_f32(&self, name: &str, v: Vector2<f32>) {
        unsafe { gl::Uniform2f(self.uni_loc(name), v.x, v.y); }
    }
    pub fn set_vec3_f32(&self, name: &str, v: Vector3<f32>) {
        unsafe { gl::Uniform3f(self.uni_loc(name), v.x, v.y, v.z); }
    }
    pub fn set_vec4_f32(&self, name: &str, v: Vector4<f32>) {
        unsafe { gl::Uniform4f(self.uni_loc(name), v.x, v.y, v.z, v.w); }
    }
    pub fn set_vec2_i32(&self, name: &str, v: Vector2<i32>) {
        unsafe { gl::Uniform2i(self.uni_loc(name), v.x, v.y); }
    }
    pub fn set_vec3_i32(&self, name: &str, v: Vector3<i32>) {
        unsafe { gl::Uniform3i(self.uni_loc(name), v.x, v.y, v.z); }
    }
    pub fn set_vec4_i32(&self, name: &str, v: Vector4<i32>) {
        unsafe { gl::Uniform4i(self.uni_loc(name), v.x, v.y, v.z, v.w); }
    }
    pub fn set_vec2_u32(&self, name: &str, v: Vector2<u32>) {
        unsafe { gl::Uniform2ui(self.uni_loc(name), v.x, v.y); }
    }
    pub fn set_vec3_u32(&self, name: &str, v: Vector3<u32>) {
        unsafe { gl::Uniform3ui(self.uni_loc(name), v.x, v.y, v.z); }
    }
    pub fn set_vec4_u32(&self, name: &str, v: Vector4<u32>) {
        unsafe { gl::Uniform4ui(self.uni_loc(name), v.x, v.y, v.z, v.w); }
    }
    pub fn set_m2_f32(&self, name: &str, m: Matrix2<f32>) {
        unsafe { gl::UniformMatrix2fv(self.uni_loc(name), 1, gl::FALSE, m.as_ptr()); }
    }
    pub fn set_m3_f32(&self, name: &str, m: Matrix3<f32>) {
        unsafe { gl::UniformMatrix3fv(self.uni_loc(name), 1, gl::FALSE, m.as_ptr()); }
    }
    pub fn set_m4_f32(&self, name: &str, m: Matrix4<f32>) {
        unsafe { gl::UniformMatrix4fv(self.uni_loc(name), 1, gl::FALSE, m.as_ptr()); }
    }
}

pub fn compile_shader(src: &str, shader_type: gl::types::GLenum) -> OpticResult<u32> {
    let c_src = CString::new(src)
        .map_err(|e| OpticError::new(OpticErrorKind::Shader, &format!("null byte in shader source: {e}")))?;

    unsafe {
        let id = gl::CreateShader(shader_type);
        gl::ShaderSource(id, 1, &c_src.as_ptr(), ptr::null());
        gl::CompileShader(id);

        let mut success = gl::FALSE as GLint;
        gl::GetShaderiv(id, gl::COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            let mut log_len = 0;
            gl::GetShaderiv(id, gl::INFO_LOG_LENGTH, &mut log_len);
            let mut log = vec![0u8; log_len.max(1) as usize - 1];
            gl::GetShaderInfoLog(
                id, log_len, ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            let msg = String::from_utf8_lossy(&log).to_string();
            gl::DeleteShader(id);
            return Err(OpticError::new(OpticErrorKind::Shader, &msg));
        }
        Ok(id)
    }
}

pub fn link_program(vert: &str, frag: &str) -> OpticResult<u32> {
    let v_id = compile_shader(vert, gl::VERTEX_SHADER)?;
    let f_id = compile_shader(frag, gl::FRAGMENT_SHADER)?;

    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, v_id);
        gl::AttachShader(program, f_id);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        gl::DeleteShader(v_id);
        gl::DeleteShader(f_id);

        if success != gl::TRUE as GLint {
            let mut log_len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);
            let mut log = vec![0u8; log_len.max(1) as usize - 1];
            gl::GetProgramInfoLog(
                program, log_len, ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            let msg = String::from_utf8_lossy(&log).to_string();
            gl::DeleteProgram(program);
            return Err(OpticError::new(OpticErrorKind::Shader, &msg));
        }
        Ok(program)
    }
}

pub fn link_compute_program(src: &str) -> OpticResult<u32> {
    let c_id = compile_shader(src, gl::COMPUTE_SHADER)?;

    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, c_id);
        gl::LinkProgram(program);

        let mut success = gl::FALSE as GLint;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        gl::DeleteShader(c_id);

        if success != gl::TRUE as GLint {
            let mut log_len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_len);
            let mut log = vec![0u8; log_len.max(1) as usize - 1];
            gl::GetProgramInfoLog(
                program, log_len, ptr::null_mut(),
                log.as_mut_ptr() as *mut gl::types::GLchar,
            );
            let msg = String::from_utf8_lossy(&log).to_string();
            gl::DeleteProgram(program);
            return Err(OpticError::new(OpticErrorKind::Shader, &msg));
        }
        Ok(program)
    }
}

pub fn delete_program(id: u32) {
    unsafe { gl::DeleteProgram(id); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slot_as_index() {
        assert_eq!(Slot::S0.as_index(), 0);
        assert_eq!(Slot::S7.as_index(), 7);
        assert_eq!(Slot::S15.as_index(), 15);
    }

    #[test]
    fn slot_total_slots() {
        assert_eq!(Slot::total_slots(), 16);
    }

    #[test]
    fn workers_empty() {
        let w = Workers::empty();
        assert_eq!(w.groups(), (0, 0, 0));
    }

    #[test]
    fn workers_set_groups() {
        let mut w = Workers::empty();
        w.set_groups(10, 1, 1);
        assert_eq!(w.groups(), (10, 1, 1));
    }

    #[test]
    fn workers_set_individual() {
        let mut w = Workers::empty();
        w.set_group_x(8);
        w.set_group_y(4);
        w.set_group_z(2);
        assert_eq!(w.groups(), (8, 4, 2));
    }

    #[test]
    fn shader_new() {
        let s = Shader::new(42, false);
        assert_eq!(s.id, 42);
        assert!(!s.is_compute);
        assert_eq!(s.tex_ids.len(), 16);
        assert_eq!(s.sbo_ids.len(), 16);
    }

    #[test]
    fn shader_new_compute() {
        let s = Shader::new(99, true);
        assert!(s.is_compute);
    }

    #[test]
    fn shader_workers_association() {
        let mut s = Shader::new(1, true);
        s.workers.set_groups(16, 1, 1);
        assert_eq!(s.workers.groups(), (16, 1, 1));
    }
}
