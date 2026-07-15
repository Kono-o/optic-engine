use optic_core::{OpticError, OpticErrorKind, OpticResult};
use cgmath::{InnerSpace, Matrix4, Vector3, Vector4};

use crate::asset::attr::{
    ATTRInfo, ATTRName, ColorATTR, CustomATTR, DataType, Pos2DATTR, Pos3DATTR, Rot2DATTR, Rot3DATTR,
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
    col: Option<&ColorATTR>,
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

/// Builder for constructing 3D instance buffers from positions, transforms, or matrices.
///
/// Collects per-instance attribute data (position, rotation, scale, colour, and custom
/// attributes) on the CPU side and interleaves them into a single GPU buffer when
/// [`upload`](InstanceDesc3D::upload) is called. Use this to prepare instanced draws of
/// [`Mesh3D`] — for example, rendering thousands of objects with a single draw call.
///
/// # Attribute layout
///
/// Attributes appear in this fixed order within the interleaved stride:
///
/// 1. `pos` — 3 × `f32` (12 bytes)
/// 2. `rot` — 4 × `f32` quaternion (16 bytes)
/// 3. `scale` — 3 × `f32` (12 bytes)
/// 4. `col` — 4 × `f32` RGBA (16 bytes)
/// 5. custom attributes, in insertion order
///
/// Any of these may be omitted (left empty). The stride shrinks accordingly.
///
/// # Example
///
/// ```
/// # use optic_render::handles::InstanceDesc3D;
/// # use cgmath::Vector3;
/// let mut desc = InstanceDesc3D::empty();
///
/// for i in 0..100 {
///     desc.pos_attr.push([i as f32 * 2.0, 0.0, 0.0]);
///     desc.col_attr.push([1.0, 0.0, 0.0, 1.0]);
/// }
///
/// // desc.upload() transfers the interleaved data to a GPU buffer.
/// ```
pub struct InstanceDesc3D {
    pub pos_attr: Pos3DATTR,
    pub rot_attr: Rot3DATTR,
    pub scale_attr: Scale3DATTR,
    pub col_attr: ColorATTR,
    pub custom_attrs: Vec<CustomATTR>,
}

impl InstanceDesc3D {
    /// Creates an empty descriptor with no attributes.
    ///
    /// Push per-instance data into the individual attribute fields before calling
    /// [`upload`](InstanceDesc3D::upload).
    pub fn empty() -> Self {
        Self {
            pos_attr: Pos3DATTR::empty(),
            rot_attr: Rot3DATTR::empty(),
            scale_attr: Scale3DATTR::empty(),
            col_attr: ColorATTR::empty(),
            custom_attrs: Vec::new(),
        }
    }

    /// Builds a descriptor from an array of 3D positions.
    ///
    /// All other attribute fields remain empty.
    pub fn from_positions(positions: &[Vector3<f32>]) -> Self {
        let mut desc = Self::empty();
        for p in positions {
            desc.pos_attr.push([p.x, p.y, p.z]);
        }
        desc
    }

    /// Decomposes a slice of 4×4 transformation matrices into position, rotation
    /// (quaternion), and scale attributes.
    ///
    /// This is a convenience constructor for users who already have transform
    /// matrices. It assumes no shear and extracts a unit quaternion from the
    /// upper-left 3×3 sub-matrix.
    ///
    /// # Panics
    ///
    /// May produce degenerate quaternions when a scale component is zero.
    pub fn from_transforms(transforms: &[Matrix4<f32>]) -> Self {
        let mut desc = Self::empty();
        for m in transforms {
            desc.pos_attr.push([m[3][0], m[3][1], m[3][2]]);

            let sx = Vector3::new(m[0][0], m[1][0], m[2][0]).magnitude();
            let sy = Vector3::new(m[0][1], m[1][1], m[2][1]).magnitude();
            let sz = Vector3::new(m[0][2], m[1][2], m[2][2]).magnitude();
            desc.scale_attr.push([sx, sy, sz]);

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

    /// Appends a custom (user-defined) attribute.
    ///
    /// Custom attribute names must not collide with the reserved names `iPos`,
    /// `iRot`, `iScale`, or `iColor`, and must be unique among themselves.
    ///
    /// Returns `&mut self` for chaining.
    pub fn add_custom_attr(&mut self, attr: CustomATTR) -> &mut Self {
        self.custom_attrs.push(attr);
        self
    }

    /// Interleaves all non-empty attributes and uploads them to a new GPU buffer.
    ///
    /// Returns an [`InstanceBuffer`] ready for use in instanced draws.
    ///
    /// # Errors
    ///
    /// - Returns an error if all attributes are empty (no data to upload).
    /// - Returns an error if non-empty attributes have mismatched element counts.
    /// - Returns an error if custom attribute names collide with reserved names
    ///   or each other.
    pub fn upload(&self) -> OpticResult<InstanceBuffer> {
        let count = self.resolve_count();
        let has_any_attr = self.pos_attr.is_empty()
            && self.rot_attr.is_empty()
            && self.scale_attr.is_empty()
            && self.col_attr.is_empty()
            && self.custom_attrs.is_empty();

        if has_any_attr {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "cannot upload an instance buffer with zero attributes populated",
            ));
        }

        self.verify_counts()?;
        self.verify_custom_names()?;

        let slots = build_slots(
            Some(&self.pos_attr),
            Some(&self.rot_attr),
            Some(&self.scale_attr),
            Some(&self.col_attr),
            &self.custom_attrs,
        );

        let stride: usize = slots.iter().map(|s| s.size).sum();
        let instance_count = count.unwrap_or(0);

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
        for c in &self.custom_attrs {
            raw.push(&c.data);
        }

        let cpu_mirror = if instance_count > 0 {
            interleave(&slots, &raw, instance_count)
        } else {
            Vec::new()
        };

        let layouts: Vec<(ATTRInfo, u32)> = slots.iter().enumerate().map(|(i, s)| (s.info.clone(), i as u32)).collect();

        let mut custom_offsets = Vec::new();
        for c in &self.custom_attrs {
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
        for attr in [&self.pos_attr.data as &dyn AsCount, &self.rot_attr.data, &self.scale_attr.data, &self.col_attr.data] {
            if !attr.is_empty() {
                return Some(attr.len());
            }
        }
        for c in &self.custom_attrs {
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
            None => return Ok(()),
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

        for c in &self.custom_attrs {
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
        for c in &self.custom_attrs {
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
        for i in 0..self.custom_attrs.len() {
            for j in i + 1..self.custom_attrs.len() {
                let ni = match &self.custom_attrs[i].info.name { ATTRName::Custom(n) => n, _ => continue };
                let nj = match &self.custom_attrs[j].info.name { ATTRName::Custom(n) => n, _ => continue };
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

/// Builder for constructing 2D instance buffers from per-instance attributes.
///
/// Like [`InstanceDesc3D`], but uses 2-element vectors for position and scale and a
/// single-precision scalar for rotation (angle in radians). Use this to prepare
/// instanced draws of [`Mesh2D`](crate::handles::Mesh2D) for efficient batch rendering
/// of 2D sprites, tiles, or UI elements.
///
/// # Attribute layout
///
/// 1. `pos` — 2 × `f32` (8 bytes)
/// 2. `rot` — 1 × `f32` angle (4 bytes)
/// 3. `scale` — 2 × `f32` (8 bytes)
/// 4. `col` — 4 × `f32` RGBA (16 bytes)
/// 5. custom attributes, in insertion order
pub struct InstanceDesc2D {
    pub pos_attr: Pos2DATTR,
    pub rot_attr: Rot2DATTR,
    pub scale_attr: Scale2DATTR,
    pub col_attr: ColorATTR,
    pub custom_attrs: Vec<CustomATTR>,
}

impl InstanceDesc2D {
    /// Creates an empty 2D descriptor.
    pub fn empty() -> Self {
        Self {
            pos_attr: Pos2DATTR::empty(),
            rot_attr: Rot2DATTR::empty(),
            scale_attr: Scale2DATTR::empty(),
            col_attr: ColorATTR::empty(),
            custom_attrs: Vec::new(),
        }
    }

    /// Appends a custom attribute for chaining.
    pub fn add_custom_attr(&mut self, attr: CustomATTR) -> &mut Self {
        self.custom_attrs.push(attr);
        self
    }

    /// Interleaves all non-empty 2D attributes and uploads to a new GPU buffer.
    ///
    /// # Errors
    ///
    /// Same error conditions as [`InstanceDesc3D::upload`].
    pub fn upload(&self) -> OpticResult<InstanceBuffer> {
        let has_any_attr = self.pos_attr.is_empty()
            && self.rot_attr.is_empty()
            && self.scale_attr.is_empty()
            && self.col_attr.is_empty()
            && self.custom_attrs.is_empty();

        if has_any_attr {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "cannot upload an instance buffer with zero attributes populated",
            ));
        }

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
        for c in &self.custom_attrs { push(c.info.clone()); }

        let stride = offset;
        let count = self.resolve_count();
        let instance_count = count.unwrap_or(0);

        let mut raw: Vec<&[u8]> = Vec::new();
        if !self.pos_attr.is_empty() { raw.push(self.pos_attr.data.as_bytes()); }
        if !self.rot_attr.is_empty() { raw.push(self.rot_attr.data.as_bytes()); }
        if !self.scale_attr.is_empty() { raw.push(self.scale_attr.data.as_bytes()); }
        if !self.col_attr.is_empty() { raw.push(self.col_attr.data.as_bytes()); }
        for c in &self.custom_attrs { raw.push(&c.data); }

        let cpu_mirror = if instance_count > 0 {
            interleave(&slots, &raw, instance_count)
        } else {
            Vec::new()
        };

        let layouts: Vec<(ATTRInfo, u32)> = slots.iter().enumerate().map(|(i, s)| (s.info.clone(), i as u32)).collect();

        let mut custom_offsets = Vec::new();
        for c in &self.custom_attrs {
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
        for c in &self.custom_attrs {
            if !c.is_empty() {
                let elem_size = c.info.elem_count * c.info.byte_count;
                return Some(c.data.len() / elem_size);
            }
        }
        None
    }
}

// ── InstanceBuffer ────────────────────────────────────────────────────────

/// GPU buffer for instanced rendering, storing per-instance transforms, colors, and custom attributes with a CPU mirror for readback.
///
/// Created by [`InstanceDesc3D::upload`] or [`InstanceDesc2D::upload`]. Each instance is a
/// single packed row of interleaved attributes (position, rotation, scale, colour, and
/// custom data) designed for use with `glDrawArraysInstanced` / `glDrawElementsInstanced`.
/// The engine binds an `InstanceBuffer` to a [`MeshHandle`](crate::handles::MeshHandle) to
/// render thousands of objects in a single draw call.
///
/// # CPU mirror
///
/// An [`InstanceBuffer`] keeps a complete CPU copy of all instance data. This
/// enables reads and partial writes without a GPU round-trip. Every mutating
/// method writes through to both the CPU mirror and the GPU buffer
/// transparently.
///
/// | Read method | Source | Latency |
/// |---|---|---|
/// | [`position`](Self::position), [`color`](Self::color), [`instance`](Self::instance) | CPU mirror | Instant |
/// | _Read from GPU_ | Not supported | — |
///
/// # Growth
///
/// The buffer grows or shrinks in place via [`set_instance_count`](Self::set_instance_count). New slots
/// are filled with defaults:
///
/// | Attribute | Default |
/// |---|---|
/// | Position | `(0, 0, 0)` |
/// | Rotation | Identity quaternion `(0, 0, 0, 1)` |
/// | Scale | `(1, 1, 1)` |
/// | Colour | White `(1, 1, 1, 1)` |
///
/// # Convenience methods
///
/// | You want to… | Use |
/// |---|---|
/// | Move an instance | [`set_position`](Self::set_position) |
/// | Rotate an instance | [`set_rotation`](Self::set_rotation) |
/// | Scale an instance | [`set_scale`](Self::set_scale) |
/// | Recolour an instance | [`set_color`](Self::set_color) |
/// | Write raw bytes | [`update_instance`](Self::update_instance) |
///
/// # Example
///
/// ```ignore
/// use optic_render::handles::{InstanceBuffer, InstanceDesc3D};
/// use cgmath::Vector3;
///
/// // Create 100 red instances along the x-axis
/// let mut desc = InstanceDesc3D::empty();
/// for i in 0..100 {
///     desc.pos_attr.push([i as f32 * 2.0, 0.0, 0.0]);
///     desc.col_attr.push([1.0, 0.0, 0.0, 1.0]);
/// }
/// let mut buffer = desc.upload()?;
///
/// // Re-position instance 5
/// buffer.set_position(5, Vector3::new(20.0, 0.0, 0.0))?;
///
/// // Add 50 more instances at the end
/// buffer.set_instance_count(150);
/// ```
pub struct InstanceBuffer {
    pub(crate) buf_id: u32,
    pub(crate) capacity: u32,
    pub(crate) count: u32,
    pub(crate) stride: u32,
    pub(crate) layouts: Vec<(ATTRInfo, u32)>,
    pub(crate) cpu_mirror: Vec<u8>,
    pub(crate) kind: InstanceKind,
}

impl InstanceBuffer {
    /// Returns the number of active instances.
    pub fn count(&self) -> u32 { self.count }

    /// Returns the number of active instances as a `usize`.
    pub fn len(&self) -> usize { self.count as usize }

    /// Returns `true` if no instances are active.
    pub fn is_empty(&self) -> bool { self.count == 0 }

    /// Removes all instances without deallocating (O(1)).
    pub fn clear(&mut self) {
        self.count = 0;
    }

    /// Returns the total capacity (allocated slots, may be larger than count).
    pub fn capacity(&self) -> u32 { self.capacity }

    /// Returns the attribute layout descriptions for this buffer.
    pub fn layouts(&self) -> &[(ATTRInfo, u32)] { &self.layouts }

    /// Updates a single attribute of one instance by attribute index.
    ///
    /// This is the lowest-level update — it writes raw bytes into both the CPU
    /// mirror and the GPU buffer in one operation. The `attr_index` refers to
    /// the attribute's position in the interleaved layout (0 = first attribute).
    ///
    /// For convenience wrappers, see:
    ///
    /// | Attribute | Convenience method |
    /// |---|---|
    /// | Position (3D) | [`set_position`](Self::set_position) |
    /// | Rotation (3D) | [`set_rotation`](Self::set_rotation) |
    /// | Scale (3D) | [`set_scale`](Self::set_scale) |
    /// | Colour | [`set_color`](Self::set_color) |
    /// | Custom (by name) | [`update_custom`](Self::update_custom) |
    ///
    /// # Type safety
    ///
    /// `value` must be a [`DataType`] whose byte count, element count, and
    /// format exactly match the attribute's declared type. A mismatch produces
    /// a descriptive error at runtime.
    ///
    /// # Errors
    ///
    /// - `index` >= `count` — instance index out of bounds.
    /// - `attr_index` out of range — invalid attribute slot.
    /// - `D`'s type parameters do not match the attribute slot.
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

        self.cpu_mirror[off..off + size].copy_from_slice(&bytes);
        subfill_instance_data(self.buf_id, off, &bytes);

        Ok(())
    }

    /// Reads a single attribute of one instance from the CPU mirror.
    ///
    /// This is the lowest-level read. It copies bytes from the CPU mirror and
    /// deserialises them into `D`. The GPU buffer is **not** touched — once an
    /// instance buffer is uploaded, data flows from the CPU mirror to the GPU,
    /// never the other way.
    ///
    /// For convenience readers, see [`position`](Self::position),
    /// [`rotation`](Self::rotation), [`scale`](Self::scale),
    /// [`color`](Self::color), and [`custom_attr`](Self::custom_attr).
    ///
    /// # Errors
    ///
    /// Same as [`update_instance`](Self::update_instance).
    pub fn instance<D: DataType>(&self, index: u32, attr_index: usize) -> OpticResult<D> {
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

        let d = deserialize::<D>(raw);
        Ok(d)
    }

    /// Updates a custom attribute of one instance by name.
    ///
    /// Use this when you defined a custom attribute via
    /// [`InstanceDesc3D::add_custom_attr`] or
    /// [`InstanceDesc2D::add_custom_attr`] and upload with that descriptor.
    /// The attribute is looked up by name (not by index), making this robust
    /// against layout reordering.
    ///
    /// # Errors
    ///
    /// - No custom attribute with that name exists.
    /// - `D`'s type parameters do not match the attribute's declared format.
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

    /// Reads a custom attribute of one instance by name.
    ///
    /// Does **not** read back from the GPU.
    ///
    /// # Errors
    ///
    /// Same as [`update_custom`](InstanceBuffer::update_custom).
    pub fn custom_attr<D: DataType>(&self, index: u32, name: &str) -> OpticResult<D> {
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

    /// Sets the position of a single instance in world space.
    ///
    /// This is a typed convenience over [`update_instance`](Self::update_instance).
    /// It automatically resolves `attr_index = 0` and converts the `cgmath`
    /// vector to the raw `[f32; 3]` the GPU expects.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no position attribute (i.e. the
    /// descriptor that created it did not push to `pos_attr`).
    pub fn set_position(&mut self, index: u32, pos: Vector3<f32>) -> OpticResult<()> {
        if !self.kind.has_pos {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no position attribute"));
        }
        let attr_index = 0;
        self.update_instance(index, attr_index, [pos.x, pos.y, pos.z])
    }

    /// Returns the position of a single instance from the CPU mirror.
    ///
    /// The counterpart to [`set_position`](Self::set_position). Reads raw bytes
    /// from the CPU mirror and wraps them back into a `cgmath::Vector3`.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no position attribute.
    pub fn position(&self, index: u32) -> OpticResult<Vector3<f32>> {
        if !self.kind.has_pos {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no position attribute"));
        }
        let arr: [f32; 3] = self.instance(index, 0)?;
        Ok(Vector3::new(arr[0], arr[1], arr[2]))
    }

    /// Sets the rotation quaternion of a single instance.
    ///
    /// The quaternion is stored as `[x, y, z, w]` in the interleaved buffer.
    /// Use `cgmath::Quaternion::new(w, x, y, z)` to construct the value, then
    /// pass `.v` (a `Vector4`) or a raw `Vector4` to this method.
    ///
    /// # Attribute-index resolution
    ///
    /// The method skips past the position attribute if present:
    ///
    /// | Layout | attr_index passed to [`update_instance`](Self::update_instance) |
    /// |---|---|
    /// | Position + Rotation | 1 |
    /// | Rotation only | 0 |
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no rotation attribute.
    pub fn set_rotation(&mut self, index: u32, rot: Vector4<f32>) -> OpticResult<()> {
        if !self.kind.has_rot {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no rotation attribute"));
        }
        let attr_index = if self.kind.has_pos { 1 } else { 0 };
        self.update_instance(index, attr_index, [rot.x, rot.y, rot.z, rot.w])
    }

    /// Returns the rotation quaternion of a single instance.
    ///
    /// The counterpart to [`set_rotation`](Self::set_rotation). Uses the same
    /// attribute-index resolution logic to locate the correct slot.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no rotation attribute.
    pub fn rotation(&self, index: u32) -> OpticResult<Vector4<f32>> {
        if !self.kind.has_rot {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no rotation attribute"));
        }
        let attr_index = if self.kind.has_pos { 1 } else { 0 };
        let arr: [f32; 4] = self.instance(index, attr_index)?;
        Ok(Vector4::new(arr[0], arr[1], arr[2], arr[3]))
    }

    /// Sets the scale of a single instance.
    ///
    /// Each component is applied independently — use a uniform scale like
    /// `Vector3::new(2.0, 2.0, 2.0)` for isotropic scaling or vary components
    /// for non-uniform stretching.
    ///
    /// # Attribute-index resolution
    ///
    /// Skips past position and rotation attributes if present.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no scale attribute.
    pub fn set_scale(&mut self, index: u32, scale: Vector3<f32>) -> OpticResult<()> {
        if !self.kind.has_scale {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no scale attribute"));
        }
        let attr_index = if self.kind.has_pos { 1 } else { 0 };
        let attr_index = if self.kind.has_rot { attr_index + 1 } else { attr_index };
        self.update_instance(index, attr_index, [scale.x, scale.y, scale.z])
    }

    /// Returns the scale of a single instance.
    ///
    /// The counterpart to [`set_scale`](Self::set_scale). Uses the same
    /// attribute-index resolution logic.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no scale attribute.
    pub fn scale(&self, index: u32) -> OpticResult<Vector3<f32>> {
        if !self.kind.has_scale {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no scale attribute"));
        }
        let mut attr_index = 0u32;
        if self.kind.has_pos { attr_index += 1; }
        if self.kind.has_rot { attr_index += 1; }
        let actual_idx = attr_index as usize;
        let arr: [f32; 3] = self.instance(index, actual_idx)?;
        Ok(Vector3::new(arr[0], arr[1], arr[2]))
    }

    /// Sets the colour of a single instance.
    ///
    /// Accepts an [`optic_core::RGBA`] value constructed with
    /// [`RGBA::new(r, g, b, a)`](optic_core::RGBA::new) or from an integer hex
    /// like [`RGBA::from(0xFF8800FF)`](optic_core::RGBA#impl-From<u32>).
    ///
    /// # Attribute-index resolution
    ///
    /// Skips past position, rotation, and scale attributes if present.
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no colour attribute.
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

    /// Returns the colour of a single instance.
    ///
    /// The counterpart to [`set_color`](Self::set_color). Uses the same
    /// attribute-index resolution logic. Returns an [`optic_core::RGBA`].
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no colour attribute.
    pub fn color(&self, index: u32) -> OpticResult<optic_core::RGBA> {
        if !self.kind.has_col {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no color attribute"));
        }
        let mut attr_index = 0u32;
        if self.kind.has_pos { attr_index += 1; }
        if self.kind.has_rot { attr_index += 1; }
        if self.kind.has_scale { attr_index += 1; }
        let rgba: [f32; 4] = self.instance(index, attr_index as usize)?;
        Ok(optic_core::RGBA(rgba[0], rgba[1], rgba[2], rgba[3]))
    }

    // ── Growth / shrink ──────────────────────────────────────────────────────

    /// Resizes the instance count, filling new slots with defaults.
    ///
    /// Use this to add or remove instances at the end of the buffer without
    /// creating a new descriptor and re-uploading. New slots are filled with
    /// sensible defaults:
    ///
    /// | Attribute | Default |
    /// |---|---|
    /// | Position | `(0, 0, 0)` |
    /// | Rotation | Identity quaternion `(0, 0, 0, 1)` |
    /// | Scale | `(1, 1, 1)` |
    /// | Colour | White `(1, 1, 1, 1)` |
    ///
    /// # Capacity
    ///
    /// If `new_count` exceeds the current capacity the allocation doubles each
    /// time (amortized O(1) growth). If `new_count` is smaller, excess
    /// instances become inaccessible but memory is **not** reclaimed — call
    /// [`shrink_to_fit`](Self::shrink_to_fit) if the buffer is persistently
    /// oversized.
    ///
    /// # Example
    ///
    /// ```ignore
    /// # let mut buffer: InstanceBuffer = desc.upload()?;
    /// buffer.set_instance_count(200);  // grow from 100 to 200
    /// buffer.set_instance_count(50);   // shrink: last 150 are inaccessible
    /// buffer.shrink_to_fit();          // release GPU memory
    /// ```
    pub fn set_instance_count(&mut self, new_count: u32) {
        if new_count > self.capacity {
            let new_cap = new_count.max(self.capacity * 2);
            self.reserve_internal(new_cap);
        }
        if new_count > self.count {
            let old_count = self.count as usize;
            let new_count_usize = new_count as usize;
            let stride = self.stride as usize;
            self.cpu_mirror.resize(new_count_usize * stride, 0u8);
            let default_slot = self.make_default_instance_bytes();
            for i in old_count..new_count_usize {
                let off = i * stride;
                self.cpu_mirror[off..off + stride].copy_from_slice(&default_slot);
            }
        }
        self.count = new_count;
        upload_instance_data(self.buf_id, &self.cpu_mirror);
    }

    /// Reserves capacity for `additional` extra instances without changing the
    /// active count. Useful before a batch of [`push_raw`](Self::push_raw)
    /// calls to avoid repeated reallocations.
    pub fn reserve(&mut self, additional: u32) {
        let needed = self.count + additional;
        if needed > self.capacity {
            let new_cap = needed.max(self.capacity * 2);
            self.reserve_internal(new_cap);
        }
    }

    /// Shrinks the GPU allocation to exactly fit the current instance count.
    ///
    /// Use after a large [`set_instance_count`](Self::set_instance_count) shrink
    /// to free GPU memory. Calling this frequently (e.g. every frame) may
    /// cause performance churn.
    pub fn shrink_to_fit(&mut self) {
        if self.count < self.capacity {
            let new_cap = self.count;
            self.capacity = new_cap;
            realloc_instance_buffer(self.buf_id, self.cpu_mirror.len());
        }
    }

    /// Appends a raw, pre-interleaved instance at the end of the buffer.
    ///
    /// Use this when you have already packed instance bytes (e.g. from reading
    /// a binary file or from a previous buffer's CPU mirror). For structured
    /// appends, prefer [`set_instance_count`](Self::set_instance_count) plus
    /// the typed setters.
    ///
    /// # Errors
    ///
    /// Returns an error if `bytes.len()` does not match `self.stride`.
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

    /// Pushes a new instance with the given position and default values for all
    /// other attributes (rotation = identity quaternion, scale = (1,1,1), etc.).
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer has no position attribute.
    pub fn push_position(&mut self, pos: Vector3<f32>) -> OpticResult<u32> {
        if !self.kind.has_pos {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no position attribute"));
        }
        let stride = self.stride as usize;
        let mut bytes = vec![0u8; stride];
        let off = self.compute_attr_offset(0);
        bytes[off..off + 12].copy_from_slice(&[
            pos.x.to_le_bytes(),
            pos.y.to_le_bytes(),
            pos.z.to_le_bytes(),
        ].concat());
        self.push_raw(&bytes)
    }

    /// Pushes a new instance with position, rotation, and scale all set.
    ///
    /// Rotation is a quaternion stored as `[x, y, z, w]` (matching
    /// [`set_rotation`](Self::set_rotation)).
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer lacks position, rotation, or scale
    /// attributes.
    pub fn push_transform(
        &mut self,
        pos: Vector3<f32>,
        rot: Vector4<f32>,
        scale: Vector3<f32>,
    ) -> OpticResult<u32> {
        if !self.kind.has_pos {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no position attribute"));
        }
        if !self.kind.has_rot {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no rotation attribute"));
        }
        if !self.kind.has_scale {
            return Err(OpticError::new(OpticErrorKind::Custom, "instance buffer has no scale attribute"));
        }
        let stride = self.stride as usize;
        let mut bytes = vec![0u8; stride];

        let pos_off = self.compute_attr_offset(0);
        let rot_off = self.compute_attr_offset(1);
        let scale_off = self.compute_attr_offset(2);

        bytes[pos_off..pos_off + 12].copy_from_slice(&[
            pos.x.to_le_bytes(),
            pos.y.to_le_bytes(),
            pos.z.to_le_bytes(),
        ].concat());
        bytes[rot_off..rot_off + 16].copy_from_slice(&[
            rot.x.to_le_bytes(),
            rot.y.to_le_bytes(),
            rot.z.to_le_bytes(),
            rot.w.to_le_bytes(),
        ].concat());
        bytes[scale_off..scale_off + 12].copy_from_slice(&[
            scale.x.to_le_bytes(),
            scale.y.to_le_bytes(),
            scale.z.to_le_bytes(),
        ].concat());

        self.push_raw(&bytes)
    }

    /// Removes and returns the last instance (O(1)).
    ///
    /// # Errors
    ///
    /// Returns an error if the buffer is empty.
    pub fn pop(&mut self) -> OpticResult<()> {
        if self.count == 0 {
            return Err(OpticError::new(
                OpticErrorKind::Custom,
                "cannot pop from an empty instance buffer",
            ));
        }
        self.count -= 1;
        Ok(())
    }

    /// Removes the instance at `index` by swapping it with the last instance
    /// (unordered, O(1)).
    ///
    /// This is the fastest removal — it simply copies the last instance over
    /// the target and decrements the count. Instance ordering is **not**
    /// preserved. Use [`remove_instance_ordered`](Self::remove_instance_ordered)
    /// if index stability matters.
    ///
    /// # Errors
    ///
    /// Returns an error if `index >= count`.
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

    /// Removes the instance at `index` while preserving the order of remaining
    /// instances (O(n)).
    ///
    /// Shifts all subsequent instances down by one. Prefer the O(1) unordered
    /// [`remove_instance`](Self::remove_instance) when order does not matter.
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

    /// Replaces the entire buffer's data from a 3D instance descriptor.
    ///
    /// This is a full re-upload: the old GPU buffer handle is replaced and the
    /// old handle is **leaked**. If you need explicit GPU-side cleanup, free
    /// the old handle via `glDeleteBuffers` before calling this method.
    ///
    /// Use this when the attribute layout has changed (e.g. added or removed
    /// a custom attribute) so the old interleaved format is incompatible.
    pub fn write_all(&mut self, desc: &InstanceDesc3D) -> OpticResult<()> {
        let new_buf = desc.upload()?;
        self.buf_id = new_buf.buf_id;
        self.capacity = new_buf.capacity;
        self.count = new_buf.count;
        self.stride = new_buf.stride;
        self.layouts = new_buf.layouts;
        self.cpu_mirror = new_buf.cpu_mirror;
        self.kind = new_buf.kind;
        Ok(())
    }

    /// Overwrites a contiguous range of instances with raw interleaved bytes.
    ///
    /// Useful when you have pre-computed instance data externally (e.g. a
    /// particle system updating all particles each frame). The byte slice must
    /// be aligned to `stride` boundaries.
    ///
    /// # Errors
    ///
    /// - `bytes.len()` is not a multiple of `stride`.
    /// - The range `[start, start + instance_count)` exceeds `self.count`.
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
            let off = if self.kind.has_pos { 12 } else { 0 };
            bytes[off + 12..off + 16].copy_from_slice(&1.0f32.to_le_bytes());
        }
        if self.kind.has_scale {
            let mut off = 0usize;
            if self.kind.has_pos { off += 12; }
            if self.kind.has_rot { off += 16; }
            bytes[off..off + 4].copy_from_slice(&1.0f32.to_le_bytes());
            bytes[off + 4..off + 8].copy_from_slice(&1.0f32.to_le_bytes());
            bytes[off + 8..off + 12].copy_from_slice(&1.0f32.to_le_bytes());
        }
        if self.kind.has_col {
            let mut off = 0usize;
            if self.kind.has_pos { off += 12; }
            if self.kind.has_rot { off += 16; }
            if self.kind.has_scale { off += 12; }
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

impl AsCount for CustomATTR {
    fn len(&self) -> usize {
        if self.info.elem_count == 0 || self.info.byte_count == 0 { return 0; }
        self.data.len() / (self.info.elem_count * self.info.byte_count)
    }
    fn is_empty(&self) -> bool { self.data.is_empty() }
}

// ── Text instance factory functions ────────────────────────────────────────

/// Standard 2D instance layout used by text rendering.
pub fn text_instance_desc_2d() -> InstanceDesc2D {
    InstanceDesc2D {
        pos_attr: Pos2DATTR::empty(),
        scale_attr: Scale2DATTR::empty(),
        col_attr: ColorATTR::empty(),
        rot_attr: Rot2DATTR::empty(),
        custom_attrs: vec![
            CustomATTR::empty::<[f32; 4]>("iUVRect"),
            CustomATTR::empty::<u32>("iStyleFlags"),
            CustomATTR::empty::<f32>("iSoftness"),
        ],
    }
}

/// Standard layout for decoration quads (borders, underlines, etc.) in 2D text.
pub fn decoration_instance_desc_2d() -> InstanceDesc2D {
    InstanceDesc2D {
        pos_attr: Pos2DATTR::empty(),
        scale_attr: Scale2DATTR::empty(),
        col_attr: ColorATTR::empty(),
        rot_attr: Rot2DATTR::empty(),
        custom_attrs: vec![],
    }
}

/// Standard 3D instance layout for billboard text.
pub fn text_instance_desc_3d() -> InstanceDesc3D {
    InstanceDesc3D {
        pos_attr: Pos3DATTR::empty(),
        scale_attr: Scale3DATTR::empty(),
        col_attr: ColorATTR::empty(),
        rot_attr: Rot3DATTR::empty(),
        custom_attrs: vec![
            CustomATTR::empty::<[f32; 4]>("iUVRect"),
            CustomATTR::empty::<u32>("iStyleFlags"),
            CustomATTR::empty::<f32>("iSoftness"),
        ],
    }
}

/// 3D decoration layout.
pub fn decoration_instance_desc_3d() -> InstanceDesc3D {
    InstanceDesc3D {
        pos_attr: Pos3DATTR::empty(),
        scale_attr: Scale3DATTR::empty(),
        col_attr: ColorATTR::empty(),
        rot_attr: Rot3DATTR::empty(),
        custom_attrs: vec![],
    }
}

// ── Deserialize helper ─────────────────────────────────────────────────────

fn deserialize<D: DataType>(bytes: &[u8]) -> D {
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
