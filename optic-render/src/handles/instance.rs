use optic_core::{OpticError, OpticErrorKind, OpticResult};
use cgmath::{InnerSpace, Matrix4, Vector3, Vector4};

use crate::asset::attr::{
    ATTRInfo, ATTRName, ColATTR, CustomATTR, DataType, Pos2DATTR, Pos3DATTR, Rot2DATTR, Rot3DATTR,
    Scale2DATTR, Scale3DATTR,
};

// ── Attribute helper for interleaved byte-level access ─────────────────────

/// Describes one attribute within the interleaved instance stride: its byte
/// offset within one instance's data, its byte size (elem_count * byte_count),
/// and its ATTRInfo for GL binding.
#[derive(Clone, Debug)]
struct AttrSlot {
    offset: usize,
    size: usize,
    info: ATTRInfo,
}

fn build_slots(
    pos: Option<&Pos3DATTR>,
    rot: Option<&Rot3DATTR>,
    scale: Option<&Scale3DATTR>,
    col: Option<&ColATTR>,
    custom: &[CustomATTR],
) -> Vec<AttrSlot> {
    let mut slots = Vec::new();
    let mut offset = 0usize;

    let mut push = |info: ATTRInfo| {
        let size = info.elem_count * info.byte_count;
        slots.push(AttrSlot { offset, size, info });
        offset += size;
    };

    if let Some(a) = pos {
        if !a.is_empty() {
            push(a.info.clone());
        }
    }
    if let Some(a) = rot {
        if !a.is_empty() {
            push(a.info.clone());
        }
    }
    if let Some(a) = scale {
        if !a.is_empty() {
            push(a.info.clone());
        }
    }
    if let Some(a) = col {
        if !a.is_empty() {
            push(a.info.clone());
        }
    }
    for c in custom {
        push(c.info.clone());
    }

    slots
}

fn interleave(slots: &[AttrSlot], attrs: &[&[u8]], count: usize) -> Vec<u8> {
    let stride: usize = slots.iter().map(|s| s.size).sum();
    let mut buf = vec![0u8; count * stride];
    for i in 0..count {
        for (slot, data) in slots.iter().zip(attrs.iter()) {
            let start = i * slot.size;
            let end = start + slot.size;
            let dst = i * stride + slot.offset;
            buf[dst..dst + slot.size].copy_from_slice(&data[start..end]);
        }
    }
    buf
}

// ── InstanceKind ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub(crate) struct CustomSlot {
    pub name: String,
    pub byte_offset: usize,
    pub byte_size: usize,
    pub typ: optic_core::ATTRType,
    pub elem_count: u32,
}

#[derive(Clone, Debug)]
pub(crate) struct InstanceKind {
    pub has_pos: bool,
    pub has_rot: bool,
    pub has_scale: bool,
    pub has_col: bool,
    pub custom_offsets: Vec<CustomSlot>,
}

// ── InstanceDesc3D ────────────────────────────────────────────────────────

pub struct InstanceDesc3D {
    pub pos_attr: Pos3DATTR,
    pub rot_attr: Rot3DATTR,
    pub scale_attr: Scale3DATTR,
    pub col_attr: ColATTR,
    pub cus_attrs: Vec<CustomATTR>,
}

impl InstanceDesc3D {
    pub fn empty() -> Self {
        Self {
            pos_attr: Pos3DATTR::empty(),
            rot_attr: Rot3DATTR::empty(),
            scale_attr: Scale3DATTR::empty(),
            col_attr: ColATTR::empty(),
            cus_attrs: Vec::new(),
        }
    }

    pub fn from_positions(positions: &[Vector3<f32>]) -> Self {
        let mut desc = Self::empty();
        for p in positions {
            desc.pos_attr.push([p.x, p.y, p.z]);
        }
        desc
    }

    pub fn from_transforms(transforms: &[Matrix4<f32>]) -> Self {
        let mut desc = Self::empty();
        for m in transforms {
            // Extract translation
            desc.pos_attr.push([m[3][0], m[3][1], m[3][2]]);

            // Extract scale: lengths of the upper-left 3x3 column vectors
            let sx = Vector3::new(m[0][0], m[1][0], m[2][0]).magnitude();
            let sy = Vector3::new(m[0][1], m[1][1], m[2][1]).magnitude();
            let sz = Vector3::new(m[0][2], m[1][2], m[2][2]).magnitude();
            desc.scale_attr.push([sx, sy, sz]);

            // Extract rotation quaternion from the 3x3 rotation matrix (assuming no shear)
            let r00 = m[0][0] / sx;
            let r01 = m[0][1] / sy;
            let r02 = m[0][2] / sz;
            let r10 = m[1][0] / sx;
            let r11 = m[1][1] / sy;
            let r12 = m[1][2] / sz;
            let r20 = m[2][0] / sx;
            let r21 = m[2][1] / sy;
            let r22 = m[2][2] / sz;

            let trace = r00 + r11 + r22;
            if trace > 0.0 {
                let s = (trace + 1.0).sqrt() * 2.0;
                desc.rot_attr.push([(r21 - r12) / s, (r02 - r20) / s, (r10 - r01) / s, s / 4.0]);
            } else if r00 > r11 && r00 > r22 {
                let s = (1.0 + r00 - r11 - r22).sqrt() * 2.0;
                desc.rot_attr.push([s / 4.0, (r01 + r10) / s, (r02 + r20) / s, (r21 - r12) / s]);
            } else if r11 > r22 {
                let s = (1.0 + r11 - r00 - r22).sqrt() * 2.0;
                desc.rot_attr.push([(r01 + r10) / s, s / 4.0, (r12 + r21) / s, (r02 - r20) / s]);
            } else {
                let s = (1.0 + r22 - r00 - r11).sqrt() * 2.0;
                desc.rot_attr.push([(r02 + r20) / s, (r12 + r21) / s, s / 4.0, (r10 - r01) / s]);
            }
        }
        desc
    }

    pub fn attach_custom_attr(&mut self, attr: CustomATTR) -> &mut Self {
        self.cus_attrs.push(attr);
        self
    }

    pub fn ship(&self) -> OpticResult<InstanceBuffer> {
        // Collect non-empty default attrs
        let count = self.resolve_count();
        let has_any_attr = self.pos_attr.is_empty()
            && self.rot_attr.is_empty()
            && self.scale_attr.is_empty()
            && self.col_attr.is_empty()
            && self.cus_attrs.is_empty();

        if has_any_attr {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "cannot ship an instance buffer with zero attributes populated",
            ));
        }

        // Verify all non-empty attrs have the same count
        self.verify_counts()?;

        // Check custom attr name collisions
        self.verify_custom_names()?;

        // Build slots and interleave
        let slots = build_slots(
            Some(&self.pos_attr),
            Some(&self.rot_attr),
            Some(&self.scale_attr),
            Some(&self.col_attr),
            &self.cus_attrs,
        );

        let stride: usize = slots.iter().map(|s| s.size).sum();
        let instance_count = count.unwrap_or(0);

        // Collect raw byte slices for each non-empty attr
        let mut raw: Vec<&[u8]> = Vec::new();
        if !self.pos_attr.is_empty() {
            raw.push(self.pos_attr.data.as_bytes());
        }
        if !self.rot_attr.is_empty() {
            raw.push(self.rot_attr.data.as_bytes());
        }
        if !self.scale_attr.is_empty() {
            raw.push(self.scale_attr.data.as_bytes());
        }
        if !self.col_attr.is_empty() {
            raw.push(self.col_attr.data.as_bytes());
        }
        for c in &self.cus_attrs {
            raw.push(&c.data);
        }

        let cpu_mirror = if instance_count > 0 {
            interleave(&slots, &raw, instance_count)
        } else {
            Vec::new()
        };

        let layouts: Vec<(ATTRInfo, u32)> = slots.iter().enumerate().map(|(i, s)| (s.info.clone(), i as u32)).collect();

        // Build InstanceKind
        let mut custom_offsets = Vec::new();
        for c in &self.cus_attrs {
            let off = slots.iter()
                .position(|s| s.info.name == c.info.name)
                .map(|idx| slots[idx].offset)
                .unwrap_or(0);
            custom_offsets.push(CustomSlot {
                name: match &c.info.name { ATTRName::Custom(n) => n.clone(), _ => String::new() },
                byte_offset: off,
                byte_size: c.info.elem_count * c.info.byte_count,
                typ: c.info.typ.clone(),
                elem_count: c.info.elem_count as u32,
            });
        }

        let kind = InstanceKind {
            has_pos: !self.pos_attr.is_empty(),
            has_rot: !self.rot_attr.is_empty(),
            has_scale: !self.scale_attr.is_empty(),
            has_col: !self.col_attr.is_empty(),
            custom_offsets,
        };

        // Create GL buffer
        let buf_id = create_instance_buffer();
        if !cpu_mirror.is_empty() {
            upload_instance_data(buf_id, &cpu_mirror);
        }

        let capacity = if instance_count > 0 { instance_count as u32 } else { 0 };

        Ok(InstanceBuffer {
            buf_id,
            capacity,
            count: instance_count as u32,
            stride: stride as u32,
            layouts,
            cpu_mirror,
            kind,
        })
    }

    fn resolve_count(&self) -> Option<usize> {
        for attr in [&self.pos_attr.data as &dyn AsCount, &self.rot_attr.data, &self.scale_attr.data, &self.col_attr.data] {
            if !attr.is_empty() {
                return Some(attr.len());
            }
        }
        for c in &self.cus_attrs {
            if !c.is_empty() {
                let elem_size = c.info.elem_count * c.info.byte_count;
                return Some(c.data.len() / elem_size);
            }
        }
        None
    }

    fn verify_counts(&self) -> OpticResult<()> {
        let count = match self.resolve_count() {
            Some(c) => c,
            None => return Ok(()), // no attrs at all — caught by has_any_attr check above
        };

        macro_rules! check {
            ($attr:expr, $name:expr) => {
                if !$attr.is_empty() && $attr.data.len() != count {
                    return Err(OpticError::new(
                        OpticErrorKind::Custom,
                        &format!(
                            "instance attribute count mismatch: {} has {} elements, expected {}",
                            $name,
                            $attr.data.len(),
                            count
                        ),
                    ));
                }
            };
        }
        check!(self.pos_attr, "pos_attr");
        check!(self.rot_attr, "rot_attr");
        check!(self.scale_attr, "scale_attr");
        check!(self.col_attr, "col_attr");

        for c in &self.cus_attrs {
            let elem_size = c.info.elem_count * c.info.byte_count;
            let c_count = if elem_size > 0 { c.data.len() / elem_size } else { 0 };
            if c_count != count {
                let name = match &c.info.name { ATTRName::Custom(n) => n.clone(), _ => "unknown".into() };
                return Err(OpticError::new(
                    OpticErrorKind::Custom,
                    &format!(
                        "instance attribute count mismatch: custom attr \"{name}\" has {c_count} elements, expected {count}"
                    ),
                ));
            }
        }

        Ok(())
    }

    fn verify_custom_names(&self) -> OpticResult<()> {
        let reserved = ["iPos", "iRot", "iScale", "iColor"];
        for c in &self.cus_attrs {
            let name = match &c.info.name {
                ATTRName::Custom(n) => n.as_str(),
                _ => continue,
            };
            if reserved.contains(&name) {
                return Err(OpticError::new(
                    OpticErrorKind::Custom,
                    &format!("custom attribute name \"{name}\" collides with reserved instance attribute name"),
                ));
            }
        }
        // Check for duplicates among custom attrs
        for i in 0..self.cus_attrs.len() {
            for j in i + 1..self.cus_attrs.len() {
                let ni = match &self.cus_attrs[i].info.name { ATTRName::Custom(n) => n, _ => continue };
                let nj = match &self.cus_attrs[j].info.name { ATTRName::Custom(n) => n, _ => continue };
                if ni == nj {
                    return Err(OpticError::new(
                        OpticErrorKind::Custom,
                        &format!("duplicate custom attribute name \"{ni}\""),
                    ));
                }
            }
        }
        Ok(())
    }
}

// ── InstanceDesc2D ────────────────────────────────────────────────────────

pub struct InstanceDesc2D {
    pub pos_attr: Pos2DATTR,
    pub rot_attr: Rot2DATTR,
    pub scale_attr: Scale2DATTR,
    pub col_attr: ColATTR,
    pub cus_attrs: Vec<CustomATTR>,
}

impl InstanceDesc2D {
    pub fn empty() -> Self {
        Self {
            pos_attr: Pos2DATTR::empty(),
            rot_attr: Rot2DATTR::empty(),
            scale_attr: Scale2DATTR::empty(),
            col_attr: ColATTR::empty(),
            cus_attrs: Vec::new(),
        }
    }

    pub fn attach_custom_attr(&mut self, attr: CustomATTR) -> &mut Self {
        self.cus_attrs.push(attr);
        self
    }

    pub fn ship(&self) -> OpticResult<InstanceBuffer> {
        let has_any_attr = self.pos_attr.is_empty()
            && self.rot_attr.is_empty()
            && self.scale_attr.is_empty()
            && self.col_attr.is_empty()
            && self.cus_attrs.is_empty();

        if has_any_attr {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "cannot ship an instance buffer with zero attributes populated",
            ));
        }

        // Build slots (2D variant — uses the 2D types directly)
        let mut slots = Vec::new();
        let mut offset = 0usize;

        let mut push = |info: ATTRInfo| {
            let size = info.elem_count * info.byte_count;
            slots.push(AttrSlot { offset, size, info: info.clone() });
            offset += size;
        };

        if !self.pos_attr.is_empty() { push(self.pos_attr.info.clone()); }
        if !self.rot_attr.is_empty() { push(self.rot_attr.info.clone()); }
        if !self.scale_attr.is_empty() { push(self.scale_attr.info.clone()); }
        if !self.col_attr.is_empty() { push(self.col_attr.info.clone()); }
        for c in &self.cus_attrs { push(c.info.clone()); }

        let stride = offset;
        let count = self.resolve_count();
        let instance_count = count.unwrap_or(0);

        // Collect raw byte slices
        let mut raw: Vec<&[u8]> = Vec::new();
        if !self.pos_attr.is_empty() { raw.push(self.pos_attr.data.as_bytes()); }
        if !self.rot_attr.is_empty() { raw.push(self.rot_attr.data.as_bytes()); }
        if !self.scale_attr.is_empty() { raw.push(self.scale_attr.data.as_bytes()); }
        if !self.col_attr.is_empty() { raw.push(self.col_attr.data.as_bytes()); }
        for c in &self.cus_attrs { raw.push(&c.data); }

        let cpu_mirror = if instance_count > 0 {
            interleave(&slots, &raw, instance_count)
        } else {
            Vec::new()
        };

        let layouts: Vec<(ATTRInfo, u32)> = slots.iter().enumerate().map(|(i, s)| (s.info.clone(), i as u32)).collect();

        // Build InstanceKind
        let mut custom_offsets = Vec::new();
        for c in &self.cus_attrs {
            let off = slots.iter()
                .position(|s| s.info.name == c.info.name)
                .map(|idx| slots[idx].offset)
                .unwrap_or(0);
            custom_offsets.push(CustomSlot {
                name: match &c.info.name { ATTRName::Custom(n) => n.clone(), _ => String::new() },
                byte_offset: off,
                byte_size: c.info.elem_count * c.info.byte_count,
                typ: c.info.typ.clone(),
                elem_count: c.info.elem_count as u32,
            });
        }

        let kind = InstanceKind {
            has_pos: !self.pos_attr.is_empty(),
            has_rot: !self.rot_attr.is_empty(),
            has_scale: !self.scale_attr.is_empty(),
            has_col: !self.col_attr.is_empty(),
            custom_offsets,
        };

        let buf_id = create_instance_buffer();
        if !cpu_mirror.is_empty() {
            upload_instance_data(buf_id, &cpu_mirror);
        }

        let capacity = if instance_count > 0 { instance_count as u32 } else { 0 };

        Ok(InstanceBuffer {
            buf_id,
            capacity,
            count: instance_count as u32,
            stride: stride as u32,
            layouts,
            cpu_mirror,
            kind,
        })
    }

    fn resolve_count(&self) -> Option<usize> {
        for attr in [&self.pos_attr.data as &dyn AsCount, &self.col_attr.data] {
            if !attr.is_empty() {
                return Some(attr.len());
            }
        }
        for c in &self.cus_attrs {
            if !c.is_empty() {
                let elem_size = c.info.elem_count * c.info.byte_count;
                return Some(c.data.len() / elem_size);
            }
        }
        None
    }
}

// ── InstanceBuffer ────────────────────────────────────────────────────────

pub struct InstanceBuffer {
    pub(crate) buf_id: u32,
    pub(crate) capacity: u32,
    pub(crate) count: u32,
    pub(crate) stride: u32,
    pub layouts: Vec<(ATTRInfo, u32)>,
    pub(crate) cpu_mirror: Vec<u8>,
    pub(crate) kind: InstanceKind,
}

impl InstanceBuffer {
    pub fn count(&self) -> u32 { self.count }
    pub fn capacity(&self) -> u32 { self.capacity }

    pub fn update_instance<D: DataType>(&mut self, index: u32, attr_index: usize, value: D) -> OpticResult<()> {
        if index >= self.count {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("instance index {index} out of bounds (count: {})", self.count)));
        }
        if attr_index >= self.layouts.len() {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("attr index {attr_index} out of bounds (layout count: {})", self.layouts.len())));
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
        let off = index as usize * self.stride as usize + self.compute_attr_offset(attr_index);
        let size = slot_info.elem_count * slot_info.byte_count;

        if bytes.len() != size {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!(
                "value byte size {} does not match attribute size {}", bytes.len(), size
            )));
        }

        // Write to CPU mirror
        self.cpu_mirror[off..off + size].copy_from_slice(&bytes);

        // Upload just this instance
        subfill_instance_data(self.buf_id, off, &bytes);

        Ok(())
    }

    pub fn get_instance<D: DataType>(&self, index: u32, attr_index: usize) -> OpticResult<D> {
        if index >= self.count {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("instance index {index} out of bounds (count: {})", self.count)));
        }
        if attr_index >= self.layouts.len() {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("attr index {attr_index} out of bounds (layout count: {})", self.layouts.len())));
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

        let off = index as usize * self.stride as usize + self.compute_attr_offset(attr_index);
        let size = slot_info.elem_count * slot_info.byte_count;
        let raw = &self.cpu_mirror[off..off + size];

        // Reconstruct D from raw bytes
        let d = deserialize::<D>(raw);
        Ok(d)
    }

    pub fn update_custom<D: DataType>(&mut self, index: u32, name: &str, value: D) -> OpticResult<()> {
        let slot = self.kind.custom_offsets.iter().find(|s| s.name == name)
            .ok_or_else(|| OpticError::new(OpticErrorKind::Custom, &format!("custom attribute \"{name}\" not found")))?;

        if slot.byte_size != D::BYTE_COUNT || slot.elem_count as usize != D::ELEM_COUNT || slot.typ != D::ATTR_FORMAT {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!(
                    "type mismatch for custom attribute \"{name}\": expected {:?}[{}], got {:?}[{}]",
                    slot.typ, slot.elem_count, D::ATTR_FORMAT, D::ELEM_COUNT,
                ),
            ));
        }

        let bytes = value.u8ify();
        let off = index as usize * self.stride as usize + slot.byte_offset;

        self.cpu_mirror[off..off + slot.byte_size].copy_from_slice(&bytes);
        subfill_instance_data(self.buf_id, off, &bytes);

        Ok(())
    }

    pub fn get_custom<D: DataType>(&self, index: u32, name: &str) -> OpticResult<D> {
        let slot = self.kind.custom_offsets.iter().find(|s| s.name == name)
            .ok_or_else(|| OpticError::new(OpticErrorKind::Custom, &format!("custom attribute \"{name}\" not found")))?;

        if slot.byte_size != D::BYTE_COUNT || slot.elem_count as usize != D::ELEM_COUNT || slot.typ != D::ATTR_FORMAT {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!(
                    "type mismatch for custom attribute \"{name}\": expected {:?}[{}], got {:?}[{}]",
                    slot.typ, slot.elem_count, D::ATTR_FORMAT, D::ELEM_COUNT,
                ),
            ));
        }

        let off = index as usize * self.stride as usize + slot.byte_offset;
        let raw = &self.cpu_mirror[off..off + slot.byte_size];
        Ok(deserialize::<D>(raw))
    }

    pub fn set_position(&mut self, index: u32, pos: Vector3<f32>) -> OpticResult<()> {
        if !self.kind.has_pos {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no position attribute"));
        }
        let attr_index = 0; // pos is always first if present
        self.update_instance(index, attr_index, [pos.x, pos.y, pos.z])
    }

    pub fn get_position(&self, index: u32) -> OpticResult<Vector3<f32>> {
        if !self.kind.has_pos {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no position attribute"));
        }
        let arr: [f32; 3] = self.get_instance(index, 0)?;
        Ok(Vector3::new(arr[0], arr[1], arr[2]))
    }

    pub fn set_rotation(&mut self, index: u32, rot: Vector4<f32>) -> OpticResult<()> {
        if !self.kind.has_rot {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no rotation attribute"));
        }
        let attr_index = if self.kind.has_pos { 1 } else { 0 };
        self.update_instance(index, attr_index, [rot.x, rot.y, rot.z, rot.w])
    }

    pub fn get_rotation(&self, index: u32) -> OpticResult<Vector4<f32>> {
        if !self.kind.has_rot {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no rotation attribute"));
        }
        let attr_index = if self.kind.has_pos { 1 } else { 0 };
        let arr: [f32; 4] = self.get_instance(index, attr_index)?;
        Ok(Vector4::new(arr[0], arr[1], arr[2], arr[3]))
    }

    pub fn set_scale(&mut self, index: u32, scale: Vector3<f32>) -> OpticResult<()> {
        if !self.kind.has_scale {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no scale attribute"));
        }
        let attr_index = if self.kind.has_pos { 1 } else { 0 };
        let attr_index = if self.kind.has_rot { attr_index + 1 } else { attr_index };
        self.update_instance(index, attr_index, [scale.x, scale.y, scale.z])
    }

    pub fn get_scale(&self, index: u32) -> OpticResult<Vector3<f32>> {
        if !self.kind.has_scale {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no scale attribute"));
        }
        let mut attr_index = 0u32;
        if self.kind.has_pos { attr_index += 1; }
        if self.kind.has_rot { attr_index += 1; }
        // attr_index is now at scale (always after pos/rot if present)
        let actual_idx = attr_index as usize;
        let arr: [f32; 3] = self.get_instance(index, actual_idx)?;
        Ok(Vector3::new(arr[0], arr[1], arr[2]))
    }

    pub fn set_color(&mut self, index: u32, color: optic_core::RGBA) -> OpticResult<()> {
        if !self.kind.has_col {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no color attribute"));
        }
        let rgba = [color.0, color.1, color.2, color.3];
        let mut attr_index = 0u32;
        if self.kind.has_pos { attr_index += 1; }
        if self.kind.has_rot { attr_index += 1; }
        if self.kind.has_scale { attr_index += 1; }
        self.update_instance(index, attr_index as usize, rgba)
    }

    pub fn get_color(&self, index: u32) -> OpticResult<optic_core::RGBA> {
        if !self.kind.has_col {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no color attribute"));
        }
        let mut attr_index = 0u32;
        if self.kind.has_pos { attr_index += 1; }
        if self.kind.has_rot { attr_index += 1; }
        if self.kind.has_scale { attr_index += 1; }
        let rgba: [f32; 4] = self.get_instance(index, attr_index as usize)?;
        Ok(optic_core::RGBA(rgba[0], rgba[1], rgba[2], rgba[3]))
    }

    // ── Growth / shrink ──────────────────────────────────────────────────────

    pub fn set_instance_count(&mut self, new_count: u32) {
        if new_count > self.capacity {
            // Grow with amortized doubling
            let new_cap = new_count.max(self.capacity * 2);
            self.reserve_internal(new_cap);
        }
        if new_count > self.count {
            // Fill new slots with defaults
            let old_count = self.count as usize;
            let new_count_usize = new_count as usize;
            let stride = self.stride as usize;
            self.cpu_mirror.resize(new_count_usize * stride, 0u8);
            // Set defaults for new slots: pos=(0,0,0), rot=(0,0,0,1), scale=(1,1,1), col=white
            let default_slot = self.make_default_instance_bytes();
            for i in old_count..new_count_usize {
                let off = i * stride;
                self.cpu_mirror[off..off + stride].copy_from_slice(&default_slot);
            }
        }
        self.count = new_count;
        upload_instance_data(self.buf_id, &self.cpu_mirror);
    }

    pub fn reserve(&mut self, additional: u32) {
        let needed = self.count + additional;
        if needed > self.capacity {
            let new_cap = needed.max(self.capacity * 2);
            self.reserve_internal(new_cap);
        }
    }

    pub fn shrink_to_fit(&mut self) {
        if self.count < self.capacity {
            let new_cap = self.count;
            self.capacity = new_cap;
            realloc_instance_buffer(self.buf_id, self.cpu_mirror.len());
        }
    }

    pub fn push_raw(&mut self, bytes: &[u8]) -> OpticResult<u32> {
        if bytes.len() != self.stride as usize {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!("push_raw byte count {} does not match instance stride {}", bytes.len(), self.stride),
            ));
        }
        let idx = self.count;
        self.set_instance_count(self.count + 1);
        let off = idx as usize * self.stride as usize;
        self.cpu_mirror[off..off + self.stride as usize].copy_from_slice(bytes);
        subfill_instance_data(self.buf_id, off, bytes);
        Ok(idx)
    }

    pub fn remove_instance(&mut self, index: u32) -> OpticResult<()> {
        if index >= self.count {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("remove index {index} out of bounds (count: {})", self.count)));
        }
        let last = self.count - 1;
        if index != last {
            let stride = self.stride as usize;
            let dst = index as usize * stride;
            let src = last as usize * stride;
            let len = stride;
            self.cpu_mirror.copy_within(src..src + len, dst);
            subfill_instance_data(self.buf_id, dst, &self.cpu_mirror[dst..dst + len]);
        }
        self.count = last;
        Ok(())
    }

    pub fn remove_instance_ordered(&mut self, index: u32) -> OpticResult<()> {
        if index >= self.count {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("remove index {index} out of bounds (count: {})", self.count)));
        }
        let stride = self.stride as usize;
        let dst = index as usize * stride;
        let end = (self.count - 1) as usize * stride;
        let len = end - dst;
        if len > 0 {
            self.cpu_mirror.copy_within(dst + stride..end + stride, dst);
        }
        self.count -= 1;
        upload_instance_data(self.buf_id, &self.cpu_mirror[..self.count as usize * stride]);
        Ok(())
    }

    // ── Whole-buffer / ranged updates ────────────────────────────────────────

    pub fn write_all(&mut self, desc: &InstanceDesc3D) -> OpticResult<()> {
        // Re-ship to validate, then take the data
        let new_buf = desc.ship()?;
        self.buf_id = new_buf.buf_id;
        self.capacity = new_buf.capacity;
        self.count = new_buf.count;
        self.stride = new_buf.stride;
        self.layouts = new_buf.layouts;
        self.cpu_mirror = new_buf.cpu_mirror;
        self.kind = new_buf.kind;
        Ok(())
    }

    pub fn write_range(&mut self, start: u32, bytes: &[u8]) -> OpticResult<()> {
        let stride = self.stride as usize;
        if bytes.len() % stride != 0 {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                &format!("write_range byte count {} is not a multiple of stride {}", bytes.len(), stride),
            ));
        }
        let instance_count = bytes.len() / stride;
        if start + instance_count as u32 > self.count {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "write_range extends past the current instance count",
            ));
        }
        let off = start as usize * stride;
        self.cpu_mirror[off..off + bytes.len()].copy_from_slice(bytes);
        subfill_instance_data(self.buf_id, off, bytes);
        Ok(())
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    fn compute_attr_offset(&self, attr_index: usize) -> usize {
        let mut offset = 0usize;
        for i in 0..attr_index {
            let si = &self.layouts[i].0;
            offset += si.elem_count * si.byte_count;
        }
        offset
    }

    fn reserve_internal(&mut self, new_cap: u32) {
        let old_size = self.cpu_mirror.len();
        let new_size = new_cap as usize * self.stride as usize;
        self.cpu_mirror.resize(new_size, 0u8);
        // Fill the newly added capacity with defaults
        let stride = self.stride as usize;
        let default_slot = self.make_default_instance_bytes();
        for i in (old_size / stride)..new_cap as usize {
            let off = i * stride;
            self.cpu_mirror[off..off + stride].copy_from_slice(&default_slot);
        }
        self.capacity = new_cap;
        realloc_instance_buffer(self.buf_id, new_size);
        upload_instance_data(self.buf_id, &self.cpu_mirror);
    }

    fn make_default_instance_bytes(&self) -> Vec<u8> {
        let stride = self.stride as usize;
        let mut bytes = vec![0u8; stride];
        if self.kind.has_pos {
            // pos = (0,0,0) is already zero
        }
        if self.kind.has_rot {
            let off = if self.kind.has_pos { 12 } else { 0 }; // 3 * 4 bytes
            // quaternion identity = (0,0,0,1)
            bytes[off + 12..off + 16].copy_from_slice(&1.0f32.to_le_bytes());
        }
        if self.kind.has_scale {
            let mut off = 0usize;
            if self.kind.has_pos { off += 12; }
            if self.kind.has_rot { off += 16; }
            // scale = (1,1,1)
            bytes[off..off + 4].copy_from_slice(&1.0f32.to_le_bytes());
            bytes[off + 4..off + 8].copy_from_slice(&1.0f32.to_le_bytes());
            bytes[off + 8..off + 12].copy_from_slice(&1.0f32.to_le_bytes());
        }
        if self.kind.has_col {
            let mut off = 0usize;
            if self.kind.has_pos { off += 12; }
            if self.kind.has_rot { off += 16; }
            if self.kind.has_scale { off += 12; }
            // color = white (1,1,1,1)
            for i in 0..4 {
                bytes[off + i * 4..off + (i + 1) * 4].copy_from_slice(&1.0f32.to_le_bytes());
            }
        }
        bytes
    }
}

// ── GL helpers ─────────────────────────────────────────────────────────────

fn create_instance_buffer() -> u32 {
    let mut id = 0u32;
    unsafe { gl::GenBuffers(1, &mut id); }
    id
}

fn upload_instance_data(id: u32, data: &[u8]) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            data.len() as gl::types::GLsizeiptr,
            data.as_ptr() as *const std::ffi::c_void,
            gl::DYNAMIC_DRAW,
        );
    }
}

fn subfill_instance_data(id: u32, offset: usize, data: &[u8]) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, id);
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            offset as isize,
            data.len() as isize,
            data.as_ptr() as *const std::ffi::c_void,
        );
    }
}

fn realloc_instance_buffer(id: u32, size: usize) {
    unsafe {
        gl::BindBuffer(gl::ARRAY_BUFFER, id);
        gl::BufferData(
            gl::ARRAY_BUFFER,
            size as gl::types::GLsizeiptr,
            std::ptr::null(),
            gl::DYNAMIC_DRAW,
        );
    }
}

// ── Count helper trait ─────────────────────────────────────────────────────

trait AsCount {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
}

impl<T> AsCount for Vec<T> {
    fn len(&self) -> usize { self.len() }
    fn is_empty(&self) -> bool { self.is_empty() }
}

// If CustomAttr doesn't implement AsCount naturally (it uses Vec<u8>), we implement it:
impl AsCount for CustomATTR {
    fn len(&self) -> usize {
        if self.info.elem_count == 0 || self.info.byte_count == 0 { return 0; }
        self.data.len() / (self.info.elem_count * self.info.byte_count)
    }
    fn is_empty(&self) -> bool { self.data.is_empty() }
}

// ── Deserialize helper ─────────────────────────────────────────────────────

fn deserialize<D: DataType>(bytes: &[u8]) -> D {
    // DataType is implemented for scalars (f32, u32, etc.) and arrays ([f32; 3], etc.)
    // We need to reconstruct D from its raw bytes using the same layout as u8ify.

    // Use std::mem::transmute-like approach via from_ne_bytes / from_raw_parts
    // D is either a scalar or a fixed-size array.
    unsafe {
        let ptr = bytes.as_ptr() as *const D;
        std::ptr::read_unaligned(ptr)
    }
}

// ── Raw byte access for typed attrs ────────────────────────────────────────

trait AsBytes {
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for Vec<[f32; 3]> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 12) }
    }
}

impl AsBytes for Vec<[f32; 4]> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 16) }
    }
}

impl AsBytes for Vec<[f32; 2]> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 8) }
    }
}

impl AsBytes for Vec<f32> {
    fn as_bytes(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.as_ptr() as *const u8, self.len() * 4) }
    }
}
