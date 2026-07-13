use optic_core::ATTRType;
use optic_render::asset::attr::DataType;

pub struct Signal<T> {
    value: T,
    dirty: bool,
}

impl<T> Signal<T> {
    pub fn new(value: T) -> Self {
        Self {
            value,
            dirty: true,
        }
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn set(&mut self, value: T) {
        self.value = value;
        self.dirty = true;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

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
