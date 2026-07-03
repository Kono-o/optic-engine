use crate::convert::{hsv_to_rgba, rgba_to_hsv};
use crate::{channel_lerp, HSV, RGBA, ToRgba};

#[derive(Copy, Clone, Debug)]
pub struct GradientStop {
    pub position: f32,
    pub color: RGBA,
}

#[derive(Copy, Clone, Debug)]
pub enum GradientInterp { Linear, Step, SmoothStep }

#[derive(Copy, Clone, Debug)]
pub enum GradientColorSpace { Rgb, Hsv }

#[derive(Copy, Clone, Debug)]
pub enum GradientWrap { Clamp, Repeat, PingPong }

pub struct Gradient {
    stops: Vec<GradientStop>,
    interp: GradientInterp,
    color_space: GradientColorSpace,
    wrap: GradientWrap,
}

impl Gradient {
    pub fn new() -> Self {
        Gradient {
            stops: Vec::new(),
            interp: GradientInterp::Linear,
            color_space: GradientColorSpace::Rgb,
            wrap: GradientWrap::Clamp,
        }
    }

    pub fn add_stop(&mut self, position: f32, color: impl ToRgba) -> &mut Self {
        let pos = position.clamp(0.0, 1.0);
        let stop = GradientStop { position: pos, color: color.to_rgba() };
        let idx = self.stops.binary_search_by(|s| s.position.partial_cmp(&pos).unwrap()).unwrap_or_else(|e| e);
        self.stops.insert(idx, stop);
        self
    }

    pub fn remove_stop(&mut self, index: usize) {
        if index < self.stops.len() {
            self.stops.remove(index);
        }
    }

    pub fn stops(&self) -> &[GradientStop] { &self.stops }

    pub fn clear(&mut self) { self.stops.clear(); }

    pub fn sample(&self, t: f32) -> RGBA {
        if self.stops.is_empty() {
            return RGBA(0.0, 0.0, 0.0, 0.0);
        }
        if self.stops.len() == 1 {
            return self.stops[0].color;
        }
        let t = match self.wrap {
            GradientWrap::Clamp => t.clamp(0.0, 1.0),
            GradientWrap::Repeat => t - t.floor(),
            GradientWrap::PingPong => {
                let r#mod = t - t.floor();
                if t.floor() as i32 % 2 == 0 { r#mod } else { 1.0 - r#mod }
            }
        };
        let t = t.clamp(0.0, 1.0);
        let i = match self.stops.binary_search_by(|s| s.position.partial_cmp(&t).unwrap()) {
            Ok(i) => i,
            Err(i) => {
                if i == 0 { return self.stops[0].color; }
                if i >= self.stops.len() { return self.stops[self.stops.len() - 1].color; }
                i - 1
            }
        };
        let (a, b) = if i + 1 < self.stops.len() {
            (self.stops[i], self.stops[i + 1])
        } else {
            return self.stops[i].color;
        };
        if a.position == b.position {
            return b.color;
        }
        let local_t = (t - a.position) / (b.position - a.position);
        let local_t = match self.interp {
            GradientInterp::Linear => local_t,
            GradientInterp::Step => 0.0,
            GradientInterp::SmoothStep => local_t * local_t * (3.0 - 2.0 * local_t),
        };
        match self.color_space {
            GradientColorSpace::Rgb => {
                channel_lerp(a.color, b.color, local_t)
            }
            GradientColorSpace::Hsv => {
                let hsv_a = rgba_to_hsv(a.color);
                let hsv_b = rgba_to_hsv(b.color);
                let h = hue_lerp(hsv_a.h, hsv_b.h, local_t);
                let s = hsv_a.s + (hsv_b.s - hsv_a.s) * local_t;
                let v = hsv_a.v + (hsv_b.v - hsv_a.v) * local_t;
                hsv_to_rgba(HSV { h, s, v }).with_alpha(
                    a.color.3 + (b.color.3 - a.color.3) * local_t,
                )
            }
        }
    }

    pub fn sample_n(&self, count: usize) -> Vec<RGBA> {
        if count == 0 { return Vec::new(); }
        let mut out = Vec::with_capacity(count);
        for i in 0..count {
            let t = i as f32 / (count - 1) as f32;
            out.push(self.sample(t));
        }
        out
    }

    pub fn set_interp(&mut self, mode: GradientInterp) -> &mut Self {
        self.interp = mode;
        self
    }

    pub fn set_color_space(&mut self, space: GradientColorSpace) -> &mut Self {
        self.color_space = space;
        self
    }

    pub fn set_wrap(&mut self, wrap: GradientWrap) -> &mut Self {
        self.wrap = wrap;
        self
    }

    pub fn reverse(&mut self) -> &mut Self {
        self.stops.reverse();
        for s in &mut self.stops {
            s.position = 1.0 - s.position;
        }
        self.stops.sort_by(|a, b| a.position.partial_cmp(&b.position).unwrap());
        self
    }

    pub fn from_colors(colors: &[impl ToRgba]) -> Self {
        let mut g = Gradient::new();
        let count = colors.len();
        if count == 0 { return g; }
        for (i, c) in colors.iter().enumerate() {
            let t = if count == 1 { 0.0 } else { i as f32 / (count - 1) as f32 };
            g.add_stop(t, *c);
        }
        g
    }

    pub fn two_color(a: impl ToRgba, b: impl ToRgba) -> Self {
        let mut g = Gradient::new();
        g.add_stop(0.0, a);
        g.add_stop(1.0, b);
        g
    }

    pub fn rainbow() -> Self {
        let mut g = Gradient::new();
        g.set_color_space(GradientColorSpace::Hsv);
        g.add_stop(0.0, HSV::new(0.0, 1.0, 1.0));
        g.add_stop(1.0, HSV::new(360.0, 1.0, 1.0));
        g
    }

    pub fn fire() -> Self {
        let mut g = Gradient::new();
        g.add_stop(0.0, crate::BLACK);
        g.add_stop(0.25, crate::RED);
        g.add_stop(0.5, crate::ORANGE);
        g.add_stop(0.75, crate::YELLOW);
        g.add_stop(1.0, crate::WHITE);
        g
    }

    pub fn grayscale() -> Self {
        let mut g = Gradient::new();
        g.add_stop(0.0, crate::BLACK);
        g.add_stop(1.0, crate::WHITE);
        g
    }
}

fn hue_lerp(a: f32, b: f32, t: f32) -> f32 {
    let mut diff = b - a;
    if diff > 180.0 { diff -= 360.0; }
    else if diff < -180.0 { diff += 360.0; }
    let result = a + diff * t;
    if result < 0.0 { result + 360.0 }
    else if result >= 360.0 { result - 360.0 }
    else { result }
}
