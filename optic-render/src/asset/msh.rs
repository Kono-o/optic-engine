use optic_core::consts::{ASSET_TYPE_MESH, CACHE_VERSION, OPTIC_MAGIC};
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

    pub fn from_stl_src(data: &[u8]) -> OpticResult<Self> {
        let mut pos_attr = Pos3DATTR::empty();
        let mut col_attr = ColATTR::empty();
        let mut uvm_attr = UVMATTR::empty();
        let mut nrm_attr = NrmATTR::empty();
        let mut ind_attr = IndATTR::empty();

        let def_col = [1.0, 1.0, 1.0, 1.0];
        let def_uvm = [0.0, 0.0];
        let mut unique_verts: HashMap<(u32, u32, u32, u32, u32, u32), u32> = HashMap::new();

        // Closure to deduplicate and push a vertex
        let push_vert = |pos: [f32; 3], nrm: [f32; 3], unique: &mut HashMap<(u32, u32, u32, u32, u32, u32), u32>,
                              pos_attr: &mut Pos3DATTR, nrm_attr: &mut NrmATTR,
                              col_attr: &mut ColATTR, uvm_attr: &mut UVMATTR| -> u32 {
            let key = (pos[0].to_bits(), pos[1].to_bits(), pos[2].to_bits(),
                       nrm[0].to_bits(), nrm[1].to_bits(), nrm[2].to_bits());
            if let Some(&idx) = unique.get(&key) {
                idx
            } else {
                let idx = pos_attr.data.len() as u32;
                unique.insert(key, idx);
                pos_attr.push(pos);
                nrm_attr.push(nrm);
                col_attr.push(def_col);
                uvm_attr.push(def_uvm);
                idx
            }
        };

        // Detect ASCII vs binary: binary STL never starts with "solid " (it's 80-byte header)
        let is_ascii = data.len() >= 6 && &data[0..6] == b"solid ";

        if is_ascii {
            let text = std::str::from_utf8(data)
                .map_err(|_| OpticError::new(OpticErrorKind::Asset, "STL file is not valid UTF-8"))?;
            let mut nrm = [0.0f32; 3];
            let mut tri_verts = Vec::new();

            for line in text.lines() {
                let line = line.trim();
                if line.starts_with("facet normal") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        nrm = [
                            parts[2].parse().unwrap_or(0.0),
                            parts[3].parse().unwrap_or(0.0),
                            parts[4].parse().unwrap_or(0.0),
                        ];
                    }
                    tri_verts.clear();
                } else if line.starts_with("vertex") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        tri_verts.push([
                            parts[1].parse().unwrap_or(0.0),
                            parts[2].parse().unwrap_or(0.0),
                            parts[3].parse().unwrap_or(0.0),
                        ]);
                    }
                } else if line.starts_with("endfacet") && tri_verts.len() == 3 {
                    for v in &tri_verts {
                        let idx = push_vert(*v, nrm, &mut unique_verts, &mut pos_attr, &mut nrm_attr, &mut col_attr, &mut uvm_attr);
                        ind_attr.push(idx);
                    }
                }
            }
        } else {
            if data.len() < 84 {
                return Err(OpticError::new(OpticErrorKind::Asset, &format!("binary STL too short: {} bytes", data.len())));
            }
            let tri_count = u32::from_le_bytes([data[80], data[81], data[82], data[83]]) as usize;
            if data.len() < 84 + tri_count * 50 {
                return Err(OpticError::new(OpticErrorKind::Asset, &format!("binary STL truncated: expected {} triangles, got {} bytes", tri_count, data.len())));
            }
            for i in 0..tri_count {
                let base = 84 + i * 50;
                let nrm = [
                    f32::from_le_bytes([data[base], data[base + 1], data[base + 2], data[base + 3]]),
                    f32::from_le_bytes([data[base + 4], data[base + 5], data[base + 6], data[base + 7]]),
                    f32::from_le_bytes([data[base + 8], data[base + 9], data[base + 10], data[base + 11]]),
                ];
                let verts = [
                    [f32::from_le_bytes([data[base + 12], data[base + 13], data[base + 14], data[base + 15]]),
                     f32::from_le_bytes([data[base + 16], data[base + 17], data[base + 18], data[base + 19]]),
                     f32::from_le_bytes([data[base + 20], data[base + 21], data[base + 22], data[base + 23]])],
                    [f32::from_le_bytes([data[base + 24], data[base + 25], data[base + 26], data[base + 27]]),
                     f32::from_le_bytes([data[base + 28], data[base + 29], data[base + 30], data[base + 31]]),
                     f32::from_le_bytes([data[base + 32], data[base + 33], data[base + 34], data[base + 35]])],
                    [f32::from_le_bytes([data[base + 36], data[base + 37], data[base + 38], data[base + 39]]),
                     f32::from_le_bytes([data[base + 40], data[base + 41], data[base + 42], data[base + 43]]),
                     f32::from_le_bytes([data[base + 44], data[base + 45], data[base + 46], data[base + 47]])],
                ];
                for v in &verts {
                    let idx = push_vert(*v, nrm, &mut unique_verts, &mut pos_attr, &mut nrm_attr, &mut col_attr, &mut uvm_attr);
                    ind_attr.push(idx);
                }
            }
        }

        Ok(Self { pos_attr, col_attr, uvm_attr, nrm_attr, ind_attr, cus_attrs: Vec::new() })
    }

    // --- from_disk: debug loads source + overwrites cache; release loads cache only ---
    #[cfg(debug_assertions)]
    pub fn from_disk(path: &str) -> OpticResult<Self> {
        let ext = optic_file::extension(path).unwrap_or_default();
        let mesh = match ext.as_str() {
            "obj" => {
                let src = optic_file::read_string(path)?;
                Self::from_obj_src(&src)?
            }
            "stl" => {
                let data = optic_file::read_bytes(path)?;
                Self::from_stl_src(&data)?
            }
            _ => return Err(OpticError::new(OpticErrorKind::Asset, &format!("unsupported mesh format: .{ext}"))),
        };
        let cache = optic_file::cached_path(path, "omesh");
        mesh.save_cached(&cache)?;
        Ok(mesh)
    }

    #[cfg(not(debug_assertions))]
    pub fn from_disk(path: &str) -> OpticResult<Self> {
        let cache = optic_file::cached_path(path, "omesh");
        Self::from_cached(&cache)
    }

    pub fn save_cached(&self, path: &str) -> OpticResult<()> {
        let has_normals = !self.nrm_attr.data.is_empty();
        let has_uvs = !self.uvm_attr.data.is_empty();
        let flags = (has_normals as u8) | ((has_uvs as u8) << 1);

        let pos_bytes = self.pos_attr.data.len() * 12;
        let nrm_bytes = self.nrm_attr.data.len() * 12;
        let uvm_bytes = self.uvm_attr.data.len() * 8;
        let col_bytes = self.col_attr.data.len() * 16;
        let ind_bytes = self.ind_attr.data.len() * 4;

        let size = 19 + 20 + pos_bytes + nrm_bytes + uvm_bytes + col_bytes + ind_bytes;
        let mut data = Vec::with_capacity(size);
        data.extend_from_slice(&OPTIC_MAGIC);
        data.push(CACHE_VERSION);
        data.push(ASSET_TYPE_MESH);
        data.push(flags);

        // Position (required)
        data.extend_from_slice(&(pos_bytes as u32).to_le_bytes());
        for v in &self.pos_attr.data {
            data.extend_from_slice(&v[0].to_le_bytes());
            data.extend_from_slice(&v[1].to_le_bytes());
            data.extend_from_slice(&v[2].to_le_bytes());
        }

        // Normals (optional)
        data.extend_from_slice(&(nrm_bytes as u32).to_le_bytes());
        for v in &self.nrm_attr.data {
            data.extend_from_slice(&v[0].to_le_bytes());
            data.extend_from_slice(&v[1].to_le_bytes());
            data.extend_from_slice(&v[2].to_le_bytes());
        }

        // UVs (optional)
        data.extend_from_slice(&(uvm_bytes as u32).to_le_bytes());
        for v in &self.uvm_attr.data {
            data.extend_from_slice(&v[0].to_le_bytes());
            data.extend_from_slice(&v[1].to_le_bytes());
        }

        // Colors (always present)
        data.extend_from_slice(&(col_bytes as u32).to_le_bytes());
        for v in &self.col_attr.data {
            data.extend_from_slice(&v[0].to_le_bytes());
            data.extend_from_slice(&v[1].to_le_bytes());
            data.extend_from_slice(&v[2].to_le_bytes());
            data.extend_from_slice(&v[3].to_le_bytes());
        }

        // Indices (always present)
        data.extend_from_slice(&(ind_bytes as u32).to_le_bytes());
        for v in &self.ind_attr.data {
            data.extend_from_slice(&v.to_le_bytes());
        }

        optic_file::write_bytes(path, &data)
    }

    #[cfg_attr(debug_assertions, allow(dead_code))]
    fn from_cached(path: &str) -> OpticResult<Self> {
        let data = optic_file::read_bytes(path)?;
        if data.len() < 24 {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("cached mesh too short: {path}")));
        }
        if data[0..17] != OPTIC_MAGIC {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("invalid optic magic in cached mesh: {path}")));
        }
        if data[17] != CACHE_VERSION {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("unsupported cache version {} in {path}", data[17])));
        }
        if data[18] != ASSET_TYPE_MESH {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("type mismatch: expected mesh in {path}")));
        }

        let mut off = 20usize;

        let read_f32x3 = |off: &mut usize, data: &[u8]| -> [f32; 3] {
            let x = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            let y = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            let z = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            [x, y, z]
        };

        let read_f32x2 = |off: &mut usize, data: &[u8]| -> [f32; 2] {
            let x = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            let y = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            [x, y]
        };

        let read_f32x4 = |off: &mut usize, data: &[u8]| -> [f32; 4] {
            let x = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            let y = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            let z = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            let w = f32::from_le_bytes([data[*off], data[*off + 1], data[*off + 2], data[*off + 3]]); *off += 4;
            [x, y, z, w]
        };

        // Position
        let pos_size = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        off += 4;
        if off + pos_size > data.len() {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached mesh (position): {path}")));
        }
        let vert_count = pos_size / 12;
        let mut pos_attr = Pos3DATTR::empty();
        for _ in 0..vert_count {
            pos_attr.push(read_f32x3(&mut off, &data));
        }

        // Normals
        let nrm_size = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        off += 4;
        let mut nrm_attr = NrmATTR::empty();
        if nrm_size > 0 {
            if off + nrm_size > data.len() {
                return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached mesh (normals): {path}")));
            }
            let nrm_count = nrm_size / 12;
            for _ in 0..nrm_count {
                nrm_attr.push(read_f32x3(&mut off, &data));
            }
        }

        // UVs
        let uvm_size = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        off += 4;
        let mut uvm_attr = UVMATTR::empty();
        if uvm_size > 0 {
            if off + uvm_size > data.len() {
                return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached mesh (UVs): {path}")));
            }
            let uvm_count = uvm_size / 8;
            for _ in 0..uvm_count {
                uvm_attr.push(read_f32x2(&mut off, &data));
            }
        }

        // Colors
        let col_size = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        off += 4;
        if off + col_size > data.len() {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached mesh (colors): {path}")));
        }
        let mut col_attr = ColATTR::empty();
        let col_count = col_size / 16;
        for _ in 0..col_count {
            col_attr.push(read_f32x4(&mut off, &data));
        }

        // Indices
        let ind_size = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]) as usize;
        off += 4;
        if off + ind_size > data.len() {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("truncated cached mesh (indices): {path}")));
        }
        let mut ind_attr = IndATTR::empty();
        let ind_count = ind_size / 4;
        for _ in 0..ind_count {
            let idx = u32::from_le_bytes([data[off], data[off + 1], data[off + 2], data[off + 3]]);
            off += 4;
            ind_attr.push(idx);
        }

        Ok(Self { pos_attr, col_attr, uvm_attr, nrm_attr, ind_attr, cus_attrs: Vec::new() })
    }

    pub fn cube(side: f32) -> Self {
        Self::cuboid(side, side, side)
    }

    pub fn cuboid(w: f32, h: f32, d: f32) -> Self {
        let mut mesh = Self::empty();
        let hw = w * 0.5;
        let hh = h * 0.5;
        let hd = d * 0.5;
        let faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
            ([0.0, 0.0, 1.0], [[-hw, -hh, hd], [hw, -hh, hd], [hw, hh, hd], [-hw, hh, hd]]),
            ([0.0, 0.0, -1.0], [[hw, -hh, -hd], [-hw, -hh, -hd], [-hw, hh, -hd], [hw, hh, -hd]]),
            ([0.0, 1.0, 0.0], [[-hw, hh, hd], [hw, hh, hd], [hw, hh, -hd], [-hw, hh, -hd]]),
            ([0.0, -1.0, 0.0], [[-hw, -hh, -hd], [hw, -hh, -hd], [hw, -hh, hd], [-hw, -hh, hd]]),
            ([1.0, 0.0, 0.0], [[hw, -hh, hd], [hw, -hh, -hd], [hw, hh, -hd], [hw, hh, hd]]),
            ([-1.0, 0.0, 0.0], [[-hw, -hh, -hd], [-hw, -hh, hd], [-hw, hh, hd], [-hw, hh, -hd]]),
        ];
        for (nrm, verts) in &faces {
            let base = mesh.pos_attr.data.len() as u32;
            for v in verts {
                mesh.pos_attr.push(*v);
                mesh.nrm_attr.push(*nrm);
            }
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.uvm_attr.push([0.0, 0.0]);
            mesh.uvm_attr.push([1.0, 0.0]);
            mesh.uvm_attr.push([1.0, 1.0]);
            mesh.uvm_attr.push([0.0, 1.0]);
            mesh.ind_attr.push(base);
            mesh.ind_attr.push(base + 1);
            mesh.ind_attr.push(base + 2);
            mesh.ind_attr.push(base);
            mesh.ind_attr.push(base + 2);
            mesh.ind_attr.push(base + 3);
        }
        mesh
    }

    pub fn sphere(radius: f32, stacks: u32, sectors: u32) -> Self {
        let mut mesh = Self::empty();
        let pi = std::f32::consts::PI;
        for i in 0..=stacks {
            let phi = pi * i as f32 / stacks as f32;
            for j in 0..=sectors {
                let theta = std::f32::consts::TAU * j as f32 / sectors as f32;
                let x = phi.sin() * theta.cos();
                let y = phi.cos();
                let z = phi.sin() * theta.sin();
                mesh.pos_attr.push([radius * x, radius * y, radius * z]);
                mesh.nrm_attr.push([x, y, z]);
                mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
                mesh.uvm_attr.push([j as f32 / sectors as f32, i as f32 / stacks as f32]);
            }
        }
        for i in 0..stacks {
            for j in 0..sectors {
                let first = i * (sectors + 1) + j;
                let second = first + sectors + 1;
                mesh.ind_attr.push(first);
                mesh.ind_attr.push(second);
                mesh.ind_attr.push(first + 1);
                mesh.ind_attr.push(second);
                mesh.ind_attr.push(second + 1);
                mesh.ind_attr.push(first + 1);
            }
        }
        mesh
    }

    pub fn cylinder(radius: f32, height: f32, segments: u32, cap: bool) -> Self {
        let mut mesh = Self::empty();
        let hh = height * 0.5;
        for i in 0..=segments {
            let a = std::f32::consts::TAU * i as f32 / segments as f32;
            let (s, c) = a.sin_cos();
            mesh.pos_attr.push([radius * c, hh, radius * s]);
            mesh.nrm_attr.push([c, 0.0, s]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.uvm_attr.push([i as f32 / segments as f32, 1.0]);
        }
        for i in 0..=segments {
            let a = std::f32::consts::TAU * i as f32 / segments as f32;
            let (s, c) = a.sin_cos();
            mesh.pos_attr.push([radius * c, -hh, radius * s]);
            mesh.nrm_attr.push([c, 0.0, s]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.uvm_attr.push([i as f32 / segments as f32, 0.0]);
        }
        for i in 0..segments {
            let t = i;
            let b = segments + 1 + i;
            mesh.ind_attr.push(t);
            mesh.ind_attr.push(b);
            mesh.ind_attr.push(t + 1);
            mesh.ind_attr.push(b);
            mesh.ind_attr.push(b + 1);
            mesh.ind_attr.push(t + 1);
        }
        if cap {
            let top_center = mesh.pos_attr.data.len() as u32;
            mesh.pos_attr.push([0.0, hh, 0.0]);
            mesh.nrm_attr.push([0.0, 1.0, 0.0]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.uvm_attr.push([0.5, 0.5]);
            for i in 0..segments {
                mesh.ind_attr.push(top_center);
                mesh.ind_attr.push(i);
                mesh.ind_attr.push(i + 1);
            }
            let bot_center = mesh.pos_attr.data.len() as u32;
            mesh.pos_attr.push([0.0, -hh, 0.0]);
            mesh.nrm_attr.push([0.0, -1.0, 0.0]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.uvm_attr.push([0.5, 0.5]);
            for i in 0..segments {
                let b = segments + 1 + i;
                mesh.ind_attr.push(bot_center);
                mesh.ind_attr.push(b + 1);
                mesh.ind_attr.push(b);
            }
        }
        mesh
    }

    pub fn cone(radius: f32, height: f32, segments: u32, cap: bool) -> Self {
        let mut mesh = Self::empty();
        let hh = height * 0.5;
        mesh.pos_attr.push([0.0, hh, 0.0]);
        mesh.nrm_attr.push([0.0, 1.0, 0.0]);
        mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
        mesh.uvm_attr.push([0.5, 1.0]);
        for i in 0..=segments {
            let a = std::f32::consts::TAU * i as f32 / segments as f32;
            let (s, c) = a.sin_cos();
            let nx = c;
            let nz = s;
            let ny = radius / height;
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            mesh.pos_attr.push([radius * c, -hh, radius * s]);
            mesh.nrm_attr.push([nx / len, ny / len, nz / len]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.uvm_attr.push([i as f32 / segments as f32, 0.0]);
        }
        for i in 0..segments {
            mesh.ind_attr.push(0);
            mesh.ind_attr.push(i + 1);
            mesh.ind_attr.push(i + 2);
        }
        if cap {
            let center = mesh.pos_attr.data.len() as u32;
            mesh.pos_attr.push([0.0, -hh, 0.0]);
            mesh.nrm_attr.push([0.0, -1.0, 0.0]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
            mesh.uvm_attr.push([0.5, 0.5]);
            for i in 0..segments {
                mesh.ind_attr.push(center);
                mesh.ind_attr.push(center + 1 + i + 1);
                mesh.ind_attr.push(center + 1 + i);
            }
        }
        mesh
    }

    pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> Self {
        let mut mesh = Self::empty();
        for i in 0..=major_segments {
            let u = std::f32::consts::TAU * i as f32 / major_segments as f32;
            for j in 0..=minor_segments {
                let v = std::f32::consts::TAU * j as f32 / minor_segments as f32;
                let x = (major_radius + minor_radius * v.cos()) * u.cos();
                let y = minor_radius * v.sin();
                let z = (major_radius + minor_radius * v.cos()) * u.sin();
                mesh.pos_attr.push([x, y, z]);
                let nx = v.cos() * u.cos();
                let ny = v.sin();
                let nz = v.cos() * u.sin();
                mesh.nrm_attr.push([nx, ny, nz]);
                mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
                mesh.uvm_attr.push([
                    i as f32 / major_segments as f32,
                    j as f32 / minor_segments as f32,
                ]);
            }
        }
        let stride = minor_segments + 1;
        for i in 0..major_segments {
            for j in 0..minor_segments {
                let first = i * stride + j;
                let second = first + stride;
                mesh.ind_attr.push(first);
                mesh.ind_attr.push(second);
                mesh.ind_attr.push(first + 1);
                mesh.ind_attr.push(second);
                mesh.ind_attr.push(second + 1);
                mesh.ind_attr.push(first + 1);
            }
        }
        mesh
    }

    pub fn plane(width: f32, depth: f32) -> Self {
        let mut mesh = Self::empty();
        let hw = width * 0.5;
        let hd = depth * 0.5;
        mesh.pos_attr.push([-hw, 0.0, hd]);
        mesh.pos_attr.push([hw, 0.0, hd]);
        mesh.pos_attr.push([hw, 0.0, -hd]);
        mesh.pos_attr.push([-hw, 0.0, -hd]);
        for _ in 0..4 {
            mesh.nrm_attr.push([0.0, 1.0, 0.0]);
            mesh.col_attr.push([1.0, 1.0, 1.0, 1.0]);
        }
        mesh.uvm_attr.push([0.0, 0.0]);
        mesh.uvm_attr.push([1.0, 0.0]);
        mesh.uvm_attr.push([1.0, 1.0]);
        mesh.uvm_attr.push([0.0, 1.0]);
        mesh.ind_attr.push(0);
        mesh.ind_attr.push(1);
        mesh.ind_attr.push(2);
        mesh.ind_attr.push(0);
        mesh.ind_attr.push(2);
        mesh.ind_attr.push(3);
        mesh
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

    pub fn fullscreen_quad() -> Self {
        let mut mesh = Self::empty();
        mesh.aspect = 1.0;
        mesh.set_pos_attr(Pos2DATTR::from_array(&[
            [-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0],
        ]));
        mesh.set_col_attr(ColATTR::from_array(&[[1.0, 1.0, 1.0, 1.0]; 4]));
        mesh.set_uvm_attr(UVMATTR::from_array(&[
            [0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0],
        ]));
        mesh.set_ind_attr(IndATTR::from_array(&[0, 1, 2, 0, 2, 3]));
        mesh
    }

    pub fn attach_custom_attr(&mut self, attr: CustomATTR) {
        self.cus_attrs.push(attr);
    }

    pub fn circle(radius: f32, segments: u32) -> Self {
        let mut mesh = Self::empty();
        let mut pos = vec![[0.0f32, 0.0f32]];
        let mut uvm = vec![[0.5f32, 0.5f32]];
        for i in 0..segments {
            let a = std::f32::consts::TAU * i as f32 / segments as f32;
            pos.push([radius * a.cos(), radius * a.sin()]);
            uvm.push([0.5 + 0.5 * a.cos(), 0.5 + 0.5 * a.sin()]);
        }
        let mut ind = Vec::new();
        for i in 0..segments {
            ind.push(0);
            ind.push(i + 1);
            ind.push(if i + 1 < segments { i + 2 } else { 1 });
        }
        let vert_count = (segments + 1) as usize;
        mesh.set_pos_attr(Pos2DATTR::from(pos));
        mesh.set_col_attr(ColATTR::from(vec![[1.0f32; 4]; vert_count]));
        mesh.set_uvm_attr(UVMATTR::from(uvm));
        mesh.set_ind_attr(IndATTR::from(ind));
        mesh
    }

    pub fn polygon(radius: f32, sides: u32) -> Self {
        Self::circle(radius, sides)
    }

    pub fn ring(inner_radius: f32, outer_radius: f32, segments: u32) -> Self {
        let mut mesh = Self::empty();
        let mut pos = Vec::new();
        let mut uvm = Vec::new();
        for i in 0..segments {
            let a = std::f32::consts::TAU * i as f32 / segments as f32;
            let (s, c) = a.sin_cos();
            pos.push([outer_radius * c, outer_radius * s]);
            uvm.push([0.5 + 0.5 * c, 0.5 + 0.5 * s]);
        }
        for i in 0..segments {
            let a = std::f32::consts::TAU * i as f32 / segments as f32;
            let (s, c) = a.sin_cos();
            pos.push([inner_radius * c, inner_radius * s]);
            uvm.push([0.5 + 0.5 * c * inner_radius / outer_radius, 0.5 + 0.5 * s * inner_radius / outer_radius]);
        }
        let mut ind = Vec::new();
        for i in 0..segments {
            let next = (i + 1) % segments;
            let o1 = i;
            let o2 = next;
            let i1 = segments + i;
            let i2 = segments + next;
            ind.push(o1);
            ind.push(o2);
            ind.push(i1);
            ind.push(i1);
            ind.push(o2);
            ind.push(i2);
        }
        let vert_count = (segments * 2) as usize;
        mesh.set_pos_attr(Pos2DATTR::from(pos));
        mesh.set_col_attr(ColATTR::from(vec![[1.0f32; 4]; vert_count]));
        mesh.set_uvm_attr(UVMATTR::from(uvm));
        mesh.set_ind_attr(IndATTR::from(ind));
        mesh
    }

    pub fn rect(width: f32, height: f32) -> Self {
        let mut mesh = Self::empty();
        let x = width * 0.5;
        let y = height * 0.5;
        mesh.set_pos_attr(Pos2DATTR::from_array(&[[-x, y], [x, y], [x, -y], [-x, -y]]));
        mesh.set_col_attr(ColATTR::from_array(&[[1.0, 1.0, 1.0, 1.0]; 4]));
        mesh.set_uvm_attr(UVMATTR::from_array(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]));
        mesh.set_ind_attr(IndATTR::from_array(&[0, 2, 1, 2, 0, 3]));
        mesh
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
    crate::GL::bind_buffer(buf_id);
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

    if !mesh.ind_attr.is_empty() {
        has_indices = true;
        ind_data = &mesh.ind_attr.data;
        ind_count = ind_data.len() as u32;
        fill_index_buffer(ind_id, ind_data);

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
    crate::GL::bind_buffer(buf_id);
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

    if !mesh.ind_attr.is_empty() {
        has_indices = true;
        ind_data = &mesh.ind_attr.data;
        ind_count = ind_data.len() as u32;
        fill_index_buffer(ind_id, ind_data);

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
    fn mesh3d_cached_roundtrip() {
        let obj = "v 0.0 0.0 0.0\nv 1.0 0.0 0.0\nv 0.0 1.0 0.0\nf 1 2 3";
        let mesh = Mesh3DFile::from_obj_src(obj).unwrap();
        let path = "/tmp/optic_test_mesh3d_cache.omesh";
        mesh.save_cached(path).unwrap();
        let loaded = Mesh3DFile::from_cached(path).unwrap();
        assert_eq!(loaded.pos_attr.data.len(), mesh.pos_attr.data.len());
        assert_eq!(loaded.ind_attr.data.len(), mesh.ind_attr.data.len());
        assert_eq!(loaded.pos_attr.data, mesh.pos_attr.data);
        assert_eq!(loaded.ind_attr.data, mesh.ind_attr.data);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn mesh3d_from_stl_ascii() {
        let stl = "solid cube\n\
            facet normal 0.0 0.0 1.0\n\
                outer loop\n\
                    vertex -1.0 -1.0 1.0\n\
                    vertex 1.0 -1.0 1.0\n\
                    vertex 1.0 1.0 1.0\n\
                endloop\n\
            endfacet\n\
            endsolid cube\n";
        let mesh = Mesh3DFile::from_stl_src(stl.as_bytes()).unwrap();
        assert_eq!(mesh.pos_attr.data.len(), 3);
        assert_eq!(mesh.ind_attr.data.len(), 3);
        assert_eq!(mesh.nrm_attr.data.len(), 3);
        assert!((mesh.nrm_attr.data[0][2] - 1.0).abs() < f32::EPSILON);
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

    #[test]
    fn mesh2d_circle() {
        let m = Mesh2DFile::circle(1.0, 8);
        assert_eq!(m.pos_attr.data.len(), 9);
        assert_eq!(m.col_attr.data.len(), 9);
        assert_eq!(m.uvm_attr.data.len(), 9);
        assert_eq!(m.ind_attr.data.len(), 24);
    }

    #[test]
    fn mesh2d_polygon() {
        let m = Mesh2DFile::polygon(1.0, 6);
        assert_eq!(m.pos_attr.data.len(), 7);
        assert_eq!(m.ind_attr.data.len(), 18);
    }

    #[test]
    fn mesh2d_ring() {
        let m = Mesh2DFile::ring(0.5, 1.0, 8);
        assert_eq!(m.pos_attr.data.len(), 16);
        assert_eq!(m.ind_attr.data.len(), 48);
    }

    #[test]
    fn mesh2d_rect() {
        let m = Mesh2DFile::rect(2.0, 3.0);
        assert_eq!(m.pos_attr.data.len(), 4);
        assert_eq!(m.ind_attr.data.len(), 6);
        assert!((m.pos_attr.data[0][0] - (-1.0)).abs() < f32::EPSILON);
        assert!((m.pos_attr.data[0][1] - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn mesh3d_cube() {
        let m = Mesh3DFile::cube(2.0);
        assert_eq!(m.pos_attr.data.len(), 24);
        assert_eq!(m.nrm_attr.data.len(), 24);
        assert_eq!(m.ind_attr.data.len(), 36);
    }

    #[test]
    fn mesh3d_cuboid() {
        let m = Mesh3DFile::cuboid(1.0, 2.0, 3.0);
        assert_eq!(m.pos_attr.data.len(), 24);
        assert_eq!(m.ind_attr.data.len(), 36);
    }

    #[test]
    fn mesh3d_sphere() {
        let m = Mesh3DFile::sphere(1.0, 8, 16);
        let verts = (8 + 1) * (16 + 1);
        assert_eq!(m.pos_attr.data.len(), verts);
        assert_eq!(m.nrm_attr.data.len(), verts);
        assert_eq!(m.ind_attr.data.len(), 8 * 16 * 6);
    }

    #[test]
    fn mesh3d_cylinder_with_caps() {
        let m = Mesh3DFile::cylinder(0.5, 2.0, 16, true);
        let body_verts = (16 + 1) * 2;
        let cap_verts = 2;
        assert_eq!(m.pos_attr.data.len(), body_verts + cap_verts);
    }

    #[test]
    fn mesh3d_cylinder_no_caps() {
        let m = Mesh3DFile::cylinder(0.5, 2.0, 16, false);
        assert_eq!(m.pos_attr.data.len(), (16 + 1) * 2);
    }

    #[test]
    fn mesh3d_cone_with_cap() {
        let m = Mesh3DFile::cone(0.5, 2.0, 16, true);
        let body_verts = 1 + (16 + 1);
        let cap_verts = 1;
        assert_eq!(m.pos_attr.data.len(), body_verts + cap_verts);
    }

    #[test]
    fn mesh3d_cone_no_cap() {
        let m = Mesh3DFile::cone(0.5, 2.0, 16, false);
        assert_eq!(m.pos_attr.data.len(), 1 + (16 + 1));
    }

    #[test]
    fn mesh3d_torus() {
        let m = Mesh3DFile::torus(1.0, 0.3, 12, 8);
        let verts = (12 + 1) * (8 + 1);
        assert_eq!(m.pos_attr.data.len(), verts);
        assert_eq!(m.nrm_attr.data.len(), verts);
        assert_eq!(m.ind_attr.data.len(), 12 * 8 * 6);
    }

    #[test]
    fn mesh3d_plane() {
        let m = Mesh3DFile::plane(2.0, 3.0);
        assert_eq!(m.pos_attr.data.len(), 4);
        assert_eq!(m.nrm_attr.data.len(), 4);
        assert_eq!(m.ind_attr.data.len(), 6);
        for nrm in &m.nrm_attr.data {
            assert!((nrm[1] - 1.0).abs() < f32::EPSILON);
        }
    }
}
