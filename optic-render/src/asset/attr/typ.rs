use optic_core::ATTRType;

/// Trait for types that can be stored as GPU vertex or instance attributes.
///
/// Implemented for i8/u8/i16/u16/i32/u32/f32/f64 and their fixed-size arrays [T;2], [T;3], [T;4].
/// Each implementation provides the GL attribute format, byte size, element count, and serialisation
/// to raw bytes for GPU upload. The engine requires this trait on all vertex and instance attribute types.
pub trait DataType: Sized {
    const ATTR_FORMAT: ATTRType;
    const BYTE_COUNT: usize;
    const ELEM_COUNT: usize;
    fn u8ify(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}

macro_rules! u8ify_impl {
    ([$t:ty; $s:literal]) => {
        fn u8ify(&self) -> Vec<u8> {
            let mut vec = Vec::new();
            for elem in self.iter() {
                vec.extend_from_slice(&elem.to_ne_bytes());
            }
            vec
        }
        fn from_bytes(bytes: &[u8]) -> Self {
            let mut arr = [<$t>::default(); $s];
            for (i, elem) in arr.iter_mut().enumerate() {
                let off = i * std::mem::size_of::<$t>();
                *elem = <$t>::from_ne_bytes(bytes[off..off + std::mem::size_of::<$t>()].try_into().unwrap());
            }
            arr
        }
    };
    ($t:ty) => {
        fn u8ify(&self) -> Vec<u8> {
            self.to_ne_bytes().to_vec()
        }
        fn from_bytes(bytes: &[u8]) -> Self {
            <$t>::from_ne_bytes(bytes.try_into().unwrap())
        }
    };
}

macro_rules! datatype {
    ($type:ty, $attr_format:expr, $byte_count:expr) => {
        impl DataType for $type {
            const ATTR_FORMAT: ATTRType = $attr_format;
            const BYTE_COUNT: usize = $byte_count;
            const ELEM_COUNT: usize = 1;
            u8ify_impl!($type);
        }

        impl DataType for [$type; 2] {
            const ATTR_FORMAT: ATTRType = $attr_format;
            const BYTE_COUNT: usize = $byte_count;
            const ELEM_COUNT: usize = 2;
            u8ify_impl!([$type; 2]);
        }

        impl DataType for [$type; 3] {
            const ATTR_FORMAT: ATTRType = $attr_format;
            const BYTE_COUNT: usize = $byte_count;
            const ELEM_COUNT: usize = 3;
            u8ify_impl!([$type; 3]);
        }

        impl DataType for [$type; 4] {
            const ATTR_FORMAT: ATTRType = $attr_format;
            const BYTE_COUNT: usize = $byte_count;
            const ELEM_COUNT: usize = 4;
            u8ify_impl!([$type; 4]);
        }
    };
}

datatype!(i8, ATTRType::I8, 1);
datatype!(u8, ATTRType::U8, 1);
datatype!(i16, ATTRType::I16, 2);
datatype!(u16, ATTRType::U16, 2);
datatype!(i32, ATTRType::I32, 4);
datatype!(u32, ATTRType::U32, 4);
datatype!(f32, ATTRType::F32, 4);
datatype!(f64, ATTRType::F64, 8);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn f32_scalar_u8ify() {
        let v: f32 = 1.0;
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn f32_array2_u8ify() {
        let v: [f32; 2] = [1.0, 2.0];
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn f32_array3_u8ify() {
        let v: [f32; 3] = [1.0, 2.0, 3.0];
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 12);
    }

    #[test]
    fn f32_array4_u8ify() {
        let v: [f32; 4] = [1.0, 2.0, 3.0, 4.0];
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 16);
    }

    #[test]
    fn u32_scalar_u8ify() {
        let v: u32 = 0x12345678;
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn u32_array2_u8ify() {
        let v: [u32; 2] = [1, 2];
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn u8_scalar_u8ify() {
        let v: u8 = 255;
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 1);
    }

    #[test]
    fn i32_scalar_u8ify() {
        let v: i32 = -42;
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 4);
    }

    #[test]
    fn f64_scalar_u8ify() {
        let v: f64 = 3.14159;
        let bytes = v.u8ify();
        assert_eq!(bytes.len(), 8);
    }

    #[test]
    fn const_byte_count() {
        assert_eq!(<u8 as DataType>::BYTE_COUNT, 1);
        assert_eq!(<i8 as DataType>::BYTE_COUNT, 1);
        assert_eq!(<u16 as DataType>::BYTE_COUNT, 2);
        assert_eq!(<i16 as DataType>::BYTE_COUNT, 2);
        assert_eq!(<u32 as DataType>::BYTE_COUNT, 4);
        assert_eq!(<i32 as DataType>::BYTE_COUNT, 4);
        assert_eq!(<f32 as DataType>::BYTE_COUNT, 4);
        assert_eq!(<f64 as DataType>::BYTE_COUNT, 8);
    }

    #[test]
    fn const_elem_count() {
        assert_eq!(<[f32; 3] as DataType>::ELEM_COUNT, 3);
        assert_eq!(<[f32; 2] as DataType>::ELEM_COUNT, 2);
        assert_eq!(<[f32; 4] as DataType>::ELEM_COUNT, 4);
        assert_eq!(<u32 as DataType>::ELEM_COUNT, 1);
    }
}
