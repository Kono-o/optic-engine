//! Multi-channel Signed Distance Field (MSDF) font atlas generation.
//!
//! MSDF is a technique for encoding font outlines into a three-channel texture
//! that preserves sharp corners at any resolution. Each pixel stores a signed
//! distance in the R, G, and B channels, classified by the nearest edge's
//! normal direction. This allows the shader to reconstruct crisp edges without
//! relying on bilinear filtering alone.
//!
//! # Pipeline
//!
//! 1. [`extract_glyph_edges`] converts a TrueType glyph outline into a list of
//!    [`Contour`]s made of [`EdgeSegment`]s.
//! 2. [`bake_msdf`] rasterises the edge data into an MSDF texture atlas.
//! 3. [`bake_sdf_from_bitmap`] converts a bitmap glyph into a single-channel
//!    SDF as a fallback when vector outlines are unavailable.
//! 4. [`trace_bitmap_to_contour`] traces a binary bitmap into [`Contour`]s for
//!    round-tripping bitmap fonts through the SDF pipeline.
//!
//! # References
//!
//! - Viktor Chlumský, *Multi-channel signed distance field generation* (2015)
//! - <https://github.com/Chlumsky/msdfgen>

use ttf_parser::Face;

/// A single edge segment within a glyph contour.
///
/// Edges are the building blocks of MSDF generation. Each variant stores the
/// control points needed to evaluate the exact signed distance from any point
/// to the curve.
#[derive(Clone, Debug)]
pub enum EdgeSegment {
    Line { from: (f32, f32), to: (f32, f32) },
    Quadratic { from: (f32, f32), ctrl: (f32, f32), to: (f32, f32) },
    Cubic { from: (f32, f32), ctrl1: (f32, f32), ctrl2: (f32, f32), to: (f32, f32) },
}

/// A closed contour (outline) composed of consecutive edge segments.
///
/// A glyph outline is typically made of one or more contours. Each contour's
/// edges form a closed loop — the last edge's endpoint connects back to the
/// first edge's start point.
#[derive(Clone, Debug)]
pub struct Contour {
    /// The ordered edge segments forming this contour.
    pub edges: Vec<EdgeSegment>,
}

/// Full outline data for a single glyph, extracted from a TrueType face.
///
/// Contains the contours (closed outlines) as well as the horizontal advance
/// and bearing measurements needed for typesetting. All coordinates are in
/// normalised font units (divided by `units_per_em`).
pub struct GlyphEdges {
    /// Closed contours making up the glyph outline.
    pub contours: Vec<Contour>,
    /// Horizontal advance in normalised font units.
    pub advance: f32,
    /// Left bearing (distance from the origin to the leftmost edge) in normalised font units.
    pub bearing_x: f32,
    /// Top bearing (distance from the baseline to the topmost edge) in normalised font units.
    pub bearing_y: f32,
    /// Bounding-box width in normalised font units.
    pub width: f32,
    /// Bounding-box height in normalised font units.
    pub height: f32,
}

#[derive(Clone, Copy, Debug)]
struct EdgeColor(u8);

const COLOR_R: EdgeColor = EdgeColor(0);
const COLOR_G: EdgeColor = EdgeColor(1);
const COLOR_B: EdgeColor = EdgeColor(2);

fn cross(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.1 - a.1 * b.0
}

fn dot(a: (f32, f32), b: (f32, f32)) -> f32 {
    a.0 * b.0 + a.1 * b.1
}

fn sub(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 - b.0, a.1 - b.1)
}

fn add(a: (f32, f32), b: (f32, f32)) -> (f32, f32) {
    (a.0 + b.0, a.1 + b.1)
}

fn scale(a: (f32, f32), s: f32) -> (f32, f32) {
    (a.0 * s, a.1 * s)
}

fn length(a: (f32, f32)) -> f32 {
    (a.0 * a.0 + a.1 * a.1).sqrt()
}

fn normalize(a: (f32, f32)) -> (f32, f32) {
    let len = length(a);
    if len > 0.0 { (a.0 / len, a.1 / len) } else { (0.0, 0.0) }
}

fn angle_between(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dot_val = dot(a, b);
    let cross_val = cross(a, b);
    cross_val.atan2(dot_val)
}

fn solve_quadratic(a: f32, b: f32, c: f32) -> Vec<f32> {
    if a.abs() < 1e-10 {
        if b.abs() < 1e-10 { return vec![]; }
        return vec![-c / b];
    }
    let disc = b * b - 4.0 * a * c;
    if disc < 0.0 { return vec![]; }
    if disc < 1e-10 { return vec![-b / (2.0 * a)]; }
    let sqrt_disc = disc.sqrt();
    vec![(-b - sqrt_disc) / (2.0 * a), (-b + sqrt_disc) / (2.0 * a)]
}

fn point_on_quad(t: f32, from: (f32, f32), ctrl: (f32, f32), to: (f32, f32)) -> (f32, f32) {
    let mt = 1.0 - t;
    add(add(scale(from, mt * mt), scale(ctrl, 2.0 * mt * t)), scale(to, t * t))
}

fn point_on_cubic(t: f32, from: (f32, f32), c1: (f32, f32), c2: (f32, f32), to: (f32, f32)) -> (f32, f32) {
    let mt = 1.0 - t;
    add(add(add(
        scale(from, mt * mt * mt),
        scale(c1, 3.0 * mt * mt * t),
    ), scale(c2, 3.0 * mt * t * t)), scale(to, t * t * t))
}

fn dist_to_segment(p: (f32, f32), from: (f32, f32), to: (f32, f32)) -> (f32, f32) {
    let ab = sub(to, from);
    let ap = sub(p, from);
    let t = dot(ap, ab) / dot(ab, ab);
    let t = t.clamp(0.0, 1.0);
    let closest = add(from, scale(ab, t));
    let d = sub(p, closest);
    let sign = if cross(ab, ap) > 0.0 { 1.0 } else { -1.0 };
    (length(d), sign * length(d))
}

fn dist_to_quad(p: (f32, f32), from: (f32, f32), ctrl: (f32, f32), to: (f32, f32)) -> (f32, f32) {
    let mut min_dist = f32::MAX;
    let mut min_signed = 0.0f32;
    let a = sub(add(scale(from, 2.0), scale(to, 2.0)), scale(ctrl, 4.0));
    let b = sub(scale(ctrl, 4.0), scale(from, 4.0));
    let c = sub(scale(from, 2.0), scale(p, 2.0));
    let ts = solve_quadratic(a.0 + a.1, b.0 + b.1, c.0 + c.1);
    for &t in &ts {
        if t >= 0.0 && t <= 1.0 {
            let pt = point_on_quad(t, from, ctrl, to);
            let d = sub(p, pt);
            let dist = length(d);
            if dist < min_dist {
                min_dist = dist;
                let tangent = if t < 0.5 {
                    sub(ctrl, from)
                } else {
                    sub(to, ctrl)
                };
                let sign = if cross(tangent, d) > 0.0 { 1.0 } else { -1.0 };
                min_signed = sign * dist;
            }
        }
    }
    let (d_lin, s_lin) = dist_to_segment(p, from, ctrl);
    if d_lin < min_dist { min_dist = d_lin; min_signed = s_lin; }
    let (d_lin, s_lin) = dist_to_segment(p, ctrl, to);
    if d_lin < min_dist { min_dist = d_lin; min_signed = s_lin; }
    (min_dist, min_signed)
}

fn dist_to_cubic(p: (f32, f32), from: (f32, f32), c1: (f32, f32), c2: (f32, f32), to: (f32, f32)) -> (f32, f32) {
    let mut min_dist = f32::MAX;
    let mut min_signed = 0.0f32;
    let num_samples = 32;
    for i in 0..=num_samples {
        let t = i as f32 / num_samples as f32;
        let pt = point_on_cubic(t, from, c1, c2, to);
        let d = sub(p, pt);
        let dist = length(d);
        if dist < min_dist {
            min_dist = dist;
            let tangent = sub(point_on_cubic((t + 0.001).min(1.0), from, c1, c2, to), pt);
            let sign = if cross(tangent, d) > 0.0 { 1.0 } else { -1.0 };
            min_signed = sign * dist;
        }
    }
    (min_dist, min_signed)
}

fn dist_to_edge(p: (f32, f32), edge: &EdgeSegment) -> (f32, f32) {
    match edge {
        EdgeSegment::Line { from, to } => dist_to_segment(p, *from, *to),
        EdgeSegment::Quadratic { from, ctrl, to } => dist_to_quad(p, *from, *ctrl, *to),
        EdgeSegment::Cubic { from, ctrl1, ctrl2, to } => dist_to_cubic(p, *from, *ctrl1, *ctrl2, *to),
    }
}

fn is_point_inside(p: (f32, f32), contours: &[Contour]) -> bool {
    let mut winding = 0i32;
    for contour in contours {
        for edge in &contour.edges {
            let (from, to) = match edge {
                EdgeSegment::Line { from, to } => (*from, *to),
                EdgeSegment::Quadratic { from, to, .. } => (*from, *to),
                EdgeSegment::Cubic { from, to, .. } => (*from, *to),
            };
            if from.1 <= p.1 {
                if to.1 > p.1 && cross(sub(to, from), sub(p, from)) > 0.0 {
                    winding += 1;
                }
            } else if to.1 <= p.1 && cross(sub(to, from), sub(p, from)) < 0.0 {
                winding -= 1;
            }
        }
    }
    winding != 0
}

fn is_sharp_corner(a: (f32, f32), b: (f32, f32)) -> bool {
    let angle = angle_between(sub(a, b), sub(b, a));
    angle.abs() > 0.3
}

fn color_edge(contours: &mut [Contour]) {
    for contour in contours.iter_mut() {
        let n = contour.edges.len();
        if n == 0 { continue; }
        let mut colors = vec![0u8; n];
        for i in 0..n {
            let prev = if i == 0 { &contour.edges[n - 1] } else { &contour.edges[i - 1] };
            let curr = &contour.edges[i];
            let (_, prev_to) = edge_endpoints(prev);
            let (curr_from, _) = edge_endpoints(curr);
            if is_sharp_corner(prev_to, curr_from) {
                let mut used = [false; 3];
                if i > 0 { let ci = colors[i - 1]; if ci < 3 { used[ci as usize] = true; } }
                let mut ci = 0u8;
                while ci < 3 && used[ci as usize] { ci += 1; }
                if ci >= 3 { ci = 0; }
                colors[i] = ci;
            } else {
                colors[i] = 0;
            }
        }
        for (i, edge) in contour.edges.iter_mut().enumerate() {
            if let Some(edge_color) = colors.get(i) {
                let _ = edge_color;
            }
        }
    }
}

fn edge_endpoints(edge: &EdgeSegment) -> ((f32, f32), (f32, f32)) {
    match edge {
        EdgeSegment::Line { from, to } => (*from, *to),
        EdgeSegment::Quadratic { from, to, .. } => (*from, *to),
        EdgeSegment::Cubic { from, to, .. } => (*from, *to),
    }
}

fn classify_edge_normal(normal: (f32, f32)) -> u8 {
    let angle = normal.1.atan2(normal.0);
    let angle = angle.to_degrees();
    if angle >= -30.0 && angle < 90.0 { 0 }
    else if angle >= 90.0 && angle < 210.0 { 1 }
    else { 2 }
}

fn edge_midpoint_normal(edge: &EdgeSegment) -> (f32, f32) {
    match edge {
        EdgeSegment::Line { from, to } => {
            let t = normalize(sub(*to, *from));
            (-t.1, t.0)
        }
        EdgeSegment::Quadratic { from, ctrl, to } => {
            let mid = point_on_quad(0.5, *from, *ctrl, *to);
            let tangent = if length(sub(*ctrl, *from)) > 0.0 {
                normalize(sub(*ctrl, *from))
            } else {
                normalize(sub(*to, *ctrl))
            };
            (-tangent.1, tangent.0)
        }
        EdgeSegment::Cubic { from, ctrl1, ctrl2, to } => {
            let mid = point_on_cubic(0.5, *from, *ctrl1, *ctrl2, *to);
            let tangent = normalize(sub(point_on_cubic(0.501, *from, *ctrl1, *ctrl2, *to), mid));
            (-tangent.1, tangent.0)
        }
    }
}

/// Extracts the outline edges and metrics for a single glyph from a TrueType
/// face.
///
/// The returned [`GlyphEdges`] contains closed contours in normalised font
/// units, ready for rasterisation with [`bake_msdf`].
///
/// Returns `None` if the glyph has no outline (e.g. a space character) or if
/// the glyph ID is outside the face's glyph range.
pub fn extract_glyph_edges(face: &Face, glyph_id: u16) -> Option<GlyphEdges> {
    struct OutlineCollector {
        contours: Vec<Contour>,
        current_contour: Vec<EdgeSegment>,
        first_point: Option<(f32, f32)>,
        prev_point: Option<(f32, f32)>,
        scale: f32,
    }

    impl ttf_parser::OutlineBuilder for OutlineCollector {
        fn move_to(&mut self, x: f32, y: f32) {
            self.close_current();
            self.first_point = Some((x as f32, y as f32));
            self.prev_point = Some((x as f32, y as f32));
        }

        fn line_to(&mut self, x: f32, y: f32) {
            if let Some(prev) = self.prev_point {
                self.current_contour.push(EdgeSegment::Line {
                    from: (prev.0 * self.scale, -prev.1 * self.scale),
                    to: (x as f32 * self.scale, -(y as f32) * self.scale),
                });
            }
            self.prev_point = Some((x as f32, y as f32));
        }

        fn quad_to(&mut self, x_ctrl: f32, y_ctrl: f32, x: f32, y: f32) {
            if let Some(prev) = self.prev_point {
                self.current_contour.push(EdgeSegment::Quadratic {
                    from: (prev.0 * self.scale, -prev.1 * self.scale),
                    ctrl: (x_ctrl as f32 * self.scale, -(y_ctrl as f32) * self.scale),
                    to: (x as f32 * self.scale, -(y as f32) * self.scale),
                });
            }
            self.prev_point = Some((x as f32, y as f32));
        }

        fn curve_to(&mut self, x_ctrl1: f32, y_ctrl1: f32, x_ctrl2: f32, y_ctrl2: f32, x: f32, y: f32) {
            if let Some(prev) = self.prev_point {
                self.current_contour.push(EdgeSegment::Cubic {
                    from: (prev.0 * self.scale, -prev.1 * self.scale),
                    ctrl1: (x_ctrl1 as f32 * self.scale, -(y_ctrl1 as f32) * self.scale),
                    ctrl2: (x_ctrl2 as f32 * self.scale, -(y_ctrl2 as f32) * self.scale),
                    to: (x as f32 * self.scale, -(y as f32) * self.scale),
                });
            }
            self.prev_point = Some((x as f32, y as f32));
        }

        fn close(&mut self) {
            self.close_current();
        }
    }

    impl OutlineCollector {
        fn close_current(&mut self) {
            if self.current_contour.is_empty() { return; }
            if let (Some(first), Some(prev)) = (self.first_point, self.prev_point) {
                if prev != first {
                    self.current_contour.push(EdgeSegment::Line {
                        from: (prev.0 * self.scale, -prev.1 * self.scale),
                        to: (first.0 * self.scale, -first.1 * self.scale),
                    });
                }
            }
            let edges = std::mem::take(&mut self.current_contour);
            self.contours.push(Contour { edges });
            self.first_point = None;
            self.prev_point = None;
        }
    }

    let units_per_em = face.units_per_em() as f32;
    let scale_v = 1.0 / units_per_em;

    let ttf_gid = ttf_parser::GlyphId(glyph_id);
    let advance = face.glyph_hor_advance(ttf_gid).map(|a| a as f32 * scale_v).unwrap_or(0.0);
    let bbox = face.glyph_bounding_box(ttf_gid).unwrap_or(ttf_parser::Rect { x_min: 0, y_min: 0, x_max: 0, y_max: 0 });
    let bearing_x = bbox.x_min as f32 * scale_v;
    let bearing_y = -(bbox.y_max as f32) * scale_v;
    let width = (bbox.x_max - bbox.x_min) as f32 * scale_v;
    let height = (bbox.y_max - bbox.y_min) as f32 * scale_v;

    let mut collector = OutlineCollector {
        contours: Vec::new(),
        current_contour: Vec::new(),
        first_point: None,
        prev_point: None,
        scale: scale_v,
    };

    face.outline_glyph(ttf_gid, &mut collector);

    collector.close_current();

    if collector.contours.is_empty() { return None; }

    color_edge(&mut collector.contours);

    Some(GlyphEdges {
        contours: collector.contours,
        advance,
        bearing_x,
        bearing_y,
        width,
        height,
    })
}

/// Rasterises a glyph's edge data into a three-channel MSDF texture.
///
/// The output is an `atlas_size × atlas_size` RGB8 buffer where each pixel's
/// R, G, and B channels store signed distances to the nearest edge, classified
/// by the edge's normal direction.
///
/// # Parameters
///
/// * `edges` — glyph outline data from [`extract_glyph_edges`].
/// * `atlas_size` — width and height of the output texture in pixels.
/// * `px_range` — the maximum signed-distance range in pixels; controls the
///   anti-aliasing falloff at the glyph boundary.
pub fn bake_msdf(
    edges: &GlyphEdges,
    atlas_size: u32,
    px_range: f32,
) -> Vec<u8> {
    let size = atlas_size as usize;
    let mut data = vec![0u8; size * size * 3];

    let margin = 1.0 / atlas_size as f32;
    let step = 1.0 / atlas_size as f32;

    let max_dim = edges.width.max(edges.height);
    if max_dim <= 0.0 { return data; }
    let draw_scale = 1.0 / max_dim * 0.9;
    let ox = (1.0 - edges.width * draw_scale) * 0.5;
    let oy = (1.0 - edges.height * draw_scale) * 0.5;

    for py in 0..size {
        for px in 0..size {
            let u = px as f32 * step + margin;
            let v = py as f32 * step + margin;
            let wx = (u - ox) / draw_scale;
            let wy = (v - oy) / draw_scale;

            let wp = (wx, wy);
            let inside = is_point_inside(wp, &edges.contours);

            let mut min_dist = [f32::MAX; 3];

            for contour in &edges.contours {
                for edge in &contour.edges {
                    let (dist, _) = dist_to_edge(wp, edge);
                    let normal = edge_midpoint_normal(edge);
                    let channel = classify_edge_normal(normal) as usize;
                    if dist < min_dist[channel] {
                        min_dist[channel] = dist;
                    }
                }
            }

            let mut any_negative = false;
            let mut any_set = false;
            for c in 0..3 {
                if min_dist[c] < f32::MAX {
                    any_set = true;
                    if min_dist[c] < px_range {
                        let signed_dist = if inside { px_range / 2.0 + min_dist[c] } else { px_range / 2.0 - min_dist[c] };
                        let norm = (signed_dist / px_range + 0.5).clamp(0.0, 1.0);
                        data[(py * size + px) * 3 + c] = (norm * 255.0) as u8;
                    } else {
                        data[(py * size + px) * 3 + c] = if inside { 255 } else { 0 };
                    }
                }
            }

            if !any_set {
                let val = if inside { 255u8 } else { 0u8 };
                for c in 0..3 {
                    data[(py * size + px) * 3 + c] = val;
                }
            }
        }
    }

    data
}

/// Converts a single-channel bitmap glyph into a three-channel SDF texture.
///
/// Each pixel is assigned the same signed distance value across all three
/// channels, producing a uniform single-channel SDF stored as RGB. Useful for
/// bitmap fonts where no vector outline is available.
///
/// # Parameters
///
/// * `bitmap` — source grayscale pixel data (`u8` per pixel, row-major).
/// * `bmp_width`, `bmp_height` — dimensions of the source bitmap.
/// * `atlas_size` — width and height of the output texture in pixels.
/// * `px_range` — the signed-distance range in pixels.
pub fn bake_sdf_from_bitmap(
    bitmap: &[u8],
    bmp_width: u32,
    bmp_height: u32,
    atlas_size: u32,
    px_range: f32,
) -> Vec<u8> {
    let size = atlas_size as usize;
    let mut data = vec![0u8; size * size * 3];

    let sx = bmp_width as f32 / atlas_size as f32;
    let sy = bmp_height as f32 / atlas_size as f32;

    for py in 0..size {
        for px in 0..size {
            let bx = (px as f32 * sx) as i32;
            let by = (py as f32 * sy) as i32;
            let bx = bx.clamp(0, bmp_width as i32 - 1) as usize;
            let by = by.clamp(0, bmp_height as i32 - 1) as usize;

            let src = bitmap[(by * bmp_width as usize + bx) as usize];
            let coverage = src as f32 / 255.0;

            let closest_trans = find_closest_transition(bitmap, bmp_width, bmp_height, bx, by, 8);
            let distance = if coverage > 0.5 {
                closest_trans
            } else {
                -closest_trans
            };
            let norm = (distance / px_range + 0.5).clamp(0.0, 1.0);
            let byte_val = (norm * 255.0) as u8;
            let idx = (py * size + px) * 3;
            data[idx] = byte_val;
            data[idx + 1] = byte_val;
            data[idx + 2] = byte_val;
        }
    }

    data
}

fn find_closest_transition(
    bitmap: &[u8],
    width: u32,
    height: u32,
    cx: usize,
    cy: usize,
    max_search: i32,
) -> f32 {
    let center_val = bitmap[cy * width as usize + cx];
    let mut min_dist = max_search as f32;

    for dy in -max_search..=max_search {
        for dx in -max_search..=max_search {
            let nx = cx as i32 + dx;
            let ny = cy as i32 + dy;
            if nx < 0 || nx >= width as i32 || ny < 0 || ny >= height as i32 {
                continue;
            }
            let nv = bitmap[ny as usize * width as usize + nx as usize];
            if (nv as i16 - center_val as i16).abs() > 64 {
                let d = ((dx * dx + dy * dy) as f32).sqrt();
                if d < min_dist {
                    min_dist = d;
                }
            }
        }
    }

    min_dist / width as f32
}

/// Traces a binary bitmap into a set of closed [`Contour`]s.
///
/// Performs a flood-fill walk on pixels with value > 128, then sorts the
/// perimeter pixels angularly around their centroid to produce a closed polygon
/// of [`EdgeSegment::Line`]s.
///
/// Contours with fewer than 3 pixels are discarded.
///
/// # Parameters
///
/// * `bitmap` — source grayscale pixel data (row-major, `u8` per pixel).
/// * `width`, `height` — dimensions of the bitmap.
pub fn trace_bitmap_to_contour(
    bitmap: &[u8],
    width: u32,
    height: u32,
) -> Vec<Contour> {
    let mut contours = Vec::new();
    let mut visited = vec![false; (width * height) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;
            if bitmap[idx] > 128 && !visited[idx] {
                if let Some(contour) = trace_contour(bitmap, width, height, x, y, &mut visited) {
                    contours.push(contour);
                }
            }
        }
    }

    contours
}

fn trace_contour(
    bitmap: &[u8],
    width: u32,
    height: u32,
    start_x: u32,
    start_y: u32,
    visited: &mut [bool],
) -> Option<Contour> {
    let mut stack = vec![(start_x, start_y)];
    let mut pixels = Vec::new();

    while let Some((x, y)) = stack.pop() {
        let idx = (y * width + x) as usize;
        if visited[idx] { continue; }
        visited[idx] = true;
        pixels.push((x, y));

        for (dx, dy) in &[(0i32, -1i32), (1, 0), (0, 1), (-1, 0)] {
            let nx = x as i32 + dx;
            let ny = y as i32 + dy;
            if nx >= 0 && nx < width as i32 && ny >= 0 && ny < height as i32 {
                let nidx = (ny as u32 * width + nx as u32) as usize;
                if bitmap[nidx] > 128 && !visited[nidx] {
                    stack.push((nx as u32, ny as u32));
                }
            }
        }
    }

    if pixels.len() < 3 { return None; }

    let mut cx_sum = 0i64;
    let mut cy_sum = 0i64;
    for &(x, y) in &pixels {
        cx_sum += x as i64;
        cy_sum += y as i64;
    }
    let cx = (cx_sum / pixels.len() as i64) as f32;
    let cy = (cy_sum / pixels.len() as i64) as f32;

    pixels.sort_by(|a, b| {
        let angle_a = ((a.1 as f32 - cy).atan2(a.0 as f32 - cx) * 1000.0) as i64;
        let angle_b = ((b.1 as f32 - cy).atan2(b.0 as f32 - cx) * 1000.0) as i64;
        angle_a.cmp(&angle_b)
    });

    let n = pixels.len();
    let mut edges = Vec::new();
    let nw = width as f32;
    let nh = height as f32;
    for i in 0..n {
        let curr = pixels[i];
        let next = pixels[(i + 1) % n];
        edges.push(EdgeSegment::Line {
            from: (curr.0 as f32 / nw, curr.1 as f32 / nh),
            to: (next.0 as f32 / nw, next.1 as f32 / nh),
        });
    }

    Some(Contour { edges })
}
