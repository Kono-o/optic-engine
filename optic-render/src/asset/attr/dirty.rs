use optic_core::ATTRType;

use super::DataType;

/// A dirty-flagged value wrapper for incremental GPU updates via custom instance attributes.
///
/// Wraps any [`DataType`]-compatible value and tracks whether it was modified since the
/// last time the dirty flag was cleared. This is a narrow optional utility — most
/// `InstanceBuffer` usage (single-instance updates via `update_instance`/`set_position`,
/// or building a changed subset and calling `write_range`) already avoids unnecessary
/// uploads without this type. `Dirty<T>` only helps the narrower case of a
/// `write_all()`-style bulk reupload where most instances are unchanged.
pub struct Dirty<T> {
    value: T,
    dirty: bool,
}

impl<T> Dirty<T> {
    /// Create a new dirty-flagged value. The dirty flag starts as `true`
    /// so the value is uploaded on the first frame.
    pub fn new(value: T) -> Self {
        Self { value, dirty: true }
    }

    /// Get a reference to the inner value. Does not affect the dirty flag.
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Replace the inner value and mark as dirty.
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.dirty = true;
    }

    /// Returns `true` if the value has been modified since the last
    /// call to [`clear_dirty`](Dirty::clear_dirty).
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Reset the dirty flag to `false`.
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<T: DataType> DataType for Dirty<T> {
    const ATTR_FORMAT: ATTRType = T::ATTR_FORMAT;
    const BYTE_COUNT: usize = T::BYTE_COUNT;
    const ELEM_COUNT: usize = T::ELEM_COUNT;

    fn u8ify(&self) -> Vec<u8> {
        self.value.u8ify()
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            value: T::from_bytes(bytes),
            dirty: false,
        }
    }
}
