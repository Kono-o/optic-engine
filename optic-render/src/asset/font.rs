use std::collections::HashMap;
use optic_core::{OpticError, OpticErrorKind, OpticResult, Size2D};
use optic_core::consts::{OPTIC_CACHE_VERSION, OPTIC_MAGIC, OFONT};

use crate::asset::img::TextureFile;
use crate::asset::msdf::{bake_msdf, bake_sdf_from_bitmap, extract_glyph_edges};

#[derive(Clone, Debug)]
pub struct GlyphMetrics {
    pub uv_rect: (f32, f32, f32, f32),
    pub size: Size2D,
    pub bearing: (f32, f32),
    pub advance: f32,
}

impl GlyphMetrics {
    pub fn zero() -> Self {
        GlyphMetrics {
            uv_rect: (0.0, 0.0, 0.0, 0.0),
            size: Size2D::new(0, 0),
            bearing: (0.0, 0.0),
            advance: 0.0,
        }
    }

    pub fn uv(&self) -> [f32; 4] {
        [self.uv_rect.0, self.uv_rect.1, self.uv_rect.2, self.uv_rect.3]
    }

    pub fn size_arr(&self) -> [f32; 2] {
        [self.size.w as f32, self.size.h as f32]
    }

    pub fn bearing_arr(&self) -> [f32; 2] {
        [self.bearing.0, self.bearing.1]
    }
}

pub struct BakedFont {
    pub atlas: TextureFile,
    pub glyphs: HashMap<u32, GlyphMetrics>,
    pub edge_softness: f32,
}

pub struct FontFamilyFile {
    pub line_height: f32,
    pub ascent: f32,
    pub descent: f32,
    pub regular: BakedFont,
    pub bold: Option<BakedFont>,
    pub italic: Option<BakedFont>,
    pub bold_italic: Option<BakedFont>,
    pub ttf_source: Option<Vec<u8>>,
    pub is_bitmap: bool,
}

pub struct BitmapFontLayout {
    pub texture: TextureFile,
    pub glyph_size: Size2D,
    pub columns: u32,
    pub codepoint_order: Vec<u32>,
    pub advance: Option<u32>,
}

const ATLAS_SIZE: u32 = 1024;
const PX_RANGE: f32 = 4.0;
const STYLE_REGULAR: u8 = 1 << 0;
const STYLE_BOLD: u8 = 1 << 1;
const STYLE_ITALIC: u8 = 1 << 2;
const STYLE_BOLD_ITALIC: u8 = 1 << 3;

impl FontFamilyFile {
    pub fn from_ttf_file(regular_bytes: &[u8], codepoint_range: (u32, u32), atlas_resolution: u32) -> OpticResult<Self> {
        let face = ttf_parser::Face::parse(regular_bytes, 0)
            .map_err(|_| OpticError::new(OpticErrorKind::Custom, "failed to parse TTF font"))?;

        let units_per_em = face.units_per_em() as f32;
        let line_height = face.height() as f32 / units_per_em;
        let ascent = face.ascender() as f32 / units_per_em;
        let descent = face.descender() as f32 / units_per_em;

        let atlas_size = atlas_resolution;
        let max_glyphs = face.number_of_glyphs().min(1024) as u32;
        let per_row = (atlas_size as f32 / 64.0).floor() as u32;
        let glyph_cell = atlas_size / per_row;

        let mut glyphs = HashMap::new();
        let mut atlas_pixels = vec![0u8; (atlas_size * atlas_size * 3) as usize];
        let mut next_gid = 0u32;

        for codepoint in codepoint_range.0..codepoint_range.1 {
            let gid = face.glyph_index(char::from_u32(codepoint).unwrap_or(' '));
            if let Some(gid) = gid {
                let gid_u16 = gid.0 as u32;
                if glyphs.contains_key(&gid_u16) { continue; }
                if next_gid >= max_glyphs { break; }

                if let Some(edges) = extract_glyph_edges(&face, gid.0) {
                    let glyph_data = bake_msdf(&edges, glyph_cell, PX_RANGE);

                    let row = next_gid / per_row;
                    let col = next_gid % per_row;
                    let dx = col * glyph_cell;
                    let dy = row * glyph_cell;

                    for gy in 0..glyph_cell as usize {
                        for gx in 0..glyph_cell as usize {
                            let src = (gy * glyph_cell as usize + gx) * 3;
                            let dst = ((dy + gy as u32) * atlas_size + (dx + gx as u32)) as usize * 3;
                            if dst + 2 < atlas_pixels.len() && src + 2 < glyph_data.len() {
                                atlas_pixels[dst] = glyph_data[src];
                                atlas_pixels[dst + 1] = glyph_data[src + 1];
                                atlas_pixels[dst + 2] = glyph_data[src + 2];
                            }
                        }
                    }

                    let u0 = col as f32 / per_row as f32;
                    let v0 = row as f32 / per_row as f32;
                    let u1 = (col + 1) as f32 / per_row as f32;
                    let v1 = (row + 1) as f32 / per_row as f32;

                    let px_size = edges.width.max(edges.height) * glyph_cell as f32;

                    glyphs.insert(gid_u16, GlyphMetrics {
                        uv_rect: (u0, v0, u1, v1),
                        size: Size2D::new(px_size as u32, px_size as u32),
                        bearing: (edges.bearing_x, edges.bearing_y),
                        advance: edges.advance,
                    });

                    next_gid += 1;
                }
            }
        }

        let atlas = TextureFile {
            bytes: atlas_pixels,
            size: Size2D::new(atlas_size, atlas_size),
            fmt: optic_core::ImgFormat::RGBA(8),
            filter: optic_core::ImgFilter::Linear,
            wrap: optic_core::ImgWrap::Clip,
        };

        Ok(FontFamilyFile {
            line_height,
            ascent,
            descent,
            regular: BakedFont { atlas, glyphs, edge_softness: 0.15 },
            bold: None,
            italic: None,
            bold_italic: None,
            ttf_source: Some(regular_bytes.to_vec()),
            is_bitmap: false,
        })
    }

    pub fn with_bold(mut self, bytes: &[u8]) -> OpticResult<Self> {
        let baked = bake_font_variant(bytes, &self.regular.glyphs, ATLAS_SIZE)?;
        self.bold = Some(baked);
        Ok(self)
    }

    pub fn with_italic(mut self, bytes: &[u8]) -> OpticResult<Self> {
        let baked = bake_font_variant(bytes, &self.regular.glyphs, ATLAS_SIZE)?;
        self.italic = Some(baked);
        Ok(self)
    }

    pub fn with_bold_italic(mut self, bytes: &[u8]) -> OpticResult<Self> {
        let baked = bake_font_variant(bytes, &self.regular.glyphs, ATLAS_SIZE)?;
        self.bold_italic = Some(baked);
        Ok(self)
    }

    pub fn from_bitmap_layout(layout: BitmapFontLayout) -> OpticResult<Self> {
        let tile_w = layout.glyph_size.w as f32;
        let tile_h = layout.glyph_size.h as f32;
        let cells_per_row = layout.columns;
        let _num_glyphs = layout.codepoint_order.len();

        let atlas_size = 512u32;
        let per_row = (atlas_size as f32 / tile_w.max(tile_h)).floor() as u32;
        let cell_px = atlas_size as f32 / per_row as f32;
        let mut atlas_pixels = vec![0u8; (atlas_size * atlas_size * 3) as usize];
        let mut glyphs = HashMap::new();

        for (i, &codepoint) in layout.codepoint_order.iter().enumerate() {
            if i >= (per_row * per_row) as usize { break; }
            let src_col = (i as u32) % cells_per_row;
            let src_row = (i as u32) / cells_per_row;

            let tile_x = src_col * layout.glyph_size.w;
            let tile_y = src_row * layout.glyph_size.h;

            let mut tile_bmp = vec![0u8; (tile_w as u32 * tile_h as u32) as usize];
            let src_tex = &layout.texture.bytes;
            let src_w = layout.texture.size.w as usize;
            for ty in 0..tile_h as u32 {
                for tx in 0..tile_w as u32 {
                    let si = ((tile_y + ty) as usize * src_w + (tile_x + tx) as usize) * 4;
                    let di = (ty * tile_w as u32 + tx) as usize;
                    tile_bmp[di] = if si + 3 < src_tex.len() && src_tex[si + 3] > 128 { 255 } else { 0 };
                }
            }

            let sdf_data = bake_sdf_from_bitmap(&tile_bmp, tile_w as u32, tile_h as u32, cell_px as u32, 2.0);

            let dst_col = (i as u32) % per_row;
            let dst_row = (i as u32) / per_row;
            let dx = dst_col * cell_px as u32;
            let dy = dst_row * cell_px as u32;

            for gy in 0..cell_px as u32 {
                for gx in 0..cell_px as u32 {
                    let si = (gy * cell_px as u32 + gx) as usize * 3;
                    let di = ((dy + gy) * atlas_size + (dx + gx)) as usize * 3;
                    if di + 2 < atlas_pixels.len() && si + 2 < sdf_data.len() {
                        atlas_pixels[di] = sdf_data[si];
                        atlas_pixels[di + 1] = sdf_data[si + 1];
                        atlas_pixels[di + 2] = sdf_data[si + 2];
                    }
                }
            }

            let u0 = dst_col as f32 / per_row as f32;
            let v0 = dst_row as f32 / per_row as f32;
            let u1 = (dst_col + 1) as f32 / per_row as f32;
            let v1 = (dst_row + 1) as f32 / per_row as f32;

            let advance = layout.advance.unwrap_or(layout.glyph_size.w) as f32;

            glyphs.insert(codepoint, GlyphMetrics {
                uv_rect: (u0, v0, u1, v1),
                size: layout.glyph_size,
                bearing: (0.0, 0.0),
                advance,
            });
        }

        let atlas = TextureFile {
            bytes: atlas_pixels,
            size: Size2D::new(atlas_size, atlas_size),
            fmt: optic_core::ImgFormat::RGBA(8),
            filter: optic_core::ImgFilter::Linear,
            wrap: optic_core::ImgWrap::Clip,
        };

        Ok(FontFamilyFile {
            line_height: tile_h,
            ascent: tile_h,
            descent: 0.0,
            regular: BakedFont { atlas, glyphs, edge_softness: 0.01 },
            bold: None,
            italic: None,
            bold_italic: None,
            ttf_source: None,
            is_bitmap: true,
        })
    }

    pub fn from_disk(path: &str) -> OpticResult<Self> {
        if path.ends_with(OFONT) {
            return Self::from_cached(path);
        }
        let cached = optic_file::cached_path(path, OFONT);
        if optic_file::exists(&cached) {
            return Self::from_cached(&cached);
        }
        let bytes = optic_file::read_bytes(path)?;
        let family = Self::from_ttf_file(&bytes, (32, 126), 512)?;
        if let Some(parent) = std::path::Path::new(&cached).parent() {
            let _ = optic_file::create_dir(&parent.to_string_lossy());
        }
        family.save_cached(&cached)?;
        Ok(family)
    }

    pub fn save_cached(&self, path: &str) -> OpticResult<()> {
        let mut data = Vec::new();
        data.extend_from_slice(&OPTIC_MAGIC);
        data.extend_from_slice(&OPTIC_CACHE_VERSION.to_le_bytes());
        data.extend_from_slice(&self.line_height.to_le_bytes());
        data.extend_from_slice(&self.ascent.to_le_bytes());
        data.extend_from_slice(&self.descent.to_le_bytes());

        data.push(if self.is_bitmap { 1 } else { 0 });
        data.push(if self.ttf_source.is_some() { 1 } else { 0 });

        let mut style_flags = STYLE_REGULAR;
        if self.bold.is_some() { style_flags |= STYLE_BOLD; }
        if self.italic.is_some() { style_flags |= STYLE_ITALIC; }
        if self.bold_italic.is_some() { style_flags |= STYLE_BOLD_ITALIC; }
        data.push(style_flags);

        if let Some(src) = &self.ttf_source {
            data.extend_from_slice(&(src.len() as u32).to_le_bytes());
            data.extend_from_slice(src);
        } else {
            data.extend_from_slice(&0u32.to_le_bytes());
        }

        write_baked_font(&mut data, &self.regular)?;
        if let Some(b) = &self.bold { write_baked_font(&mut data, b)?; }
        if let Some(b) = &self.italic { write_baked_font(&mut data, b)?; }
        if let Some(b) = &self.bold_italic { write_baked_font(&mut data, b)?; }

        optic_file::write_bytes(path, &data)
    }

    pub fn from_cached(path: &str) -> OpticResult<Self> {
        let data = optic_file::read_bytes(path)?;
        let header = 8 + 2 + 12 + 2;
        if data.len() < header + 1 {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("cached font too short: {path}")));
        }
        if data[..8] != OPTIC_MAGIC {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("invalid font cache magic: {path}")));
        }
        let version = u16::from_le_bytes([data[8], data[9]]);
        if version != OPTIC_CACHE_VERSION {
            return Err(OpticError::new(OpticErrorKind::Custom, &format!("unsupported font cache version {version} in {path}")));
        }

        let mut offset = 10usize;
        let line_height = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;
        let ascent = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;
        let descent = f32::from_le_bytes(data[offset..offset+4].try_into().unwrap());
        offset += 4;

        let is_bitmap = data[offset] != 0;
        offset += 1;
        let has_ttf = data[offset] != 0;
        offset += 1;
        let flags = data[offset];
        offset += 1;

        let ttf_source = if has_ttf {
            if data.len() < offset + 4 {
                return Err(OpticError::new(OpticErrorKind::Custom, &format!("cached font missing ttf length: {path}")));
            }
            let len = u32::from_le_bytes(data[offset..offset+4].try_into().unwrap()) as usize;
            offset += 4;
            if data.len() < offset + len {
                return Err(OpticError::new(OpticErrorKind::Custom, &format!("cached font ttf truncated: {path}")));
            }
            let src = data[offset..offset+len].to_vec();
            offset += len;
            Some(src)
        } else {
            offset += 4;
            None
        };

        let regular = read_baked_font(&data, &mut offset, path)?;
        let bold = if flags & STYLE_BOLD != 0 { Some(read_baked_font(&data, &mut offset, path)?) } else { None };
        let italic = if flags & STYLE_ITALIC != 0 { Some(read_baked_font(&data, &mut offset, path)?) } else { None };
        let bold_italic = if flags & STYLE_BOLD_ITALIC != 0 { Some(read_baked_font(&data, &mut offset, path)?) } else { None };

        Ok(FontFamilyFile { line_height, ascent, descent, regular, bold, italic, bold_italic, ttf_source, is_bitmap })
    }

    pub fn fallback() -> OpticResult<Self> {
        fallback_font_family()
    }

    pub fn units_per_em(&self) -> f32 {
        if self.is_bitmap {
            self.line_height
        } else {
            1.0
        }
    }
}

fn bake_font_variant(bytes: &[u8], template: &HashMap<u32, GlyphMetrics>, atlas_resolution: u32) -> OpticResult<BakedFont> {
    let face = ttf_parser::Face::parse(bytes, 0)
        .map_err(|_| OpticError::new(OpticErrorKind::Custom, "failed to parse TTF font variant"))?;

    let atlas_size = atlas_resolution;
    let per_row = (atlas_size as f32 / 64.0).floor() as u32;
    let glyph_cell = atlas_size / per_row;

    let mut glyphs = HashMap::new();
    let mut atlas_pixels = vec![0u8; (atlas_size * atlas_size * 3) as usize];
    let mut idx = 0u32;

    let mut sorted_gids: Vec<_> = template.keys().collect();
    sorted_gids.sort();

    for &&gid in &sorted_gids {
        if idx >= per_row * per_row { break; }
        let gid_u16 = gid as u16;
        if let Some(edges) = extract_glyph_edges(&face, gid_u16) {
            let glyph_data = bake_msdf(&edges, glyph_cell, PX_RANGE);

            let row = idx / per_row;
            let col = idx % per_row;
            let dx = col * glyph_cell;
            let dy = row * glyph_cell;

            for gy in 0..glyph_cell as usize {
                for gx in 0..glyph_cell as usize {
                    let src = (gy * glyph_cell as usize + gx) * 3;
                    let dst = ((dy + gy as u32) * atlas_size + (dx + gx as u32)) as usize * 3;
                    if dst + 2 < atlas_pixels.len() && src + 2 < glyph_data.len() {
                        atlas_pixels[dst] = glyph_data[src];
                        atlas_pixels[dst + 1] = glyph_data[src + 1];
                        atlas_pixels[dst + 2] = glyph_data[src + 2];
                    }
                }
            }

            let u0 = col as f32 / per_row as f32;
            let v0 = row as f32 / per_row as f32;
            let u1 = (col + 1) as f32 / per_row as f32;
            let v1 = (row + 1) as f32 / per_row as f32;

            if let Some(tmpl) = template.get(&gid) {
                glyphs.insert(gid, GlyphMetrics {
                    uv_rect: (u0, v0, u1, v1),
                    size: tmpl.size,
                    bearing: (edges.bearing_x, edges.bearing_y),
                    advance: edges.advance,
                });
            }

            idx += 1;
        }
    }

    let atlas = TextureFile {
        bytes: atlas_pixels,
        size: Size2D::new(atlas_size, atlas_size),
        fmt: optic_core::ImgFormat::RGBA(8),
        filter: optic_core::ImgFilter::Linear,
        wrap: optic_core::ImgWrap::Clip,
    };

    Ok(BakedFont { atlas, glyphs, edge_softness: 0.15 })
}

fn encode_texture(tex: &TextureFile) -> Vec<u8> {
    let mut data = Vec::with_capacity(12 + tex.bytes.len());
    data.push(tex.fmt.channels());
    data.push(tex.fmt.bit_depth());
    data.extend_from_slice(&(tex.size.w as u32).to_le_bytes());
    data.extend_from_slice(&(tex.size.h as u32).to_le_bytes());
    data.push(match tex.filter {
        optic_core::ImgFilter::Closest => 0u8,
        optic_core::ImgFilter::Linear => 1u8,
    });
    data.push(match tex.wrap {
        optic_core::ImgWrap::Repeat => 0u8,
        optic_core::ImgWrap::Extend => 1u8,
        optic_core::ImgWrap::Clip => 2u8,
    });
    data.extend_from_slice(&tex.bytes);
    data
}

fn decode_texture(data: &[u8], path: &str) -> OpticResult<(TextureFile, usize)> {
    if data.len() < 12 {
        return Err(OpticError::new(OpticErrorKind::Asset, &format!("cached font texture too short: {path}")));
    }
    let channels = data[0];
    let bit_depth = data[1];
    let w = u32::from_le_bytes(data[2..6].try_into().unwrap());
    let h = u32::from_le_bytes(data[6..10].try_into().unwrap());
    let filter = match data[10] {
        0 => optic_core::ImgFilter::Closest,
        _ => optic_core::ImgFilter::Linear,
    };
    let wrap = match data[11] {
        0 => optic_core::ImgWrap::Repeat,
        1 => optic_core::ImgWrap::Extend,
        _ => optic_core::ImgWrap::Clip,
    };
    let expected = w as usize * h as usize * channels as usize * (bit_depth as usize / 8);
    if data.len() < 12 + expected {
        return Err(OpticError::new(OpticErrorKind::Asset, &format!("cached font texture size mismatch for {path}")));
    }
    let bytes = data[12..12 + expected].to_vec();
    Ok((
        TextureFile {
            bytes,
            size: Size2D::new(w, h),
            fmt: optic_core::ImgFormat::new(channels, bit_depth),
            filter,
            wrap,
        },
        12 + expected,
    ))
}

fn write_baked_font(buf: &mut Vec<u8>, baked: &BakedFont) -> OpticResult<()> {
    buf.extend_from_slice(&encode_texture(&baked.atlas));
    buf.extend_from_slice(&baked.edge_softness.to_le_bytes());
    buf.extend_from_slice(&(baked.glyphs.len() as u32).to_le_bytes());
    let mut gids: Vec<u32> = baked.glyphs.keys().copied().collect();
    gids.sort_unstable();
    for gid in gids {
        let m = baked.glyphs.get(&gid).unwrap();
        buf.extend_from_slice(&gid.to_le_bytes());
        buf.extend_from_slice(&m.uv_rect.0.to_le_bytes());
        buf.extend_from_slice(&m.uv_rect.1.to_le_bytes());
        buf.extend_from_slice(&m.uv_rect.2.to_le_bytes());
        buf.extend_from_slice(&m.uv_rect.3.to_le_bytes());
        buf.extend_from_slice(&m.size.w.to_le_bytes());
        buf.extend_from_slice(&m.size.h.to_le_bytes());
        buf.extend_from_slice(&m.bearing.0.to_le_bytes());
        buf.extend_from_slice(&m.bearing.1.to_le_bytes());
        buf.extend_from_slice(&m.advance.to_le_bytes());
    }
    Ok(())
}

fn read_baked_font(data: &[u8], offset: &mut usize, path: &str) -> OpticResult<BakedFont> {
    let tail = &data[*offset..];
    let (atlas, consumed) = decode_texture(tail, path)?;
    *offset += consumed;
    if data.len() < *offset + 8 {
        return Err(OpticError::new(OpticErrorKind::Asset, &format!("cached font style truncated: {path}")));
    }
    let edge_softness = f32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
    *offset += 4;
    let glyph_count = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap()) as usize;
    *offset += 4;

    let mut glyphs = HashMap::new();
    for _ in 0..glyph_count {
        if data.len() < *offset + 40 {
            return Err(OpticError::new(OpticErrorKind::Asset, &format!("cached font glyph truncated: {path}")));
        }
        let gid = u32::from_le_bytes(data[*offset..*offset + 4].try_into().unwrap());
        let u0 = f32::from_le_bytes(data[*offset + 4..*offset + 8].try_into().unwrap());
        let v0 = f32::from_le_bytes(data[*offset + 8..*offset + 12].try_into().unwrap());
        let u1 = f32::from_le_bytes(data[*offset + 12..*offset + 16].try_into().unwrap());
        let v1 = f32::from_le_bytes(data[*offset + 16..*offset + 20].try_into().unwrap());
        let w = u32::from_le_bytes(data[*offset + 20..*offset + 24].try_into().unwrap());
        let h = u32::from_le_bytes(data[*offset + 24..*offset + 28].try_into().unwrap());
        let bx = f32::from_le_bytes(data[*offset + 28..*offset + 32].try_into().unwrap());
        let by = f32::from_le_bytes(data[*offset + 32..*offset + 36].try_into().unwrap());
        let advance = f32::from_le_bytes(data[*offset + 36..*offset + 40].try_into().unwrap());
        *offset += 40;
        glyphs.insert(
            gid,
            GlyphMetrics {
                uv_rect: (u0, v0, u1, v1),
                size: Size2D::new(w, h),
                bearing: (bx, by),
                advance,
            },
        );
    }

    Ok(BakedFont { atlas, glyphs, edge_softness })
}

fn fallback_font_family() -> OpticResult<FontFamilyFile> {
    let glyph_w = 8u32;
    let glyph_h = 8u32;
    let columns = 16u32;
    let atlas_w = glyph_w * columns;
    let atlas_h = glyph_h * 8;
    let mut rgba = vec![0u8; (atlas_w * atlas_h * 4) as usize];

    for cp in 32u32..=126 {
        let idx = (cp - 32) as usize;
        let col = (idx % columns as usize) as u32;
        let row = (idx / columns as usize) as u32;
        let pattern = fallback_glyph_pattern(cp);
        for y in 0..glyph_h {
            for x in 0..glyph_w {
                let bit = (pattern[y as usize] >> (7 - x)) & 1;
                let px = ((row * glyph_h + y) * atlas_w + col * glyph_w + x) as usize;
                let alpha = if bit != 0 { 255 } else { 0 };
                rgba[px * 4] = 255;
                rgba[px * 4 + 1] = 255;
                rgba[px * 4 + 2] = 255;
                rgba[px * 4 + 3] = alpha;
            }
        }
    }

    let codepoint_order: Vec<u32> = (32..=126).collect();
    let layout = BitmapFontLayout {
        texture: TextureFile {
            bytes: rgba,
            size: Size2D::new(atlas_w, atlas_h),
            fmt: optic_core::ImgFormat::RGBA(8),
            filter: optic_core::ImgFilter::Closest,
            wrap: optic_core::ImgWrap::Clip,
        },
        glyph_size: Size2D::new(glyph_w, glyph_h),
        columns,
        codepoint_order,
        advance: Some(glyph_w),
    };
    FontFamilyFile::from_bitmap_layout(layout)
}

fn fallback_glyph_pattern(cp: u32) -> [u8; 8] {
    match cp {
        32 => [0; 8],
        33 => [0x18, 0x3C, 0x3C, 0x18, 0x18, 0x00, 0x18, 0x00],
        48..=57 => [0x3C, 0x66, 0x6E, 0x76, 0x66, 0x66, 0x3C, 0x00],
        65..=90 => [0x18, 0x3C, 0x66, 0x66, 0x7E, 0x66, 0x66, 0x00],
        97..=122 => [0x00, 0x00, 0x3C, 0x06, 0x3E, 0x66, 0x3E, 0x00],
        _ => [0x7E, 0x81, 0xA5, 0x81, 0xBD, 0x99, 0x81, 0x7E],
    }
}
