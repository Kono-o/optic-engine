#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PolyMode {
    Points,
    WireFrame,
    Filled,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Cull {
    Clock,
    AntiClock,
}

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum DrawMode {
    Points,
    Lines,
    #[default]
    Triangles,
    Strip,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImgFormat {
    R(u8),
    RG(u8),
    RGB(u8),
    RGBA(u8),
}

impl ImgFormat {
    pub fn channels(&self) -> u8 {
        match self {
            ImgFormat::R(_) => 1,
            ImgFormat::RG(_) => 2,
            ImgFormat::RGB(_) => 3,
            ImgFormat::RGBA(_) => 4,
        }
    }
    pub fn bit_depth(&self) -> u8 {
        *match self {
            ImgFormat::R(bd) => bd,
            ImgFormat::RG(bd) => bd,
            ImgFormat::RGB(bd) => bd,
            ImgFormat::RGBA(bd) => bd,
        }
    }
    pub fn pixel_size(&self) -> u8 {
        self.channels() * self.bit_depth()
    }
    pub fn from(channels: u8, bit_depth: u8) -> ImgFormat {
        match channels {
            1 => ImgFormat::R(bit_depth),
            2 => ImgFormat::RG(bit_depth),
            3 => ImgFormat::RGB(bit_depth),
            _ => ImgFormat::RGBA(bit_depth),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgFilter {
    Closest,
    Linear,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgWrap {
    Repeat,
    Extend,
    Clip,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ATTRType {
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    F32,
    F64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polymode_variants() {
        match PolyMode::Points { _ => {} }
        match PolyMode::WireFrame { _ => {} }
        match PolyMode::Filled { _ => {} }
    }

    #[test]
    fn cull_variants() {
        match Cull::Clock { _ => {} }
        match Cull::AntiClock { _ => {} }
    }

    #[test]
    fn draw_mode_default() {
        let dm: DrawMode = Default::default();
        match dm {
            DrawMode::Triangles => {},
            _ => panic!("default should be Triangles"),
        }
    }

    #[test]
    fn draw_mode_variants() {
        match DrawMode::Points { _ => {} }
        match DrawMode::Lines { _ => {} }
        match DrawMode::Triangles { _ => {} }
        match DrawMode::Strip { _ => {} }
    }

    #[test]
    fn imgformat_channels() {
        assert_eq!(ImgFormat::R(8).channels(), 1);
        assert_eq!(ImgFormat::RG(8).channels(), 2);
        assert_eq!(ImgFormat::RGB(8).channels(), 3);
        assert_eq!(ImgFormat::RGBA(8).channels(), 4);
    }

    #[test]
    fn imgformat_bit_depth() {
        assert_eq!(ImgFormat::R(8).bit_depth(), 8);
        assert_eq!(ImgFormat::RGBA(16).bit_depth(), 16);
        assert_eq!(ImgFormat::RGB(32).bit_depth(), 32);
    }

    #[test]
    fn imgformat_pixel_size() {
        assert_eq!(ImgFormat::RGBA(8).pixel_size(), 32);
        assert_eq!(ImgFormat::RGB(8).pixel_size(), 24);
        assert_eq!(ImgFormat::R(16).pixel_size(), 16);
    }

    #[test]
    fn imgformat_from() {
        match ImgFormat::from(1, 8) {
            ImgFormat::R(8) => {},
            _ => panic!("expected R(8)"),
        }
        match ImgFormat::from(4, 16) {
            ImgFormat::RGBA(16) => {},
            _ => panic!("expected RGBA(16)"),
        }
        match ImgFormat::from(5, 8) {
            ImgFormat::RGBA(8) => {},
            _ => panic!("expected RGBA(8) for 5 channels"),
        }
    }

    #[test]
    fn imgfilter_variants() {
        match ImgFilter::Closest { _ => {} }
        match ImgFilter::Linear { _ => {} }
    }

    #[test]
    fn imgwrap_variants() {
        match ImgWrap::Repeat { _ => {} }
        match ImgWrap::Extend { _ => {} }
        match ImgWrap::Clip { _ => {} }
    }

    #[test]
    fn attr_type_variants() {
        match ATTRType::U8 { _ => {} }
        match ATTRType::I8 { _ => {} }
        match ATTRType::U16 { _ => {} }
        match ATTRType::I16 { _ => {} }
        match ATTRType::U32 { _ => {} }
        match ATTRType::I32 { _ => {} }
        match ATTRType::F32 { _ => {} }
        match ATTRType::F64 { _ => {} }
    }
}
