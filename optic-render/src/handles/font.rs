use std::collections::HashMap;
use optic_core::{OpticResult, Size2D};

use crate::asset::{FontFamilyFile, GlyphMetrics};
use crate::Texture2D;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}

impl FontStyle {
    pub fn with_bold(self, bold: bool) -> Self {
        match (self, bold) {
            (Self::Regular, true) => Self::Bold,
            (Self::Italic, true) => Self::BoldItalic,
            (Self::Bold, false) => Self::Regular,
            (Self::BoldItalic, false) => Self::Italic,
            (other, _) => other,
        }
    }

    pub fn with_italic(self, italic: bool) -> Self {
        match (self, italic) {
            (Self::Regular, true) => Self::Italic,
            (Self::Bold, true) => Self::BoldItalic,
            (Self::Italic, false) => Self::Regular,
            (Self::BoldItalic, false) => Self::Bold,
            (other, _) => other,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FontFamily {
    line_height: f32,
    ascent: f32,
    descent: f32,
    is_bitmap: bool,
    ttf_source: Option<Vec<u8>>,
    regular_atlas: Texture2D,
    bold_atlas: Option<Texture2D>,
    italic_atlas: Option<Texture2D>,
    bold_italic_atlas: Option<Texture2D>,
    regular_glyphs: HashMap<u32, GlyphMetrics>,
    bold_glyphs: HashMap<u32, GlyphMetrics>,
    italic_glyphs: HashMap<u32, GlyphMetrics>,
    bold_italic_glyphs: HashMap<u32, GlyphMetrics>,
    regular_softness: f32,
    bold_softness: f32,
    italic_softness: f32,
    bold_italic_softness: f32,
}

impl FontFamily {
    pub(crate) fn new(file: &FontFamilyFile) -> OpticResult<Self> {
        let regular_atlas = file.regular.atlas.upload();
        let bold_atlas = file.bold.as_ref().map(|b| b.atlas.upload());
        let italic_atlas = file.italic.as_ref().map(|i| i.atlas.upload());
        let bold_italic_atlas = file.bold_italic.as_ref().map(|bi| bi.atlas.upload());

        Ok(FontFamily {
            line_height: file.line_height,
            ascent: file.ascent,
            descent: file.descent,
            is_bitmap: file.is_bitmap,
            ttf_source: file.ttf_source.clone(),
            regular_atlas,
            bold_atlas,
            italic_atlas,
            bold_italic_atlas,
            regular_glyphs: file.regular.glyphs.clone(),
            bold_glyphs: file.bold.as_ref().map(|b| b.glyphs.clone()).unwrap_or_default(),
            italic_glyphs: file.italic.as_ref().map(|i| i.glyphs.clone()).unwrap_or_default(),
            bold_italic_glyphs: file.bold_italic.as_ref().map(|bi| bi.glyphs.clone()).unwrap_or_default(),
            regular_softness: file.regular.edge_softness,
            bold_softness: file.bold.as_ref().map(|b| b.edge_softness).unwrap_or(file.regular.edge_softness),
            italic_softness: file.italic.as_ref().map(|i| i.edge_softness).unwrap_or(file.regular.edge_softness),
            bold_italic_softness: file.bold_italic.as_ref().map(|bi| bi.edge_softness).unwrap_or(file.regular.edge_softness),
        })
    }

    #[cfg(test)]
    fn new_no_upload(file: &FontFamilyFile) -> Self {
        let dummy_tex = || Texture2D::new(0, Size2D::new(1, 1), optic_core::ImgFormat::RGBA(8), optic_core::ImgFilter::Linear, optic_core::ImgWrap::Clip);

        FontFamily {
            line_height: file.line_height,
            ascent: file.ascent,
            descent: file.descent,
            is_bitmap: file.is_bitmap,
            ttf_source: file.ttf_source.clone(),
            regular_atlas: dummy_tex(),
            bold_atlas: file.bold.as_ref().map(|_| dummy_tex()),
            italic_atlas: file.italic.as_ref().map(|_| dummy_tex()),
            bold_italic_atlas: file.bold_italic.as_ref().map(|_| dummy_tex()),
            regular_glyphs: file.regular.glyphs.clone(),
            bold_glyphs: file.bold.as_ref().map(|b| b.glyphs.clone()).unwrap_or_default(),
            italic_glyphs: file.italic.as_ref().map(|i| i.glyphs.clone()).unwrap_or_default(),
            bold_italic_glyphs: file.bold_italic.as_ref().map(|bi| bi.glyphs.clone()).unwrap_or_default(),
            regular_softness: file.regular.edge_softness,
            bold_softness: file.bold.as_ref().map(|b| b.edge_softness).unwrap_or(file.regular.edge_softness),
            italic_softness: file.italic.as_ref().map(|i| i.edge_softness).unwrap_or(file.regular.edge_softness),
            bold_italic_softness: file.bold_italic.as_ref().map(|bi| bi.edge_softness).unwrap_or(file.regular.edge_softness),
        }
    }

    pub fn fallback_bitmap() -> OpticResult<Self> {
        Self::new(&FontFamilyFile::fallback()?)
    }

    #[cfg(test)]
    pub fn test_bitmap() -> Self {
        Self::new_no_upload(&FontFamilyFile::fallback().expect("fallback font"))
    }

    pub fn line_height(&self) -> f32 { self.line_height }
    pub fn ascent(&self) -> f32 { self.ascent }
    pub fn descent(&self) -> f32 { self.descent }

    pub fn is_bitmap(&self) -> bool { self.is_bitmap }

    pub fn units_per_em(&self) -> f32 {
        if self.is_bitmap {
            self.line_height
        } else {
            1.0
        }
    }

    pub fn face_data(&self) -> Option<&[u8]> {
        self.ttf_source.as_deref()
    }

    pub fn atlas(&self, style: FontStyle) -> &Texture2D {
        match style {
            FontStyle::Regular => &self.regular_atlas,
            FontStyle::Bold => self.bold_atlas.as_ref().unwrap_or(&self.regular_atlas),
            FontStyle::Italic => self.italic_atlas.as_ref().unwrap_or(&self.regular_atlas),
            FontStyle::BoldItalic => self.bold_italic_atlas.as_ref().unwrap_or(&self.regular_atlas),
        }
    }

    pub fn primary_atlas(&self) -> &Texture2D {
        &self.regular_atlas
    }

    pub fn edge_softness(&self, style: FontStyle) -> f32 {
        match style {
            FontStyle::Regular => self.regular_softness,
            FontStyle::Bold => self.bold_softness,
            FontStyle::Italic => self.italic_softness,
            FontStyle::BoldItalic => self.bold_italic_softness,
        }
    }

    pub fn glyph(&self, style: FontStyle, gid: u32) -> Option<&GlyphMetrics> {
        let map = match style {
            FontStyle::Regular => &self.regular_glyphs,
            FontStyle::Bold => &self.bold_glyphs,
            FontStyle::Italic => &self.italic_glyphs,
            FontStyle::BoldItalic => &self.bold_italic_glyphs,
        };
        if map.contains_key(&gid) {
            map.get(&gid)
        } else {
            self.regular_glyphs.get(&gid)
        }
    }

    pub fn has_style(&self, style: FontStyle) -> bool {
        match style {
            FontStyle::Regular => true,
            FontStyle::Bold => self.bold_atlas.is_some(),
            FontStyle::Italic => self.italic_atlas.is_some(),
            FontStyle::BoldItalic => self.bold_italic_atlas.is_some(),
        }
    }

    pub fn resolve_style(&self, bold: bool, italic: bool) -> (FontStyle, bool, bool) {
        let mut style = FontStyle::Regular;
        let mut faux_bold = false;
        let mut faux_italic = false;

        if bold {
            if self.has_style(FontStyle::Bold) {
                style = style.with_bold(true);
            } else {
                faux_bold = true;
            }
        }
        if italic {
            if self.has_style(FontStyle::Italic) {
                style = style.with_italic(true);
            } else {
                faux_italic = true;
            }
        }

        if bold && italic && !self.has_style(FontStyle::BoldItalic) {
            if self.has_style(FontStyle::Bold) {
                style = FontStyle::Bold;
                faux_italic = !self.has_style(FontStyle::Italic);
            } else if self.has_style(FontStyle::Italic) {
                style = FontStyle::Italic;
                faux_bold = true;
            }
        }

        (style, faux_bold, faux_italic)
    }

    pub fn delete(self) {
        self.regular_atlas.delete();
        if let Some(atlas) = self.bold_atlas { atlas.delete(); }
        if let Some(atlas) = self.italic_atlas { atlas.delete(); }
        if let Some(atlas) = self.bold_italic_atlas { atlas.delete(); }
    }
}

impl Default for FontFamily {
    fn default() -> Self {
        let dummy_tex = Texture2D::new(0, Size2D::new(1, 1), optic_core::ImgFormat::RGBA(8), optic_core::ImgFilter::Linear, optic_core::ImgWrap::Clip);
        FontFamily {
            line_height: 1.0,
            ascent: 0.8,
            descent: -0.2,
            is_bitmap: false,
            ttf_source: None,
            regular_atlas: dummy_tex.clone(),
            bold_atlas: None,
            italic_atlas: None,
            bold_italic_atlas: None,
            regular_glyphs: HashMap::new(),
            bold_glyphs: HashMap::new(),
            italic_glyphs: HashMap::new(),
            bold_italic_glyphs: HashMap::new(),
            regular_softness: 0.15,
            bold_softness: 0.15,
            italic_softness: 0.15,
            bold_italic_softness: 0.15,
        }
    }
}
