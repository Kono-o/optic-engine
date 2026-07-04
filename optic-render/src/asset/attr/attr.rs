use optic_core::ATTRType;

use crate::asset::attr::DataType;

#[derive(Clone, Debug, PartialEq)]
pub enum ATTRName {
    Custom(String),
    Pos2D,
    Pos3D,
    Col,
    UVM,
    Nrm,
    Ind,
    Rot3D,
    Rot2D,
    Scale3D,
    Scale2D,
}

impl ATTRName {
    pub fn as_string(&self) -> String {
        match self {
            ATTRName::Pos2D => "pos2d".into(),
            ATTRName::Pos3D => "pos3d".into(),
            ATTRName::Col => "color".into(),
            ATTRName::UVM => "uv map".into(),
            ATTRName::Nrm => "normals".into(),
            ATTRName::Ind => "indices".into(),
            ATTRName::Rot3D => "rotation 3d".into(),
            ATTRName::Rot2D => "rotation 2d".into(),
            ATTRName::Scale3D => "scale 3d".into(),
            ATTRName::Scale2D => "scale 2d".into(),
            ATTRName::Custom(n) => format!("{n}(custom)"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ATTRInfo {
    pub name: ATTRName,
    pub typ: ATTRType,
    pub byte_count: usize,
    pub elem_count: usize,
}

impl ATTRInfo {
    pub fn empty() -> Self {
        Self {
            name: ATTRName::Pos3D,
            typ: ATTRType::F32,
            byte_count: 0,
            elem_count: 0,
        }
    }

    pub fn fmt_as_string(&self) -> String {
        let typ_str = match self.typ {
            ATTRType::U8 => "u8",
            ATTRType::I8 => "i8",
            ATTRType::U16 => "u16",
            ATTRType::I16 => "i16",
            ATTRType::U32 => "u32",
            ATTRType::I32 => "i32",
            ATTRType::F32 => "f32",
            ATTRType::F64 => "f64",
        };
        if self.elem_count == 1 {
            typ_str.to_string()
        } else {
            format!("[{typ_str};{}]", self.elem_count)
        }
    }
}

macro_rules! attr {
    ($attr:ident, $typ:ty, $name:expr) => {
        #[derive(Debug, Clone)]
        pub struct $attr {
            pub data: Vec<$typ>,
            pub info: ATTRInfo,
        }

        impl $attr {
            pub fn empty() -> Self {
                let mut info = ATTRInfo::empty();
                info.typ = <$typ>::ATTR_FORMAT;
                info.byte_count = <$typ>::BYTE_COUNT;
                info.elem_count = <$typ>::ELEM_COUNT;
                info.name = $name;
                Self { data: Vec::new(), info }
            }

            pub fn from(vec: Vec<$typ>) -> Self {
                let mut attr = Self::empty();
                for elem in vec {
                    attr.data.push(elem);
                }
                attr
            }

            pub fn from_array(array: &[$typ]) -> Self {
                Self::from(Vec::from(array))
            }

            pub fn push(&mut self, elem: $typ) {
                self.data.push(elem);
            }

            pub fn is_empty(&self) -> bool {
                self.data.is_empty()
            }
        }
    };
}

attr!(Pos3DATTR, [f32; 3], ATTRName::Pos3D);
attr!(Pos2DATTR, [f32; 2], ATTRName::Pos2D);
attr!(ColATTR, [f32; 4], ATTRName::Col);
attr!(UVMATTR, [f32; 2], ATTRName::UVM);
attr!(NrmATTR, [f32; 3], ATTRName::Nrm);
attr!(IndATTR, u32, ATTRName::Ind);
attr!(Rot3DATTR, [f32; 4], ATTRName::Rot3D);
attr!(Rot2DATTR, f32, ATTRName::Rot2D);
attr!(Scale3DATTR, [f32; 3], ATTRName::Scale3D);
attr!(Scale2DATTR, [f32; 2], ATTRName::Scale2D);

#[derive(Debug)]
pub struct CustomATTR {
    pub data: Vec<u8>,
    pub info: ATTRInfo,
}

impl CustomATTR {
    pub fn empty<D: DataType>(name: &str) -> Self {
        let mut info = ATTRInfo::empty();
        info.typ = D::ATTR_FORMAT;
        info.byte_count = D::BYTE_COUNT;
        info.elem_count = D::ELEM_COUNT;
        info.name = ATTRName::Custom(name.to_string());
        Self { data: Vec::new(), info }
    }

    pub fn from<D: DataType>(name: &str, vec: Vec<D>) -> Self {
        let mut attr = Self::empty::<D>(name);
        for elem in vec {
            let bytes = elem.u8ify();
            attr.data.extend_from_slice(&bytes);
        }
        attr
    }

    pub fn from_array<D: DataType + Clone>(name: &str, array: &[D]) -> Self {
        Self::from(name, Vec::from(array))
    }

    pub fn push<D: DataType>(&mut self, elem: D) {
        let bytes = elem.u8ify();
        self.data.extend_from_slice(&bytes);
    }

    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attr_info_empty() {
        let info = ATTRInfo::empty();
        assert_eq!(info.byte_count, 0);
        assert_eq!(info.elem_count, 0);
    }

    #[test]
    fn attr_info_fmt_as_string() {
        let mut info = ATTRInfo::empty();
        info.typ = ATTRType::F32;
        info.elem_count = 3;
        assert_eq!(info.fmt_as_string(), "[f32;3]");
        info.elem_count = 1;
        assert_eq!(info.fmt_as_string(), "f32");
    }

    #[test]
    fn attr_name_as_string() {
        assert_eq!(ATTRName::Pos2D.as_string(), "pos2d");
        assert_eq!(ATTRName::Pos3D.as_string(), "pos3d");
        assert_eq!(ATTRName::Col.as_string(), "color");
        assert_eq!(ATTRName::UVM.as_string(), "uv map");
        assert_eq!(ATTRName::Nrm.as_string(), "normals");
        assert_eq!(ATTRName::Ind.as_string(), "indices");
        assert_eq!(ATTRName::Rot3D.as_string(), "rotation 3d");
        assert_eq!(ATTRName::Rot2D.as_string(), "rotation 2d");
        assert_eq!(ATTRName::Scale3D.as_string(), "scale 3d");
        assert_eq!(ATTRName::Scale2D.as_string(), "scale 2d");
        let custom = ATTRName::Custom("user_data".into());
        assert_eq!(custom.as_string(), "user_data(custom)");
    }

    #[test]
    fn pos3d_attr() {
        let mut attr = Pos3DATTR::empty();
        assert!(attr.is_empty());
        attr.push([1.0, 2.0, 3.0]);
        assert!(!attr.is_empty());
        assert_eq!(attr.data.len(), 1);
        assert_eq!(attr.data[0], [1.0, 2.0, 3.0]);
    }

    #[test]
    fn pos3d_from_array() {
        let attr = Pos3DATTR::from_array(&[[0.0, 0.0, 0.0], [1.0, 1.0, 1.0]]);
        assert_eq!(attr.data.len(), 2);
        assert_eq!(attr.info.name.as_string(), "pos3d");
    }

    #[test]
    fn pos2d_attr() {
        let mut attr = Pos2DATTR::empty();
        attr.push([0.5, 0.5]);
        assert_eq!(attr.data[0], [0.5, 0.5]);
    }

    #[test]
    fn color_attr() {
        let mut attr = ColATTR::empty();
        attr.push([1.0, 0.0, 0.0, 1.0]);
        assert_eq!(attr.data[0], [1.0, 0.0, 0.0, 1.0]);
    }

    #[test]
    fn uvm_attr() {
        let attr = UVMATTR::from_array(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        assert_eq!(attr.data.len(), 4);
    }

    #[test]
    fn nrm_attr() {
        let attr = NrmATTR::from_array(&[[0.0, 1.0, 0.0]]);
        assert_eq!(attr.data[0], [0.0, 1.0, 0.0]);
    }

    #[test]
    fn ind_attr() {
        let mut attr = IndATTR::empty();
        attr.push(0);
        attr.push(1);
        attr.push(2);
        assert_eq!(attr.data, vec![0, 1, 2]);
    }

    #[test]
    fn rot3d_attr() {
        let mut attr = Rot3DATTR::empty();
        attr.push([0.0, 0.0, 0.0, 1.0]);
        assert_eq!(attr.data[0], [0.0, 0.0, 0.0, 1.0]);
        assert_eq!(attr.info.elem_count, 4);
        assert_eq!(attr.info.byte_count, 4);
    }

    #[test]
    fn rot2d_attr() {
        let mut attr = Rot2DATTR::empty();
        attr.push(1.5708);
        assert!((attr.data[0] - 1.5708).abs() < 0.001);
        assert_eq!(attr.info.elem_count, 1);
    }

    #[test]
    fn scale3d_attr() {
        let attr = Scale3DATTR::from_array(&[[1.0, 1.0, 1.0], [2.0, 2.0, 2.0]]);
        assert_eq!(attr.data.len(), 2);
        assert_eq!(attr.info.name.as_string(), "scale 3d");
    }

    #[test]
    fn custom_attr_empty() {
        let attr = CustomATTR::empty::<[f32; 3]>("weights");
        assert!(attr.is_empty());
        assert_eq!(attr.info.name.as_string(), "weights(custom)");
        assert_eq!(attr.info.typ, ATTRType::F32);
        assert_eq!(attr.info.elem_count, 3);
    }

    #[test]
    fn custom_attr_from_array() {
        let attr = CustomATTR::from_array::<[f32; 3]>("bone_weights", &[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]);
        assert!(!attr.is_empty());
        assert_eq!(attr.data.len(), 24); // 2 * 3 * 4 bytes
    }

    #[test]
    fn custom_attr_push() {
        let mut attr = CustomATTR::empty::<u32>("ids");
        attr.push(42u32);
        attr.push(99u32);
        assert_eq!(attr.data.len(), 8); // 2 * 4 bytes
    }
}
