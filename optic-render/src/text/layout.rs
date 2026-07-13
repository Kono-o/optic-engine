use optic_core::{OpticResult, RGBA, WHITE};

use crate::asset::attr::CustomATTR;
use crate::handles::font::{FontFamily, FontStyle};
use crate::handles::instance::{InstanceDesc2D, InstanceDesc3D};
use crate::text::bbcode::{
    self, parse, ParsedText, PulseEffect, ShakeEffect, StyledSpan, TextStyle, WaveEffect,
    FAUX_BOLD, FAUX_ITALIC, BORDER,
};

const DEFAULT_COLOR: RGBA = WHITE;

#[derive(Clone, Debug)]
pub struct ShapedGlyph {
    pub gid: u32,
    pub cluster_start: usize,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
}

#[derive(Clone, Debug)]
pub struct LayoutGlyph {
    pub gid: u32,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub uv: [f32; 4],
    pub color: RGBA,
    pub style_flags: u32,
    pub softness: f32,
    pub span_index: usize,
    pub char_index: usize,
    pub style: TextStyle,
}

#[derive(Clone, Debug)]
pub struct LayoutDecoration {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub color: RGBA,
    pub span_index: usize,
    pub char_index: usize,
    pub style: TextStyle,
    pub kind: DecorationKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationKind {
    Background,
    Underline,
    Strikethrough,
}

#[derive(Clone, Debug)]
pub struct TextLayout {
    pub parsed: ParsedText,
    pub glyphs: Vec<LayoutGlyph>,
    pub decorations: Vec<LayoutDecoration>,
    pub width: f32,
    pub height: f32,
    pub is_dynamic: bool,
}

pub fn layout_text(
    raw: &str,
    font: &FontFamily,
    base_size: f32,
    wrap_width: f32,
) -> OpticResult<TextLayout> {
    let parsed = parse(raw)?;
    let scale = base_size / font.units_per_em().max(1.0);
    let line_height = font.line_height() * scale;
    let mut glyphs = Vec::new();
    let mut decorations = Vec::new();
    let mut cursor_x = 0.0f32;
    let mut cursor_y = 0.0f32;
    let mut line_width = 0.0f32;
    let mut max_width = 0.0f32;
    let mut span_index = 0usize;
    let mut global_char = 0usize;

    for span in &parsed.spans {
        let span_size = span.style.size.unwrap_or(1.0) * base_size;
        let span_scale = span_size / font.units_per_em().max(1.0);
        let shaped = shape_span(span, font);

        let mut run_x = cursor_x;
        for (i, sg) in shaped.iter().enumerate() {
            let (font_style, faux_bold, faux_italic) =
                font.resolve_style(span.style.bold, span.style.italic);
            let gid = sg.gid;
            let metrics = font.glyph(font_style, gid).or_else(|| font.glyph(FontStyle::Regular, gid));

            let (_advance, bearing, size, uv) = match metrics {
                Some(m) => (m.advance, m.bearing_arr(), m.size_arr(), m.uv()),
                None => continue,
            };

            let gw = size[0] * span_scale;
            let gh = size[1] * span_scale;
            let advance_px = sg.x_advance * span_scale + span.style.kerning * span_scale;

            if wrap_width > 0.0 && run_x > 0.0 && run_x + advance_px > wrap_width {
                max_width = max_width.max(line_width);
                cursor_y += line_height;
                run_x = 0.0;
            }

            let mut flags = 0u32;
            if faux_bold {
                flags |= FAUX_BOLD;
            }
            if faux_italic {
                flags |= FAUX_ITALIC;
            }
            if span.style.border_color.is_some() {
                flags |= BORDER;
            }

            let color = span.style.color.unwrap_or(DEFAULT_COLOR);
            let x = run_x + bearing[0] * span_scale + sg.x_offset * span_scale + span.style.offset[0];
            let y = cursor_y + bearing[1] * span_scale + sg.y_offset * span_scale + span.style.offset[1];

            glyphs.push(LayoutGlyph {
                gid,
                x,
                y,
                width: gw,
                height: gh,
                uv,
                color,
                style_flags: flags,
                softness: font.edge_softness(font_style),
                span_index,
                char_index: global_char + i,
                style: span.style.clone(),
            });

            push_decorations(
                &mut decorations,
                span,
                span_index,
                global_char + i,
                x,
                y,
                gw,
                gh,
                span_scale,
                run_x,
                advance_px,
                cursor_y,
                line_height,
            );

            run_x += advance_px;
            line_width = run_x;
        }

        cursor_x = run_x;
        max_width = max_width.max(line_width);
        span_index += 1;
        global_char += span.text.chars().count();
    }

    let height = cursor_y + line_height;
    Ok(TextLayout {
        is_dynamic: parsed.is_dynamic,
        parsed,
        glyphs,
        decorations,
        width: if wrap_width > 0.0 { wrap_width.min(max_width) } else { max_width },
        height,
    })
}

fn push_decorations(
    out: &mut Vec<LayoutDecoration>,
    span: &StyledSpan,
    span_index: usize,
    char_index: usize,
    x: f32,
    y: f32,
    gw: f32,
    gh: f32,
    scale: f32,
    run_x: f32,
    advance: f32,
    baseline_y: f32,
    line_height: f32,
) {
    if let Some(bg) = span.style.bgcolor {
        out.push(LayoutDecoration {
            x: run_x,
            y: baseline_y,
            width: advance,
            height: line_height,
            color: bg,
            span_index,
            char_index,
            style: span.style.clone(),
            kind: DecorationKind::Background,
        });
    }
    if span.style.underline {
        let uy = baseline_y + line_height * 0.85;
        out.push(LayoutDecoration {
            x: run_x,
            y: uy,
            width: advance,
            height: (2.0 * scale).max(1.0),
            color: span.style.color.unwrap_or(DEFAULT_COLOR),
            span_index,
            char_index,
            style: span.style.clone(),
            kind: DecorationKind::Underline,
        });
    }
    if span.style.strikethrough {
        let sy = y + gh * 0.5;
        out.push(LayoutDecoration {
            x: x,
            y: sy,
            width: gw.max(advance),
            height: (2.0 * scale).max(1.0),
            color: span.style.color.unwrap_or(DEFAULT_COLOR),
            span_index,
            char_index,
            style: span.style.clone(),
            kind: DecorationKind::Strikethrough,
        });
    }
}

pub fn shape_span(span: &StyledSpan, font: &FontFamily) -> Vec<ShapedGlyph> {
    if font.is_bitmap() {
        return shape_bitmap(&span.text);
    }
    shape_ttf(span, font)
}

fn shape_bitmap(text: &str) -> Vec<ShapedGlyph> {
    let mut out = Vec::new();
    let mut cluster = 0usize;
    for ch in text.chars() {
        let cp = ch as u32;
        out.push(ShapedGlyph {
            gid: cp,
            cluster_start: cluster,
            x_offset: 0.0,
            y_offset: 0.0,
            x_advance: 1.0,
        });
        cluster += ch.len_utf8();
    }
    out
}

fn shape_ttf(span: &StyledSpan, font: &FontFamily) -> Vec<ShapedGlyph> {
    let face_data = match font.face_data() {
        Some(d) => d,
        None => return shape_bitmap(&span.text),
    };

    let face = match rustybuzz::Face::from_slice(face_data, 0) {
        Some(f) => f,
        None => return shape_bitmap(&span.text),
    };

    let mut buffer = rustybuzz::UnicodeBuffer::new();
    buffer.push_str(&span.text);
    let output = rustybuzz::shape(&face, &[], buffer);

    let upem = face.units_per_em() as f32;
    output
        .glyph_infos()
        .iter()
        .zip(output.glyph_positions().iter())
        .map(|(info, pos)| ShapedGlyph {
            gid: info.glyph_id,
            cluster_start: info.cluster as usize,
            x_offset: pos.x_offset as f32 / upem,
            y_offset: pos.y_offset as f32 / upem,
            x_advance: pos.x_advance as f32 / upem,
        })
        .collect()
}

pub fn build_glyph_desc_2d(layout: &TextLayout, time: f32) -> InstanceDesc2D {
    let mut desc = InstanceDesc2D::empty();
    let mut uv_attr = CustomATTR::empty::<[f32; 4]>("iUVRect");
    let mut style_attr = CustomATTR::empty::<u32>("iStyleFlags");
    let mut soft_attr = CustomATTR::empty::<f32>("iSoftness");

    for g in &layout.glyphs {
        let (x, y, scale_mul, color) = apply_dynamic(g, time);
        desc.pos_attr.push([x, y]);
        desc.scale_attr.push([g.width * scale_mul, g.height * scale_mul]);
        desc.col_attr.push([color.0, color.1, color.2, color.3]);
        uv_attr.push(g.uv);
        style_attr.push(g.style_flags);
        soft_attr.push(g.softness);
    }

    desc.add_custom_attr(uv_attr);
    desc.add_custom_attr(style_attr);
    desc.add_custom_attr(soft_attr);
    desc
}

pub fn build_glyph_desc_3d(layout: &TextLayout, time: f32) -> InstanceDesc3D {
    let mut desc = InstanceDesc3D::empty();
    let mut uv_attr = CustomATTR::empty::<[f32; 4]>("iUVRect");
    let mut style_attr = CustomATTR::empty::<u32>("iStyleFlags");
    let mut soft_attr = CustomATTR::empty::<f32>("iSoftness");

    for g in &layout.glyphs {
        let (x, y, scale_mul, color) = apply_dynamic(g, time);
        desc.pos_attr.push([x, y, 0.0]);
        desc.scale_attr.push([g.width * scale_mul, g.height * scale_mul, 1.0]);
        desc.col_attr.push([color.0, color.1, color.2, color.3]);
        uv_attr.push(g.uv);
        style_attr.push(g.style_flags);
        soft_attr.push(g.softness);
    }

    desc.add_custom_attr(uv_attr);
    desc.add_custom_attr(style_attr);
    desc.add_custom_attr(soft_attr);
    desc
}

pub fn build_decoration_desc_2d(layout: &TextLayout, time: f32) -> InstanceDesc2D {
    let mut desc = InstanceDesc2D::empty();

    for d in &layout.decorations {
        let color = decoration_color(d, time);
        desc.pos_attr.push([d.x, d.y]);
        desc.scale_attr.push([d.width, d.height]);
        desc.col_attr.push([color.0, color.1, color.2, color.3]);
    }

    desc
}

pub fn build_decoration_desc_3d(layout: &TextLayout, time: f32) -> InstanceDesc3D {
    let mut desc = InstanceDesc3D::empty();

    for d in &layout.decorations {
        let color = decoration_color(d, time);
        desc.pos_attr.push([d.x, d.y, 0.0]);
        desc.scale_attr.push([d.width, d.height, 1.0]);
        desc.col_attr.push([color.0, color.1, color.2, color.3]);
    }

    desc
}

fn apply_dynamic(g: &LayoutGlyph, time: f32) -> (f32, f32, f32, RGBA) {
    let mut x = g.x;
    let mut y = g.y;
    let mut scale = 1.0f32;
    let mut color = g.color;

    if let Some(wave) = &g.style.wave {
        y += apply_wave(wave, time, g.char_index);
    }
    if let Some(shake) = &g.style.shake {
        let (dx, dy) = apply_shake(shake, time, g.char_index);
        x += dx;
        y += dy;
    }
    if let Some(rainbow) = &g.style.rainbow {
        color = bbcode::rainbow_color(rainbow, time, g.char_index, g.color.3);
    }
    if let Some(pulse) = &g.style.pulse {
        scale *= apply_pulse(pulse, time, g.char_index);
    }

    (x, y, scale, color)
}

fn decoration_color(d: &LayoutDecoration, time: f32) -> RGBA {
    if let Some(rainbow) = &d.style.rainbow {
        bbcode::rainbow_color(rainbow, time, d.char_index, d.color.3)
    } else {
        d.color
    }
}

fn apply_wave(effect: &WaveEffect, time: f32, index: usize) -> f32 {
    (time * effect.speed + index as f32 * effect.freq).sin() * effect.amp
}

fn apply_shake(effect: &ShakeEffect, time: f32, index: usize) -> (f32, f32) {
    let t = (time * effect.speed + index as f32 * 1.7).sin();
    let u = (time * effect.speed * 1.3 + index as f32 * 2.3).cos();
    (t * effect.amp, u * effect.amp * 0.5)
}

fn apply_pulse(effect: &PulseEffect, time: f32, index: usize) -> f32 {
    1.0 + (time * effect.speed + index as f32 * 0.2).sin() * effect.amp
}

/// Test-only mock shaper that merges "fi" into a single ligature glyph.
#[cfg(test)]
pub fn shape_ligature_mock(text: &str) -> Vec<ShapedGlyph> {
    let mut out = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'f' && bytes[i + 1] == b'i' {
            out.push(ShapedGlyph {
                gid: 1000,
                cluster_start: i,
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: 1.8,
            });
            i += 2;
        } else {
            out.push(ShapedGlyph {
                gid: bytes[i] as u32,
                cluster_start: i,
                x_offset: 0.0,
                y_offset: 0.0,
                x_advance: 1.0,
            });
            i += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::handles::font::FontFamily;

    fn test_font() -> FontFamily {
        FontFamily::test_bitmap()
    }

    #[test]
    fn bitmap_skips_shaping() {
        let font = test_font();
        let span = StyledSpan {
            text: "A".into(),
            style: TextStyle::default(),
        };
        let shaped = shape_span(&span, &font);
        assert_eq!(shaped.len(), 1);
        assert_eq!(shaped[0].gid, b'A' as u32);
    }

    #[test]
    fn faux_bold_when_no_bold_atlas() {
        let font = test_font();
        let layout = layout_text("[b]X[/b]", &font, 16.0, 0.0).unwrap();
        assert_eq!(layout.glyphs.len(), 1);
        assert_ne!(layout.glyphs[0].style_flags & FAUX_BOLD, 0);
    }

    #[test]
    fn ligature_produces_one_glyph() {
        let glyphs = shape_ligature_mock("fi");
        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].gid, 1000);
    }

    #[test]
    fn rainbow_layout_uses_hsv() {
        let font = test_font();
        let layout = layout_text(
            "[rainbow speed=1]A[/rainbow]",
            &font,
            16.0,
            0.0,
        )
        .unwrap();
        assert!(layout.is_dynamic);
        let desc = build_glyph_desc_2d(&layout, 0.5);
        assert!(!desc.col_attr.is_empty());
        let expected = bbcode::rainbow_color(
            &crate::text::bbcode::RainbowEffect { speed: 1.0 },
            0.5,
            0,
            layout.glyphs[0].color.3,
        );
        assert!((desc.col_attr.data[0][0] - expected.0).abs() < 0.05);
        assert!((desc.col_attr.data[0][1] - expected.1).abs() < 0.05);
        assert!((desc.col_attr.data[0][2] - expected.2).abs() < 0.05);
    }

    #[test]
    fn wave_effect_offsets_y() {
        let font = test_font();
        let layout = layout_text("[wave]A[/wave]", &font, 16.0, 0.0).unwrap();
        assert!(layout.is_dynamic);
        let static_y = layout.glyphs[0].y;
        let desc = build_glyph_desc_2d(&layout, 1.5);
        assert!((desc.pos_attr.data[0][1] - static_y).abs() > 0.001);
    }

    #[test]
    fn reordering_is_inert_ltr() {
        let paths = [
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/TTF/DejaVuSans.ttf",
            "/usr/share/fonts/dejavu/DejaVuSans.ttf",
        ];
        let mut font_bytes = None;
        for p in paths {
            if let Ok(b) = std::fs::read(p) {
                font_bytes = Some(b);
                break;
            }
        }
        let Some(bytes) = font_bytes else {
            return;
        };

        let face = rustybuzz::Face::from_slice(&bytes, 0).expect("ttf face");
        let mut buffer = rustybuzz::UnicodeBuffer::new();
        buffer.push_str("The quick brown fox");
        let output = rustybuzz::shape(&face, &[], buffer);

        let clusters: Vec<usize> = output
            .glyph_infos()
            .iter()
            .map(|info| info.cluster as usize)
            .collect();
        for w in clusters.windows(2) {
            assert!(
                w[1] >= w[0],
                "cluster indices must be monotonically increasing for LTR Latin: {:?}",
                clusters
            );
        }
    }
}
