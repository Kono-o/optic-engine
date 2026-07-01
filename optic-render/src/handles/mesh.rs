use cgmath;
use optic_core::{ATTRType, DrawMode};

use crate::GL;

use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr};
use std::ffi::c_void;
use std::ptr;

use crate::asset::attr::ATTRInfo;
use crate::handles::Shader;

#[derive(Clone, Debug)]
pub struct MeshHandle {
    pub layouts: Vec<(ATTRInfo, u32)>,
    pub draw_mode: DrawMode,
    pub has_indices: bool,
    pub vert_count: u32,
    pub ind_count: u32,
    pub vao_id: u32,
    pub buf_id: u32,
    pub ind_id: u32,
}

impl MeshHandle {
    pub fn draw(&self) {
        GL::bind_vao(self.vao_id);
        match self.has_indices {
            false => self.draw_array(),
            true => {
                GL::bind_ebo(self.ind_id);
                self.draw_indexed();
            }
        }
    }

    fn draw_indexed(&self) {
        unsafe {
            gl::DrawElements(
                match_draw_mode(&self.draw_mode),
                self.ind_count as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    fn draw_array(&self) {
        unsafe {
            gl::DrawArrays(match_draw_mode(&self.draw_mode), 0, self.vert_count as GLsizei);
        }
    }

    pub fn delete(self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao_id);
            gl::DeleteBuffers(1, &self.buf_id);
            if self.has_indices {
                gl::DeleteBuffers(1, &self.ind_id);
            }
        }
    }
}

fn match_draw_mode(dm: &DrawMode) -> GLenum {
    match dm {
        DrawMode::Points => gl::POINTS,
        DrawMode::Lines => gl::LINES,
        DrawMode::Triangles => gl::TRIANGLES,
        DrawMode::Strip => gl::TRIANGLE_STRIP,
    }
}

macro_rules! mesh_struct {
    ($mesh:ident, $transform:ty) => {
        #[derive(Clone, Debug)]
        pub struct $mesh {
            pub visibility: bool,
            pub handle: MeshHandle,
            pub shader: Option<Shader>,
            pub transform: $transform,
            pub draw_mode: DrawMode,
        }

        impl $mesh {
            pub fn set_shader(&mut self, shader: Shader) { self.shader = Some(shader); }
            pub fn remove_shader(&mut self) { self.shader = None; }
            pub fn get_draw_mode(&self) -> DrawMode { self.handle.draw_mode }
            pub fn set_draw_mode(&mut self, draw_mode: DrawMode) { self.handle.draw_mode = draw_mode; }
            pub fn index_count(&self) -> u32 { self.handle.ind_count }
            pub fn vertex_count(&self) -> u32 { self.handle.vert_count }
            pub fn has_indices(&self) -> bool { self.handle.has_indices }
            pub fn is_empty(&self) -> bool { self.vertex_count() == 0 }
            pub fn is_visible(&self) -> bool { self.visibility && !self.is_empty() }
            pub fn set_visibility(&mut self, enable: bool) { self.visibility = enable; }
            pub fn toggle_visibility(&mut self) { self.visibility = !self.visibility; }
            pub fn update(&mut self) { self.transform.calc_matrix(); }
            pub fn delete(self) { self.handle.delete(); }
        }
    };
}

mesh_struct!(Mesh3D, crate::util::transform::Transform3D);
mesh_struct!(Mesh2D, crate::util::transform::Transform2D);

impl Mesh3D {
    pub fn log_info(&self) {
        let shader_id = self.shader.as_ref().map(|s| s.id).unwrap_or(0);
        let mode = format!("{:?}", self.get_draw_mode());
        println!(
            "[Mesh3D] vis={} verts={} inds={} has_idx={} shader={} mode={} vao={} buf={} ind={}",
            self.visibility,
            self.vertex_count(),
            self.index_count(),
            self.has_indices(),
            shader_id,
            mode,
            self.handle.vao_id,
            self.handle.buf_id,
            self.handle.ind_id,
        );
    }

    pub fn render(&self, view: &cgmath::Matrix4<f32>, proj: &cgmath::Matrix4<f32>) {
        if !self.is_visible() { return; }
        let shader = match &self.shader { None => return, Some(sh) => sh };
        shader.bind();

        shader.set_m4_f32("uView", *view);
        shader.set_m4_f32("uProj", *proj);
        shader.set_m4_f32("uTfm", self.transform.matrix());

        shader.bind_textures();
        shader.bind_storages();
        self.handle.draw();
    }
}

impl Mesh2D {
    pub fn log_info(&self) {
        let shader_id = self.shader.as_ref().map(|s| s.id).unwrap_or(0);
        let mode = format!("{:?}", self.get_draw_mode());
        println!(
            "[Mesh2D] vis={} verts={} inds={} has_idx={} shader={} mode={} vao={} buf={} ind={}",
            self.visibility,
            self.vertex_count(),
            self.index_count(),
            self.has_indices(),
            shader_id,
            mode,
            self.handle.vao_id,
            self.handle.buf_id,
            self.handle.ind_id,
        );
    }

    pub fn render(&self, proj: &cgmath::Matrix4<f32>) {
        if !self.is_visible() { return; }
        let shader = match &self.shader { None => return, Some(sh) => sh };
        shader.bind();

        shader.set_m4_f32("uProj", *proj);
        let tfm = self.transform.matrix();
        let layer = self.transform.layer() as u32;
        shader.set_m4_f32("uTfm", tfm);
        shader.set_u32("uLayer", layer);

        shader.bind_textures();
        shader.bind_storages();
        self.handle.draw();
    }
}

pub fn create_mesh_buffer() -> (u32, u32) {
    let (mut v_id, mut b_id) = (0u32, 0u32);
    unsafe {
        gl::GenVertexArrays(1, &mut v_id);
        gl::GenBuffers(1, &mut b_id);
    }
    (v_id, b_id)
}

pub fn set_attr_layout(attr: &ATTRInfo, attr_id: u32, stride: usize, local_offset: usize) {
    unsafe {
        gl::VertexAttribPointer(
            attr_id,
            attr.elem_count as GLint,
            match_attr_type(&attr.typ),
            gl::FALSE,
            stride as GLsizei,
            match local_offset {
                0 => ptr::null(),
                _ => local_offset as *const c_void,
            },
        );
        gl::EnableVertexAttribArray(attr_id);
    }
}

pub fn fill_buffer(id: u32, data: &[u8]) {
    unsafe {
        GL::bind_buffer(id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            data.len() as GLsizeiptr,
            data.as_ptr() as *const c_void,
            gl::DYNAMIC_DRAW,
        );
    }
}

pub fn subfill_buffer(id: u32, offset: usize, data: &[u8]) {
    unsafe {
        GL::bind_buffer(id);
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            offset as isize,
            data.len() as isize,
            data.as_ptr() as *const c_void,
        );
    }
}

pub fn resize_buffer(id: u32, size: usize) {
    unsafe {
        GL::bind_buffer(id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size as GLsizeiptr,
            ptr::null(),
            gl::DYNAMIC_DRAW,
        );
    }
}

pub fn create_index_buffer() -> u32 {
    let mut id = 0u32;
    unsafe { gl::GenBuffers(1, &mut id); }
    id
}

pub fn fill_index_buffer(id: u32, data: &[u32]) {
    unsafe {
        GL::bind_ebo(id);
        gl::BufferData(
            gl::ELEMENT_ARRAY_BUFFER,
            (data.len() * size_of::<u32>()) as GLsizeiptr,
            data.as_ptr() as *const c_void,
            gl::DYNAMIC_DRAW,
        );
    }
}

pub struct StorageBuffer {
    pub id: u32,
    pub size: usize,
}

impl StorageBuffer {
    pub fn new(size: usize) -> Self {
        let id = create_storage_buffer();
        resize_storage_buffer(id, size);
        Self { id, size }
    }

    pub fn resize(&mut self, size: usize) {
        self.bind();
        if size != self.size {
            self.size = size;
            resize_storage_buffer(self.id, self.size);
        }
    }

    pub fn fill(&mut self, data: &[u8]) {
        self.bind();
        self.resize(data.len());
        fill_storage_buffer(self.id, data);
    }

    pub fn subfill(&mut self, offset: usize, data: &[u8]) {
        self.bind();
        let len = data.len() + offset;
        self.resize(len);
        subfill_storage_buffer(self.id, offset, data);
    }

    pub fn fetch(&self) -> Vec<u8> {
        self.bind();
        read_storage_buffer(self.id, self.size)
    }

    pub fn delete(self) {
        unsafe { gl::DeleteBuffers(1, &self.id); }
    }

    fn bind(&self) {
        unsafe { gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, self.id); }
    }
}

fn create_storage_buffer() -> u32 {
    let mut id = 0u32;
    unsafe { gl::GenBuffers(1, &mut id); }
    id
}

fn fill_storage_buffer(id: u32, data: &[u8]) {
    unsafe {
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER,
            data.len() as GLsizeiptr,
            data.as_ptr() as *const c_void,
            gl::DYNAMIC_DRAW,
        );
    }
}

fn subfill_storage_buffer(id: u32, offset: usize, data: &[u8]) {
    unsafe {
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
        gl::BufferSubData(
            gl::SHADER_STORAGE_BUFFER,
            offset as isize,
            data.len() as isize,
            data.as_ptr() as *const c_void,
        );
    }
}

fn resize_storage_buffer(id: u32, size: usize) {
    unsafe {
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
        gl::BufferData(
            gl::SHADER_STORAGE_BUFFER,
            size as GLsizeiptr,
            ptr::null(),
            gl::DYNAMIC_DRAW,
        );
    }
}

fn read_storage_buffer(id: u32, size: usize) -> Vec<u8> {
    let mut data = vec![0u8; size];
    unsafe {
        gl::BindBuffer(gl::SHADER_STORAGE_BUFFER, id);
        gl::GetBufferSubData(
            gl::SHADER_STORAGE_BUFFER,
            0,
            size as GLsizeiptr,
            data.as_mut_ptr() as *mut c_void,
        );
    }
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_handle_fields() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: false,
            vert_count: 42,
            ind_count: 0,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        assert_eq!(mh.vert_count, 42);
        assert!(!mh.has_indices);
    }

    #[test]
    fn mesh3d_default_state() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: true,
            vert_count: 3,
            ind_count: 3,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        let m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        assert!(m3d.is_visible());
        assert!(!m3d.is_empty());
        assert_eq!(m3d.vertex_count(), 3);
        assert_eq!(m3d.index_count(), 3);
        assert!(m3d.has_indices());
        assert_eq!(m3d.get_draw_mode(), DrawMode::Triangles);
        assert!(m3d.shader.is_none());
    }

    #[test]
    fn mesh3d_visibility_toggle() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: false,
            vert_count: 3,
            ind_count: 0,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        let mut m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        assert!(m3d.is_visible());
        m3d.set_visibility(false);
        assert!(!m3d.is_visible());
        m3d.toggle_visibility();
        assert!(m3d.is_visible());
    }

    #[test]
    fn mesh3d_set_draw_mode() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: false,
            vert_count: 3,
            ind_count: 0,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        let mut m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        m3d.set_draw_mode(DrawMode::Points);
        assert_eq!(m3d.get_draw_mode(), DrawMode::Points);
    }

    #[test]
    fn mesh3d_shader_management() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: false,
            vert_count: 3,
            ind_count: 0,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        let mut m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        assert!(m3d.shader.is_none());
        let s = Shader::new(99, false);
        m3d.set_shader(s);
        assert!(m3d.shader.is_some());
        assert_eq!(m3d.shader.as_ref().unwrap().id, 99);
        m3d.remove_shader();
        assert!(m3d.shader.is_none());
    }

    #[test]
    fn mesh3d_update_calc_matrix() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: false,
            vert_count: 3,
            ind_count: 0,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        let mut m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        let ident = m3d.transform.matrix();
        m3d.transform.set_pos_all(10.0, 20.0, 30.0);
        m3d.update();
        let m = m3d.transform.matrix();
        assert!(ident != m);
    }

    #[test]
    fn mesh3d_is_empty_true() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: false,
            vert_count: 0,
            ind_count: 0,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        let m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        assert!(m3d.is_empty());
        assert!(!m3d.is_visible());
    }

    #[test]
    fn mesh2d_default_state() {
        let mh = MeshHandle {
            layouts: vec![],
            draw_mode: DrawMode::Triangles,
            has_indices: true,
            vert_count: 4,
            ind_count: 6,
            vao_id: 0,
            buf_id: 0,
            ind_id: 0,
        };
        let m2d = Mesh2D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform2D::default(),
            draw_mode: DrawMode::Triangles,
        };
        assert!(m2d.is_visible());
        assert_eq!(m2d.vertex_count(), 4);
        assert!(m2d.has_indices());
    }

    #[test]
    fn storage_buffer_create() {
        // Only test the struct fields, not GL creation
        let sb = StorageBuffer { id: 0, size: 0 };
        assert_eq!(sb.id, 0);
        assert_eq!(sb.size, 0);
    }
}

fn match_attr_type(attr_type: &ATTRType) -> GLenum {
    match attr_type {
        ATTRType::I8 => gl::BYTE,
        ATTRType::U8 => gl::UNSIGNED_BYTE,
        ATTRType::I16 => gl::SHORT,
        ATTRType::U16 => gl::UNSIGNED_SHORT,
        ATTRType::I32 => gl::INT,
        ATTRType::U32 => gl::UNSIGNED_INT,
        ATTRType::F32 => gl::FLOAT,
        ATTRType::F64 => gl::DOUBLE,
    }
}
