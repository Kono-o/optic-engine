use cgmath;
use optic_core::{ATTRType, DrawMode, OpticError, OpticErrorKind, OpticResult};

use crate::GL;

use gl::types::{GLenum, GLint, GLsizei, GLsizeiptr};
use std::ffi::c_void;
use std::ptr;

use crate::asset::attr::{ATTRInfo, ATTRName};
use crate::asset::attr::DataType;
use crate::handles::instance::InstanceBuffer;
use crate::handles::Shader;

/// Low-level OpenGL mesh handle wrapping VAO, VBO, IBO, and instance state.
///
/// Created by [`Mesh3DFile::upload`](crate::asset::Mesh3DFile::upload) or
/// [`Mesh2DFile::upload`](crate::asset::Mesh2DFile::upload). Contains the vertex
/// layouts, index state, and instance buffer binding.
#[derive(Clone, Debug)]
pub struct MeshHandle {
    pub(crate) layouts: Vec<(ATTRInfo, u32)>,
    pub(crate) has_indices: bool,
    pub(crate) vertex_count: u32,
    pub(crate) index_count: u32,
    pub(crate) vao_id: u32,
    pub(crate) vertex_buffer_id: u32,
    pub(crate) index_buffer_id: u32,
    pub(crate) vertex_stride: u32,
    pub(crate) instance_buf_id: u32,
    pub(crate) instance_count: u32,
}

impl MeshHandle {
    /// Issues the draw call for this mesh with the given draw mode.
    pub fn draw_as(&self, mode: DrawMode) {
        GL::bind_vao(self.vao_id);
        if self.instance_buf_id != 0 && self.instance_count > 0 {
            match self.has_indices {
                false => self.draw_array_instanced(mode),
                true => {
                    GL::bind_ebo(self.index_buffer_id);
                    self.draw_indexed_instanced(mode);
                }
            }
        } else {
            match self.has_indices {
                false => self.draw_array(mode),
                true => {
                    GL::bind_ebo(self.index_buffer_id);
                    self.draw_indexed(mode);
                }
            }
        }
    }

    fn draw_indexed(&self, mode: DrawMode) {
        unsafe {
            gl::DrawElements(
                match_draw_mode(&mode),
                self.index_count as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    fn draw_array(&self, mode: DrawMode) {
        unsafe {
            gl::DrawArrays(match_draw_mode(&mode), 0, self.vertex_count as GLsizei);
        }
    }

    fn draw_indexed_instanced(&self, mode: DrawMode) {
        unsafe {
            gl::DrawElementsInstanced(
                match_draw_mode(&mode),
                self.index_count as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
                self.instance_count as GLsizei,
            );
        }
    }

    fn draw_array_instanced(&self, mode: DrawMode) {
        unsafe {
            gl::DrawArraysInstanced(
                match_draw_mode(&mode),
                0,
                self.vertex_count as GLsizei,
                self.instance_count as GLsizei,
            );
        }
    }

    /// Returns the vertex layouts.
    pub fn layouts(&self) -> &Vec<(ATTRInfo, u32)> { &self.layouts }
    /// Returns `true` if this mesh uses indexed drawing.
    pub fn has_indices(&self) -> bool { self.has_indices }
    /// Returns the vertex count.
    pub fn vertex_count(&self) -> u32 { self.vertex_count }
    /// Returns the index count.
    pub fn index_count(&self) -> u32 { self.index_count }
    /// Returns the VAO ID.
    pub fn vao_id(&self) -> u32 { self.vao_id }
    /// Returns the vertex buffer ID.
    pub fn vertex_buffer_id(&self) -> u32 { self.vertex_buffer_id }
    /// Returns the index buffer ID.
    pub fn index_buffer_id(&self) -> u32 { self.index_buffer_id }
    /// Returns the vertex stride in bytes.
    pub fn vertex_stride(&self) -> u32 { self.vertex_stride }
    /// Returns the instance buffer ID (0 if not instanced).
    pub fn instance_buf_id(&self) -> u32 { self.instance_buf_id }
    /// Returns the instance count (0 if not instanced).
    pub fn instance_count(&self) -> u32 { self.instance_count }

    /// Binds an instance buffer to this mesh for instanced rendering.
    ///
    /// Instance attributes are appended after the vertex attributes using
    /// `glVertexAttribDivisor(1)`.
    pub fn set_instances(&mut self, buffer: &InstanceBuffer) {
        if buffer.count() == 0 {
            self.instance_buf_id = 0;
            self.instance_count = 0;
            return;
        }

        GL::bind_vao(self.vao_id);
        GL::bind_buffer(buffer.buf_id);

        let base_attr = self.layouts.len() as u32;
        let mut offset = 0usize;
        for (i, (info, _)) in buffer.layouts.iter().enumerate() {
            let location = base_attr + i as u32;
            let attr_size = info.elem_count * info.byte_count;
            set_attr_layout(info, location, buffer.stride as usize, offset);
            unsafe { gl::VertexAttribDivisor(location, 1); }
            offset += attr_size;
        }

        self.instance_buf_id = buffer.buf_id;
        self.instance_count = buffer.count();
    }

    /// Updates a single vertex attribute value on the GPU.
    ///
    /// The type `D` must match the attribute's declared type at creation time.
    ///
    /// # Errors
    ///
    /// Returns [`OpticErrorKind::Custom`] if `index` exceeds the vertex count,
    /// `attr_index` exceeds the layout count, or `D` does not match the
    /// attribute's declared format and element count.
    pub fn update_vertex<D: DataType>(&self, index: u32, attr_index: usize, value: D) -> OpticResult<()> {
        if index >= self.vertex_count {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!("vertex index {index} out of bounds (count: {})", self.vertex_count),
            ));
        }
        if attr_index >= self.layouts.len() {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!("attr index {attr_index} out of bounds (layout count: {})", self.layouts.len()),
            ));
        }
        let slot_info = &self.layouts[attr_index].0;
        if slot_info.byte_count != D::BYTE_COUNT || slot_info.elem_count != D::ELEM_COUNT || slot_info.typ != D::ATTR_FORMAT {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!(
                    "type mismatch: attribute {} expects {:?}[{}], got {:?}[{}]",
                    slot_info.name.as_string(),
                    slot_info.typ,
                    slot_info.elem_count,
                    D::ATTR_FORMAT,
                    D::ELEM_COUNT,
                ),
            ));
        }
        let bytes = value.u8ify();
        let off = self.compute_vert_attr_offset(attr_index, index);
        subfill_buffer(self.vertex_buffer_id, off, &bytes);
        Ok(())
    }

    /// Reads a single vertex attribute value from the GPU.
    ///
    /// # Errors
    ///
    /// Returns [`OpticErrorKind::Custom`] if `index` exceeds the vertex count,
    /// `attr_index` exceeds the layout count, or `D` does not match the
    /// attribute's declared format and element count.
    pub fn vertex<D: DataType>(&self, index: u32, attr_index: usize) -> OpticResult<D> {
        if index >= self.vertex_count {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!("vertex index {index} out of bounds (count: {})", self.vertex_count),
            ));
        }
        if attr_index >= self.layouts.len() {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!("attr index {attr_index} out of bounds (layout count: {})", self.layouts.len()),
            ));
        }
        let slot_info = &self.layouts[attr_index].0;
        if slot_info.byte_count != D::BYTE_COUNT || slot_info.elem_count != D::ELEM_COUNT || slot_info.typ != D::ATTR_FORMAT {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!(
                    "type mismatch: attribute {} expects {:?}[{}], got {:?}[{}]",
                    slot_info.name.as_string(),
                    slot_info.typ,
                    slot_info.elem_count,
                    D::ATTR_FORMAT,
                    D::ELEM_COUNT,
                ),
            ));
        }
        let off = self.compute_vert_attr_offset(attr_index, index);
        let size = slot_info.elem_count * slot_info.byte_count;
        let mut data = vec![0u8; size];
        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vertex_buffer_id);
            gl::GetBufferSubData(
                gl::ARRAY_BUFFER,
                off as isize,
                size as isize,
                data.as_mut_ptr() as *mut c_void,
            );
        }
        Ok(unsafe { std::ptr::read_unaligned(data.as_ptr() as *const D) })
    }

    /// Finds the attribute index for a given name.
    fn attr_index_by_name(&self, name: &str) -> Option<usize> {
        self.layouts.iter().position(|(info, _)| match &info.name {
            ATTRName::Custom(n) => n == name,
            other => other.as_string() == name,
        })
    }

    /// Updates a vertex attribute by name.
    ///
    /// # Errors
    ///
    /// Returns an error if no attribute matches `name` or the type does not
    /// match.
    pub fn update_custom_vertex<D: DataType>(&self, index: u32, name: &str, value: D) -> OpticResult<()> {
        let attr_index = self.attr_index_by_name(name)
            .ok_or_else(|| OpticError::new(OpticErrorKind::Custom, &format!("vertex attribute \"{name}\" not found")))?;
        self.update_vertex(index, attr_index, value)
    }

    /// Reads a vertex attribute by name.
    ///
    /// # Errors
    ///
    /// Returns an error if no attribute matches `name` or the type does not
    /// match.
    pub fn custom_vertex<D: DataType>(&self, index: u32, name: &str) -> OpticResult<D> {
        let attr_index = self.attr_index_by_name(name)
            .ok_or_else(|| OpticError::new(OpticErrorKind::Custom, &format!("vertex attribute \"{name}\" not found")))?;
        self.vertex(index, attr_index)
    }

    /// Writes raw vertex data starting at `start_vertex`.
    ///
    /// `data` length must be a multiple of the vertex stride.
    ///
    /// # Errors
    ///
    /// Returns [`OpticErrorKind::Custom`] if `data` length is not a multiple
    /// of the vertex stride or if the write extends past the vertex count.
    pub fn write_range(&self, start_vertex: u32, data: &[u8]) -> OpticResult<()> {
        let stride = self.vertex_stride as usize;
        if data.len() % stride != 0 {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "write_range data length must be a multiple of vertex stride",
            ));
        }
        let vertex_count = data.len() / stride;
        if start_vertex + vertex_count as u32 > self.vertex_count {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "write_range extends past the vertex count",
            ));
        }
        let start_off = start_vertex as usize * stride;
        subfill_buffer(self.vertex_buffer_id, start_off, data);
        Ok(())
    }

    fn compute_vert_attr_offset(&self, attr_index: usize, vertex_index: u32) -> usize {
        let mut offset = vertex_index as usize * self.vertex_stride as usize;
        for i in 0..attr_index {
            let si = &self.layouts[i].0;
            offset += si.elem_count * si.byte_count;
        }
        offset
    }

    /// Deletes the VAO, VBO, and IBO from the GPU.
    pub fn delete(self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vao_id);
            gl::DeleteBuffers(1, &self.vertex_buffer_id);
            if self.has_indices {
                gl::DeleteBuffers(1, &self.index_buffer_id);
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
        /// High-level mesh with visibility, shader, transform, and draw mode.
        ///
        /// Cloning is cheap — the handle shares the same GPU buffers.
        #[derive(Clone, Debug)]
        pub struct $mesh {
            visibility: bool,
            handle: MeshHandle,
            shader: Option<Shader>,
            transform: $transform,
            draw_mode: DrawMode,
        }

        impl $mesh {
            /// Creates a new mesh from a GPU handle with default settings.
            pub fn new(handle: MeshHandle) -> Self {
                Self {
                    visibility: true,
                    handle,
                    shader: None,
                    transform: <$transform>::default(),
                    draw_mode: DrawMode::Triangles,
                }
            }
            /// Attaches a shader to this mesh.
            pub fn set_shader(&mut self, shader: Shader) { self.shader = Some(shader); }
            /// Detaches the current shader.
            pub fn remove_shader(&mut self) { self.shader = None; }
            /// Returns a reference to the shader, if any.
            pub fn shader(&self) -> Option<&Shader> { self.shader.as_ref() }
            /// Returns a reference to the mesh handle.
            pub fn handle(&self) -> &MeshHandle { &self.handle }
            /// Returns a reference to the transform.
            pub fn transform(&self) -> &$transform { &self.transform }
            /// Returns a mutable reference to the transform.
            pub fn transform_mut(&mut self) -> &mut $transform { &mut self.transform }
            /// Returns the current draw mode.
            pub fn draw_mode(&self) -> DrawMode { self.draw_mode }
            /// Sets the draw mode.
            pub fn set_draw_mode(&mut self, draw_mode: DrawMode) { self.draw_mode = draw_mode; }
            /// Returns the index count.
            pub fn index_count(&self) -> u32 { self.handle.index_count }
            /// Returns the vertex count.
            pub fn vertex_count(&self) -> u32 { self.handle.vertex_count }
            /// Returns `true` if this mesh uses indexed drawing.
            pub fn has_indices(&self) -> bool { self.handle.has_indices }
            /// Returns `true` when the mesh has no vertices.
            pub fn is_empty(&self) -> bool { self.vertex_count() == 0 }
            /// Returns `true` when the mesh is both visible and non-empty.
            pub fn is_visible(&self) -> bool { self.visibility && !self.is_empty() }
            /// Shows or hides this mesh.
            pub fn set_visibility(&mut self, enable: bool) { self.visibility = enable; }
            /// Toggles visibility.
            pub fn toggle_visibility(&mut self) { self.visibility = !self.visibility; }
            /// Recomputes the transformation matrix.
            pub fn update(&mut self) { self.transform.calc_matrix(); }
            /// Deletes the GPU resources for this mesh's handle.
            pub fn delete(self) { self.handle.delete(); }
        }
    };
}

// A 3D mesh with position, rotation, and scale.
mesh_struct!(Mesh3D, crate::util::transform::Transform3D);
// A 2D mesh with position, rotation, scale, and layer.
mesh_struct!(Mesh2D, crate::util::transform::Transform2D);

impl Mesh3D {
    /// Prints debug information about this mesh to stdout.
    pub fn log_info(&self) {
        let shader_id = self.shader.as_ref().map(|s| s.id).unwrap_or(0);
        let mode = format!("{:?}", self.draw_mode());
        println!(
            "[Mesh3D] vis={} verts={} inds={} has_idx={} shader={} mode={} vao={} buf={} ind={}",
            self.visibility,
            self.vertex_count(),
            self.index_count(),
            self.has_indices(),
            shader_id,
            mode,
            self.handle.vao_id,
            self.handle.vertex_buffer_id,
            self.handle.index_buffer_id,
        );
    }

    /// Renders this mesh with the given view and projection matrices.
    ///
    /// Binds the shader, sets `uView`, `uProj`, `uTfm` uniforms, binds
    /// textures and storage buffers, then issues the draw call.
    pub fn render(&self, view: &cgmath::Matrix4<f32>, proj: &cgmath::Matrix4<f32>) {
        if !self.is_visible() { return; }
        let shader = match &self.shader { None => return, Some(sh) => sh };
        shader.bind();

        shader.set_m4_f32("uView", *view);
        shader.set_m4_f32("uProj", *proj);
        shader.set_m4_f32("uTfm", self.transform.matrix());

        shader.bind_textures();
        shader.bind_storages();
        self.handle.draw_as(self.draw_mode);
    }
}

impl Mesh2D {
    /// Prints debug information about this mesh to stdout.
    pub fn log_info(&self) {
        let shader_id = self.shader.as_ref().map(|s| s.id).unwrap_or(0);
        let mode = format!("{:?}", self.draw_mode());
        println!(
            "[Mesh2D] vis={} verts={} inds={} has_idx={} shader={} mode={} vao={} buf={} ind={}",
            self.visibility,
            self.vertex_count(),
            self.index_count(),
            self.has_indices(),
            shader_id,
            mode,
            self.handle.vao_id,
            self.handle.vertex_buffer_id,
            self.handle.index_buffer_id,
        );
    }

    /// Renders this 2D mesh with an orthographic projection matrix.
    ///
    /// Sets `uProj`, `uTfm`, `uLayer` uniforms, binds textures and
    /// storage buffers, then issues the draw call.
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
        self.handle.draw_as(self.draw_mode);
    }
}

/// Creates a VAO + VBO pair and returns their IDs.
pub fn create_mesh_buffer() -> (u32, u32) {
    let (mut v_id, mut b_id) = (0u32, 0u32);
    unsafe {
        gl::GenVertexArrays(1, &mut v_id);
        gl::GenBuffers(1, &mut b_id);
    }
    (v_id, b_id)
}

/// Configures a vertex attribute pointer for the given layout.
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

/// Uploads data to a VBO (full buffer replace).
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

/// Uploads data to a sub-range of a VBO.
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

/// Resizes a VBO without uploading data (contents become undefined).
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

/// Creates an IBO (element array buffer) and returns its ID.
pub fn create_index_buffer() -> u32 {
    let mut id = 0u32;
    unsafe { gl::GenBuffers(1, &mut id); }
    id
}

/// Uploads index data to an IBO.
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

/// A GPU-side shader storage buffer (SSBO) for compute or vertex pulling.
///
/// Supports dynamic resizing, filling, sub-range updates, and read-back.
///
/// # Example
///
/// ```ignore
/// use optic_render::handles::StorageBuffer;
///
/// let mut ssbo = StorageBuffer::new(1024);
/// ssbo.fill(&[1u8; 1024]);
/// let data = ssbo.fetch();
/// ```
pub struct StorageBuffer {
    pub(crate) id: u32,
    pub(crate) size: usize,
}

impl StorageBuffer {
    /// Creates a new SSBO with the given byte size (zero-initialised).
    pub fn new(size: usize) -> Self {
        let id = create_storage_buffer();
        resize_storage_buffer(id, size);
        Self { id, size }
    }

    /// Returns the GL buffer ID.
    pub fn id(&self) -> u32 { self.id }
    /// Returns the buffer byte size.
    pub fn size(&self) -> usize { self.size }

    /// Resizes the buffer (contents become undefined if size changed).
    pub fn resize(&mut self, size: usize) {
        self.bind();
        if size != self.size {
            self.size = size;
            resize_storage_buffer(self.id, self.size);
        }
    }

    /// Replaces the full buffer contents.
    pub fn fill(&mut self, data: &[u8]) {
        self.bind();
        self.resize(data.len());
        fill_storage_buffer(self.id, data);
    }

    /// Writes data at a byte offset, growing the buffer if needed.
    pub fn subfill(&mut self, offset: usize, data: &[u8]) {
        self.bind();
        let len = data.len() + offset;
        self.resize(len);
        subfill_storage_buffer(self.id, offset, data);
    }

    /// Reads the entire buffer back to the CPU.
    pub fn fetch(&self) -> Vec<u8> {
        self.bind();
        read_storage_buffer(self.id, self.size)
    }

    /// Deletes the GPU buffer.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_handle_fields() {
        let mh = MeshHandle {
            layouts: vec![],
            has_indices: false,
            vertex_count: 42,
            index_count: 0,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
        };
        assert_eq!(mh.vertex_count, 42);
        assert!(!mh.has_indices);
    }

    #[test]
    fn mesh3d_default_state() {
        let mh = MeshHandle {
            layouts: vec![],
            has_indices: true,
            vertex_count: 3,
            index_count: 3,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
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
        assert_eq!(m3d.draw_mode(), DrawMode::Triangles);
        assert!(m3d.shader.is_none());
    }

    #[test]
    fn mesh3d_visibility_toggle() {
        let mh = MeshHandle {
            layouts: vec![],
            has_indices: false,
            vertex_count: 3,
            index_count: 0,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
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
            has_indices: false,
            vertex_count: 3,
            index_count: 0,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
        };
        let mut m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        m3d.set_draw_mode(DrawMode::Points);
        assert_eq!(m3d.draw_mode(), DrawMode::Points);
    }

    #[test]
    fn mesh3d_shader_management() {
        let mh = MeshHandle {
            layouts: vec![],
            has_indices: false,
            vertex_count: 3,
            index_count: 0,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
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
            has_indices: false,
            vertex_count: 3,
            index_count: 0,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
        };
        let mut m3d = Mesh3D {
            visibility: true,
            handle: mh,
            shader: None,
            transform: crate::util::transform::Transform3D::default(),
            draw_mode: DrawMode::Triangles,
        };
        let ident = m3d.transform.matrix();
        m3d.transform.set_position(10.0, 20.0, 30.0);
        m3d.update();
        let m = m3d.transform.matrix();
        assert!(ident != m);
    }

    #[test]
    fn mesh3d_is_empty_true() {
        let mh = MeshHandle {
            layouts: vec![],
            has_indices: false,
            vertex_count: 0,
            index_count: 0,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
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
            has_indices: true,
            vertex_count: 4,
            index_count: 6,
            vao_id: 0,
            vertex_buffer_id: 0,
            index_buffer_id: 0,
            vertex_stride: 0,
            instance_buf_id: 0,
            instance_count: 0,
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
