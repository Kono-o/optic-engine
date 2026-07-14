//! Dirty-flagged value wrapper for incremental GPU attribute updates.
//!
//! [`Signal<T>`] wraps any [`DataType`]-compatible value and tracks whether it
//! has been modified since the last time the dirty flag was cleared. This
//! enables the instance-buffer system to skip unchanged attributes when
//! uploading per-instance data to the GPU, avoiding redundant memory copies
//! and transfer overhead.
//!
//! # Architecture
//!
//! ```text
//! Signal<T>  →  CustomATTR  →  InstanceBuffer  →  GPU
//! ```
//!
//! 1. A user-facing component (e.g. a transform, colour, or sprite) owns a
//!    `Signal<T>`.
//! 2. Each frame the component checks [`Signal::is_dirty`]. If `true`, the
//!    new value is serialised via [`DataType::u8ify`] and written into the
//!    corresponding slot of the instance buffer.
//! 3. After the upload, [`Signal::clear_dirty`] is called so the slot is not
//!    re-uploaded next frame.
//!
//! # When to use `Signal<T>` vs a plain `T`
//!
//! Use `Signal<T>` when:
//! - The value is written to a per-instance GPU buffer every frame.
//! - Most instances do **not** change every frame, so skipping unchanged
//!   slots saves meaningful bandwidth.
//!
//! Use a plain `T` when the value is always read (never written mid-frame)
//! or when there is no incremental-upload path.

use optic_core::ATTRType;
use optic_render::asset::attr::DataType;

/// A dirty-flagged value wrapper for incremental GPU updates via custom instance attributes.
///
/// Wraps any DataType-compatible value and tracks whether it was modified since the last time the
/// dirty flag was cleared. The instance-buffer system uses this to skip unchanged per-instance
/// attributes, avoiding redundant GPU uploads. Use Signal when most instances don't change every
/// frame but you still want fine-grained per-frame updates for those that do.
pub struct Signal<T> {
    value: T,
    dirty: bool,
}

impl<T> Signal<T> {
    /// Create a new signal with the given initial value.
    ///
    /// The dirty flag is set to `true` so the value is uploaded on the
    /// first frame.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let s = Signal::new(42u32);
    /// assert!(s.is_dirty());
    /// assert_eq!(*s.value(), 42);
    /// ```
    pub fn new(value: T) -> Self {
        Self {
            value,
            dirty: true,
        }
    }

    /// Get a reference to the inner value.
    ///
    /// This does **not** affect the dirty flag.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let s = Signal::new([1.0f32, 2.0, 3.0]);
    /// assert_eq!(s.value(), &[1.0, 2.0, 3.0]);
    /// ```
    pub fn value(&self) -> &T {
        &self.value
    }

    /// Replace the inner value and mark the signal as dirty.
    ///
    /// The dirty flag is set unconditionally — even if the new value is
    /// equal to the old one. If equality-based optimisation is needed, compare
    /// before calling `set`.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut s = Signal::new(0.0f32);
    /// s.clear_dirty();
    ///
    /// s.set(1.0);
    /// assert!(s.is_dirty());
    /// ```
    pub fn set(&mut self, value: T) {
        self.value = value;
        self.dirty = true;
    }

    /// Returns `true` if the value has been modified since the last call to
    /// [`clear_dirty`](Signal::clear_dirty).
    ///
    /// The instance-buffer system checks this each frame to decide whether
    /// to re-upload the slot.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut s = Signal::new(());
    /// assert!(s.is_dirty());
    ///
    /// s.clear_dirty();
    /// assert!(!s.is_dirty());
    ///
    /// s.set(());
    /// assert!(s.is_dirty());
    /// ```
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Reset the dirty flag to `false`.
    ///
    /// Call this after the value has been uploaded to the instance buffer.
    /// The value itself is not modified.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let mut s = Signal::new(10u8);
    /// // … upload to GPU …
    /// s.clear_dirty();
    /// assert!(!s.is_dirty());
    /// ```
    pub fn clear_dirty(&mut self) {
        self.dirty = false;
    }
}

impl<T: DataType> DataType for Signal<T> {
    const ATTR_FORMAT: ATTRType = T::ATTR_FORMAT;
    const BYTE_COUNT: usize = T::BYTE_COUNT;
    const ELEM_COUNT: usize = T::ELEM_COUNT;

    fn u8ify(&self) -> Vec<u8> {
        self.value.u8ify()
    }
}
