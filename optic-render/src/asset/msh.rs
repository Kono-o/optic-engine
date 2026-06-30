use optic_core::{DrawMode, OpticError, OpticErrorKind, OpticResult};
use cgmath::Vector2;
use std::collections::HashMap;

use crate::asset::attr::{ATTRInfo, ColATTR, CustomATTR, IndATTR, NrmATTR, Pos2DATTR, Pos3DATTR, UVMATTR};
use crate::asset::attr::DataType;
use crate::handles::mesh::{
    create_index_buffer, create_mesh_buffer, fill_buffer, fill_index_buffer, set_attr_layout,
    MeshHandle,
};

enum OBJ {
    Parsed {
        pos_attr: Pos3DATTR,
        col_attr: ColATTR,
        uvm_attr: UVMATTR,
        nrm_attr: NrmATTR,
        ind_attr: IndATTR,
    },
    NonTriangle(String),
}

impl OBJ {
    fn parse(src: &str) -> Self {
        let mut pos_attr = Pos3DATTR::empty();
        let mut col_attr = ColATTR::empty();
        let mut uvm_attr = UVMATTR::empty();
        let mut nrm_attr = NrmATTR::empty();
        let mut ind_attr = IndATTR::empty();

        let mut pos_data = Vec::new();
        let mut uvm_data = Vec::new();
        let mut nrm_data = Vec::new();
        type Vert = Vec<usize>;
        let mut verts: Vec<Vert> = Vec::new();
        let mut unique_verts = HashMap::new();

        for line in src.lines() {
            let line = line.trim();
            let words: Vec<&str> = line.split(' ').collect();
            if words.is_empty() { continue; }
            match words[0] {
                "v" => pos_data.push(Self::parse_3(&words)),
                "vt" => uvm_data.push(Self::parse_2(&words)),
                "vn" => nrm_data.push(Self::parse_3(&words)),
                "f" => {
                    if words.len() != 4 {
                        return OBJ::NonTriangle(line.to_string());
                    }
                    for word in &words[1..] {
                        let tokens: Vec<&str> = word.split('/').collect();
                        let vert = tokens.iter()
                            .map(|s| s.parse::<usize>().unwrap_or(1).saturating_sub(1))
                            .collect();
                        verts.push(vert);
                    }
                }
                _ => {}
            }
        }

        let attr_count = verts.first().map_or(0, |v| v.len());
        let pos_exists = attr_count > 0;
        let uvm_exists = attr_count > 1;
        let nrm_exists = attr_count > 2;

        let def_uvm = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0]];
        let def_col = [1.0, 1.0, 1.0, 1.0];
        let def_nrm = [1.0, 1.0, 1.0];

        for (i, vert) in verts.iter().enumerate() {
            let key = (
                pos_exists.then(|| vert[0]),
                uvm_exists.then(|| vert[1]),
                nrm_exists.then(|| vert[2]),
            );

            if let Some(&idx) = unique_verts.get(&key) {
                ind_attr.push(idx as u32);
            } else {
                let v_local = i % 3;
                let new = pos_attr.data.len();
                unique_verts.insert(key, new);
                pos_attr.push(if pos_exists { pos_data[vert[0]] } else { [0.0; 3] });
                uvm_attr.push(if uvm_exists { uvm_data[vert[1]] } else { def_uvm[v_local] });
                nrm_attr.push(if nrm_exists { nrm_data[vert[2]] } else { def_nrm });
                col_attr.push(def_col);
                ind_attr.push(new as u32);
            }
        }

        OBJ::Parsed { pos_attr, col_attr, uvm_attr, nrm_attr, ind_attr }
    }

    fn parse_2(words: &[&str]) -> [f32; 2] {
        let x = words.get(1).and_then(|w| w.parse().ok()).unwrap_or(0.0);
        let y = words.get(2).and_then(|w| w.parse().ok()).unwrap_or(0.0);
        [x, 1.0 - y]
    }

    fn parse_3(words: &[&str]) -> [f32; 3] {
        let x = words.get(1).and_then(|w| w.parse().ok()).unwrap_or(0.0);
        let y = words.get(2).and_then(|w| w.parse().ok()).unwrap_or(0.0);
        let z = words.get(3).and_then(|w| w.parse().ok()).unwrap_or(0.0);
        [x, y, z]
    }
}

pub struct Mesh3DFile {
    pub pos_attr: Pos3DATTR,
    pub col_attr: ColATTR,
    pub uvm_attr: UVMATTR,
    pub nrm_attr: NrmATTR,
    pub ind_attr: IndATTR,
    pub cus_attrs: Vec<CustomATTR>,
}

impl Mesh3DFile {
    pub fn empty() -> Self {
        Self {
            pos_attr: Pos3DATTR::empty(),
            col_attr: ColATTR::empty(),
            uvm_attr: UVMATTR::empty(),
            nrm_attr: NrmATTR::empty(),
            ind_attr: IndATTR::empty(),
            cus_attrs: Vec::new(),
        }
    }

    pub fn from_obj_src(src: &str) -> OpticResult<Self> {
        match OBJ::parse(src) {
            OBJ::NonTriangle(line) => Err(OpticError::new(
                OpticErrorKind::Asset,
                &format!("mesh not triangulated at: {line}"),
            )),
            OBJ::Parsed { pos_attr, col_attr, uvm_attr, nrm_attr, ind_attr } => {
                Ok(Self { pos_attr, col_attr, uvm_attr, nrm_attr, ind_attr, cus_attrs: Vec::new() })
            }
        }
    }

    pub fn from_obj(path: &str) -> OpticResult<Self> {
        let src = optic_file::read_string(path)?;
        Self::from_obj_src(&src)
    }

    pub fn from_obj_cached(path: &str) -> OpticResult<Self> {
        let cached = optic_file::cached_path(path, "omesh");
        if optic_file::exists(&cached) {
            let src = optic_file::read_string(&cached)?;
            return Self::from_obj_src(&src);
        }
        let src = optic_file::read_string(path)?;
        let mesh = Self::from_obj_src(&src)?;
        if let Some(parent) = std::path::Path::new(&cached).parent() {
            let _ = optic_file::create_dir(&parent.to_string_lossy());
        }
        optic_file::write_string(&cached, &src)?;
        Ok(mesh)
    }

    pub fn attach_custom_attr(&mut self, attr: CustomATTR) {
        self.cus_attrs.push(attr);
    }

    pub fn has_no_attr(&self) -> bool {
        self.starts_with_custom() && self.cus_attrs.is_empty()
    }

    pub fn starts_with_custom(&self) -> bool {
        self.pos_attr.is_empty() && self.col_attr.is_empty()
            && self.uvm_attr.is_empty() && self.nrm_attr.is_empty()
    }

    pub fn ship(&self) -> MeshHandle {
        create_mesh3d_handle(self)
    }
}

pub struct Mesh2DFile {
    pub pos_attr: Pos2DATTR,
    pub layer: u8,
    pub aspect: f32,
    pub col_attr: ColATTR,
    pub uvm_attr: UVMATTR,
    pub ind_attr: IndATTR,
    pub cus_attrs: Vec<CustomATTR>,
}

pub enum Center {
    TopLeft, TopRight, BottomLeft, BottomRight, Middle, Custom(f32, f32),
}

impl Center {
    fn offset(&self) -> Vector2<f32> {
        match self {
            Center::TopLeft => Vector2::new(1.0, -1.0),
            Center::TopRight => Vector2::new(-1.0, -1.0),
            Center::BottomRight => Vector2::new(-1.0, 1.0),
            Center::BottomLeft => Vector2::new(1.0, 1.0),
            Center::Middle => Vector2::new(0.0, 0.0),
            Center::Custom(x, y) => Vector2::new(-x, -y),
        }
    }
}

impl Mesh2DFile {
    pub fn empty() -> Self {
        Self {
            pos_attr: Pos2DATTR::empty(),
            layer: 0,
            aspect: 1.0,
            col_attr: ColATTR::empty(),
            uvm_attr: UVMATTR::empty(),
            ind_attr: IndATTR::empty(),
            cus_attrs: Vec::new(),
        }
    }

    fn offset_pos_by_center(&mut self, center: &Center) {
        let offset = center.offset();
        for pos in &mut self.pos_attr.data {
            pos[0] += offset.x * self.aspect;
            pos[1] += offset.y;
        }
    }

    pub fn set_pos_attr(&mut self, attr: Pos2DATTR) { self.pos_attr = attr; }
    pub fn set_layer(&mut self, layer: u8) { self.layer = layer; }
    pub fn set_center(&mut self, center: Center) { self.offset_pos_by_center(&center); }
    pub fn set_col_attr(&mut self, attr: ColATTR) { self.col_attr = attr; }
    pub fn set_uvm_attr(&mut self, attr: UVMATTR) { self.uvm_attr = attr; }
    pub fn set_ind_attr(&mut self, attr: IndATTR) { self.ind_attr = attr; }

    pub fn quad(size: &optic_core::Size2D) -> Self {
        let mut mesh = Self::empty();
        mesh.aspect = size.aspect_ratio();
        let x = mesh.aspect;
        let y = 1.0;
        mesh.set_pos_attr(Pos2DATTR::from_array(&[[-x, y], [x, y], [x, -y], [-x, -y]]));
        mesh.offset_pos_by_center(&Center::Middle);
        mesh.set_col_attr(ColATTR::from_array(&[[1.0, 1.0, 1.0, 1.0]; 4]));
        mesh.set_uvm_attr(UVMATTR::from_array(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]));
        mesh.set_ind_attr(IndATTR::from_array(&[0, 2, 1, 2, 0, 3]));
        mesh
    }

    pub fn attach_custom_attr(&mut self, attr: CustomATTR) {
        self.cus_attrs.push(attr);
    }

    pub fn starts_with_custom(&self) -> bool {
        self.pos_attr.is_empty() && self.col_attr.is_empty()
            && self.uvm_attr.is_empty()
    }

    pub fn ship(&self) -> MeshHandle {
        create_mesh2d_handle(self)
    }
}

#[allow(dead_code)]
trait BufferExt {
    fn push_attr<T: DataType>(&mut self, attr: &[T]);
}

impl BufferExt for Vec<u8> {
    fn push_attr<T: DataType>(&mut self, attr: &[T]) {
        for elem in attr {
            self.extend_from_slice(&elem.u8ify());
        }
    }
}

#[allow(unused_mut)]
fn create_mesh3d_handle(mesh: &Mesh3DFile) -> MeshHandle {
    let (vao_id, buf_id) = create_mesh_buffer();
    let ind_id = create_index_buffer();

    let mut stride = 0usize;
    let mut attrs: Vec<(&ATTRInfo, &dyn AsDataRef)> = Vec::new();
    let mut has_indices = false;
    let mut ind_count = 0;
    let mut ind_data: &[u32];

    if !mesh.pos_attr.is_empty() {
        let info = &mesh.pos_attr.info;
        stride += info.elem_count * info.byte_count;
        attrs.push((info, &mesh.pos_attr.data as &dyn AsDataRef));
    }
    if !mesh.col_attr.is_empty() {
        let info = &mesh.col_attr.info;
        stride += info.elem_count * info.byte_count;
        attrs.push((info, &mesh.col_attr.data as &dyn AsDataRef));
    }
    if !mesh.uvm_attr.is_empty() {
        let info = &mesh.uvm_attr.info;
        stride += info.elem_count * info.byte_count;
        attrs.push((info, &mesh.uvm_attr.data as &dyn AsDataRef));
    }
    if !mesh.nrm_attr.is_empty() {
        let info = &mesh.nrm_attr.info;
        stride += info.elem_count * info.byte_count;
        attrs.push((info, &mesh.nrm_attr.data as &dyn AsDataRef));
    }
    for cus in &mesh.cus_attrs {
        stride += cus.info.elem_count * cus.info.byte_count;
        attrs.push((&cus.info, &cus.data as &dyn AsDataRef));
    }

    let vert_count = if mesh.starts_with_custom() {
        let first = &mesh.cus_attrs[0];
        first.data.len() / (first.info.byte_count * first.info.elem_count)
    } else {
        mesh.pos_attr.data.len()
    } as u32;

    let mut buffer: Vec<u8> = Vec::new();
    for i in 0..vert_count as usize {
        for &(info, data) in &attrs {
            let elem_size = info.elem_count * info.byte_count;
            let start = i * elem_size;
            let end = start + elem_size;
            buffer.extend_from_slice(&data.as_bytes()[start..end]);
        }
    }

    crate::GL::bind_vao(vao_id);
    let mut attr_id = 0u32;
    let mut offset = 0usize;
    let mut layouts = Vec::new();

    for &(info, _) in &attrs {
        set_attr_layout(info, attr_id, stride, offset);
        offset += info.elem_count * info.byte_count;
        layouts.push((info.clone(), attr_id));
        attr_id += 1;
    }

    if !buffer.is_empty() {
        fill_buffer(buf_id, &buffer);
    }
    crate::GL::unbind_buffer();

    if !mesh.ind_attr.is_empty() {
        has_indices = true;
        ind_data = &mesh.ind_attr.data;
        ind_count = ind_data.len() as u32;
        fill_index_buffer(ind_id, ind_data);
        crate::GL::unbind_ebo();
    }

    MeshHandle {
        layouts,
        draw_mode: DrawMode::Triangles,
        has_indices,
        vert_count,
        ind_count,
        vao_id,
        buf_id,
        ind_id,
    }
}

#[allow(unused_mut)]
fn create_mesh2d_handle(mesh: &Mesh2DFile) -> MeshHandle {
    let (vao_id, buf_id) = create_mesh_buffer();
    let ind_id = create_index_buffer();

    let mut stride = 0usize;
    let mut attrs: Vec<(&ATTRInfo, &dyn AsDataRef)> = Vec::new();
    let mut has_indices = false;
    let mut ind_data: &[u32];
    let mut ind_count = 0;

    if !mesh.pos_attr.is_empty() {
        let info = &mesh.pos_attr.info;
        stride += info.elem_count * info.byte_count;
        attrs.push((info, &mesh.pos_attr.data as &dyn AsDataRef));
    }
    if !mesh.col_attr.is_empty() {
        let info = &mesh.col_attr.info;
        stride += info.elem_count * info.byte_count;
        attrs.push((info, &mesh.col_attr.data as &dyn AsDataRef));
    }
    if !mesh.uvm_attr.is_empty() {
        let info = &mesh.uvm_attr.info;
        stride += info.elem_count * info.byte_count;
        attrs.push((info, &mesh.uvm_attr.data as &dyn AsDataRef));
    }
    for cus in &mesh.cus_attrs {
        stride += cus.info.elem_count * cus.info.byte_count;
        attrs.push((&cus.info, &cus.data as &dyn AsDataRef));
    }

    let vert_count = if mesh.starts_with_custom() {
        let first = &mesh.cus_attrs[0];
        first.data.len() / (first.info.byte_count * first.info.elem_count)
    } else {
        mesh.pos_attr.data.len()
    } as u32;

    let mut buffer: Vec<u8> = Vec::new();
    for i in 0..vert_count as usize {
        for &(info, data) in &attrs {
            let elem_size = info.elem_count * info.byte_count;
            let start = i * elem_size;
            let end = start + elem_size;
            buffer.extend_from_slice(&data.as_bytes()[start..end]);
        }
    }

    crate::GL::bind_vao(vao_id);
    let mut attr_id = 0u32;
    let mut offset = 0usize;
    let mut layouts = Vec::new();

    for &(info, _) in &attrs {
        set_attr_layout(info, attr_id, stride, offset);
        offset += info.elem_count * info.byte_count;
        layouts.push((info.clone(), attr_id));
        attr_id += 1;
    }

    if !buffer.is_empty() {
        fill_buffer(buf_id, &buffer);
    }
    crate::GL::unbind_buffer();

    if !mesh.ind_attr.is_empty() {
        has_indices = true;
        ind_data = &mesh.ind_attr.data;
        ind_count = ind_data.len() as u32;
        fill_index_buffer(ind_id, ind_data);
        crate::GL::unbind_ebo();
    }

    MeshHandle {
        layouts,
        draw_mode: DrawMode::Triangles,
        has_indices,
        vert_count,
        ind_count,
        vao_id,
        buf_id,
        ind_id,
    }
}

trait AsDataRef {
    fn as_bytes(&self) -> &[u8];
}

impl AsDataRef for Vec<[f32; 3]> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 12) }
    }
}

impl AsDataRef for Vec<[f32; 2]> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 8) }
    }
}

impl AsDataRef for Vec<[f32; 4]> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 16) }
    }
}

impl AsDataRef for Vec<u32> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 4) }
    }
}

impl AsDataRef for Vec<u8> {
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use optic_core::Size2D;

    #[test]
    fn mesh3d_file_empty() {
        let m = Mesh3DFile::empty();
        assert!(m.pos_attr.is_empty());
        assert!(m.col_attr.is_empty());
        assert!(m.uvm_attr.is_empty());
        assert!(m.nrm_attr.is_empty());
        assert!(m.ind_attr.is_empty());
        assert!(m.cus_attrs.is_empty());
    }

    #[test]
    fn mesh3d_file_starts_with_custom() {
        let m = Mesh3DFile::empty();
        assert!(m.starts_with_custom());
    }

    #[test]
    fn mesh3d_file_has_no_attr_true() {
        let m = Mesh3DFile::empty();
        assert!(m.has_no_attr());
    }

    #[test]
    fn mesh3d_file_has_no_attr_false_with_pos() {
        let mut m = Mesh3DFile::empty();
        m.pos_attr.push([1.0, 2.0, 3.0]);
        assert!(!m.has_no_attr());
    }

    #[test]
    fn mesh3d_file_attach_custom() {
        let mut m = Mesh3DFile::empty();
        let attr = CustomATTR::empty::<u32>("bone_ids");
        m.attach_custom_attr(attr);
        assert_eq!(m.cus_attrs.len(), 1);
    }

    #[test]
    fn mesh2d_file_empty() {
        let m = Mesh2DFile::empty();
        assert!(m.pos_attr.is_empty());
        assert!(m.col_attr.is_empty());
        assert!(m.uvm_attr.is_empty());
        assert!(m.ind_attr.is_empty());
        assert!(m.cus_attrs.is_empty());
        assert_eq!(m.layer, 0);
        assert!((m.aspect - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn mesh2d_file_starts_with_custom() {
        let m = Mesh2DFile::empty();
        assert!(m.starts_with_custom());
    }

    #[test]
    fn mesh2d_quad() {
        let size = Size2D::from(100, 100);
        let m = Mesh2DFile::quad(&size);
        assert_eq!(m.pos_attr.data.len(), 4);
        assert_eq!(m.col_attr.data.len(), 4);
        assert_eq!(m.uvm_attr.data.len(), 4);
        assert_eq!(m.ind_attr.data.len(), 6);
    }

    #[test]
    fn mesh2d_setters() {
        let mut m = Mesh2DFile::empty();
        m.set_pos_attr(Pos2DATTR::from_array(&[[0.0, 0.0], [1.0, 0.0]]));
        m.set_col_attr(ColATTR::from_array(&[[1.0; 4]; 2]));
        m.set_uvm_attr(UVMATTR::from_array(&[[0.0, 0.0], [1.0, 0.0]]));
        m.set_ind_attr(IndATTR::from_array(&[0, 1]));
        m.set_layer(5);
        assert_eq!(m.pos_attr.data.len(), 2);
        assert_eq!(m.layer, 5);
    }

    #[test]
    fn mesh2d_set_center_topleft() {
        let mut m = Mesh2DFile::empty();
        m.set_pos_attr(Pos2DATTR::from_array(&[[0.0, 0.0]]));
        m.set_center(Center::TopLeft);
        // TopLeft offset is (1.0, -1.0)
        assert!((m.pos_attr.data[0][0] - 1.0).abs() < f32::EPSILON);
        assert!((m.pos_attr.data[0][1] - (-1.0)).abs() < f32::EPSILON);
    }

    #[test]
    fn mesh2d_set_center_custom() {
        let mut m = Mesh2DFile::empty();
        m.set_pos_attr(Pos2DATTR::from_array(&[[0.0, 0.0]]));
        m.set_center(Center::Custom(0.5, 0.5));
        assert!((m.pos_attr.data[0][0] - (-0.5)).abs() < f32::EPSILON);
        assert!((m.pos_attr.data[0][1] - (-0.5)).abs() < f32::EPSILON);
    }

    #[test]
    fn mesh3d_from_obj_src() {
        let obj = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3";
        let mesh = Mesh3DFile::from_obj_src(obj).unwrap();
        assert_eq!(mesh.pos_attr.data.len(), 3);
        assert_eq!(mesh.ind_attr.data.len(), 3);
    }

    #[test]
    fn mesh3d_from_obj_src_non_triangle() {
        let obj = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3 4";
        let result = Mesh3DFile::from_obj_src(obj);
        assert!(result.is_err());
    }

    #[test]
    fn mesh3d_from_obj_cached_roundtrip() {
        let obj = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3";
        let obj_path = "/tmp/optic_test_mesh3d_cached.obj";
        optic_file::write_string(obj_path, obj).unwrap();
        // from_obj_cached should cache then return
        let mesh = Mesh3DFile::from_obj_cached(obj_path).unwrap();
        assert_eq!(mesh.pos_attr.data.len(), 3);
        let cached = optic_file::cached_path(obj_path, "omesh");
        assert!(optic_file::exists(&cached));
        // second call should read from cache
        let mesh2 = Mesh3DFile::from_obj_cached(obj_path).unwrap();
        assert_eq!(mesh2.pos_attr.data.len(), 3);
        let _ = std::fs::remove_file(obj_path);
        if let Some(parent) = std::path::Path::new(&cached).parent() {
            let _ = std::fs::remove_dir_all(parent);
        }
    }

    #[test]
    fn obj_parse_simple_triangle() {
        let obj = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3";
        match OBJ::parse(obj) {
            OBJ::Parsed { pos_attr, ind_attr, .. } => {
                assert_eq!(pos_attr.data.len(), 3);
                assert_eq!(ind_attr.data.len(), 3);
                assert_eq!(pos_attr.data[0], [0.0, 0.0, 0.0]);
                assert_eq!(pos_attr.data[1], [1.0, 0.0, 0.0]);
                assert_eq!(pos_attr.data[2], [0.0, 1.0, 0.0]);
            }
            OBJ::NonTriangle(_) => panic!("expected parsed triangle"),
        }
    }

    #[test]
    fn obj_parse_with_uv_and_normals() {
        let obj = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nvt 0.0 0.0\nvt 1.0 0.0\nvt 0.0 1.0\nvn 0.0 0.0 1.0\nf 1/1/1 2/2/1 3/3/1";
        match OBJ::parse(obj) {
            OBJ::Parsed { pos_attr, uvm_attr, nrm_attr, ind_attr, .. } => {
                assert_eq!(pos_attr.data.len(), 3);
                assert_eq!(uvm_attr.data.len(), 3);
                assert!(!nrm_attr.data.is_empty());
                assert_eq!(ind_attr.data.len(), 3);
            }
            OBJ::NonTriangle(_) => panic!("expected parsed triangle"),
        }
    }

    #[test]
    fn obj_parse_non_triangle() {
        let obj = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nv 1.0 1.0 0.0\nf 1 2 3 4";
        match OBJ::parse(obj) {
            OBJ::Parsed { .. } => panic!("expected non-triangle error"),
            OBJ::NonTriangle(line) => assert!(line.contains("4")),
        }
    }

    #[test]
    fn obj_parse_empty() {
        match OBJ::parse("") {
            OBJ::Parsed { pos_attr, ind_attr, .. } => {
                assert!(pos_attr.is_empty());
                assert!(ind_attr.is_empty());
            }
            OBJ::NonTriangle(_) => panic!("expected empty parsed"),
        }
    }

    #[test]
    fn obj_parse_parse_2() {
        let words = vec!["vt", "0.5", "0.5"];
        let result = OBJ::parse_2(&words);
        assert!((result[0] - 0.5).abs() < f32::EPSILON);
        assert!((result[1] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn obj_parse_parse_3() {
        let words = vec!["v", "1.0", "2.0", "3.0"];
        let result = OBJ::parse_3(&words);
        assert_eq!(result, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn center_variants() {
        assert_eq!(Center::TopLeft.offset(), Vector2::new(1.0, -1.0));
        assert_eq!(Center::TopRight.offset(), Vector2::new(-1.0, -1.0));
        assert_eq!(Center::BottomRight.offset(), Vector2::new(-1.0, 1.0));
        assert_eq!(Center::BottomLeft.offset(), Vector2::new(1.0, 1.0));
        assert_eq!(Center::Middle.offset(), Vector2::new(0.0, 0.0));
    }
}
