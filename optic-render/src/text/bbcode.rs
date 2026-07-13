use optic_core::{HSV, OpticError, OpticErrorKind, OpticResult, RGBA, BLACK, WHITE};

/// Style flags packed into instance `style_flags` (u32).
pub const FAUX_BOLD: u32 = 1 << 0;
pub const FAUX_ITALIC: u32 = 1 << 1;
pub const BORDER: u32 = 1 << 2;

#[derive(Clone, Debug, PartialEq)]
pub struct WaveEffect {
    pub amp: f32,
    pub freq: f32,
    pub speed: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ShakeEffect {
    pub amp: f32,
    pub speed: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RainbowEffect {
    pub speed: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PulseEffect {
    pub amp: f32,
    pub speed: f32,
}

#[derive(Clone, Debug, Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub color: Option<RGBA>,
    pub bgcolor: Option<RGBA>,
    pub strikethrough: bool,
    pub underline: bool,
    pub size: Option<f32>,
    pub border_color: Option<RGBA>,
    pub border_width: f32,
    pub kerning: f32,
    pub offset: [f32; 2],
    pub wave: Option<WaveEffect>,
    pub shake: Option<ShakeEffect>,
    pub rainbow: Option<RainbowEffect>,
    pub pulse: Option<PulseEffect>,
}

impl TextStyle {
    pub fn is_dynamic(&self) -> bool {
        self.wave.is_some()
            || self.shake.is_some()
            || self.rainbow.is_some()
            || self.pulse.is_some()
    }

    pub fn style_flags(&self, faux_bold: bool, faux_italic: bool) -> u32 {
        let mut flags = 0u32;
        if faux_bold || (self.bold && faux_bold) {
            flags |= FAUX_BOLD;
        }
        if faux_italic || (self.italic && faux_italic) {
            flags |= FAUX_ITALIC;
        }
        if self.border_color.is_some() {
            flags |= BORDER;
        }
        flags
    }
}

#[derive(Clone, Debug)]
pub struct StyledSpan {
    pub text: String,
    pub style: TextStyle,
}

#[derive(Clone, Debug)]
pub struct ParsedText {
    pub spans: Vec<StyledSpan>,
    pub is_dynamic: bool,
}

#[derive(Clone, Debug)]
enum OpenTag {
    Bold,
    Italic,
    Color(RGBA),
    BgColor(RGBA),
    Strike,
    Underline,
    Size(f32),
    Border { color: RGBA, width: f32 },
    Kerning(f32),
    Offset([f32; 2]),
    Wave(WaveEffect),
    Shake(ShakeEffect),
    Rainbow(RainbowEffect),
    Pulse(PulseEffect),
}

pub fn parse(raw: &str) -> OpticResult<ParsedText> {
    let mut spans = Vec::new();
    let mut current_text = String::new();
    let mut stack: Vec<TextStyle> = vec![TextStyle::default()];
    let mut is_dynamic = false;
    let mut i = 0;
    let bytes = raw.as_bytes();

    while i < bytes.len() {
        if bytes[i] == b'[' {
            if let Some((tag, close, new_i)) = parse_tag(&raw[i..])? {
                if !current_text.is_empty() {
                    spans.push(StyledSpan {
                        text: std::mem::take(&mut current_text),
                        style: stack.last().cloned().unwrap_or_default(),
                    });
                }
                if close {
                    pop_tag(&mut stack, &tag)?;
                } else {
                    if tag.is_dynamic() {
                        is_dynamic = true;
                    }
                    stack.push(apply_open_tag(stack.last().cloned().unwrap_or_default(), &tag));
                }
                i += new_i;
                continue;
            }
        }
        current_text.push(bytes[i] as char);
        i += 1;
    }

    if !current_text.is_empty() {
        spans.push(StyledSpan {
            text: current_text,
            style: stack.last().cloned().unwrap_or_default(),
        });
    }

    if stack.len() > 1 {
        return Err(OpticError::new(
            OpticErrorKind::Custom,
            "unclosed BBCode tag",
        ));
    }

    Ok(ParsedText { spans, is_dynamic })
}

pub fn detect_dynamic(raw: &str) -> bool {
    parse(raw)
        .map(|p| p.is_dynamic)
        .unwrap_or_else(|_| {
            raw.contains("[wave")
                || raw.contains("[shake")
                || raw.contains("[rainbow")
                || raw.contains("[pulse")
        })
}

fn parse_tag(src: &str) -> OpticResult<Option<(OpenTag, bool, usize)>> {
    let end = src.find(']').ok_or_else(|| {
        OpticError::new(OpticErrorKind::Custom, "unclosed BBCode tag")
    })?;
    let inner = &src[1..end];
    let close = inner.starts_with('/');
    let body = if close { &inner[1..] } else { inner };

    let tag = if close {
        parse_close_tag(body)?
    } else {
        parse_open_tag(body)?
    };

    Ok(Some((tag, close, end + 1)))
}

fn parse_close_tag(name: &str) -> OpticResult<OpenTag> {
    match name.trim().to_lowercase().as_str() {
        "b" => Ok(OpenTag::Bold),
        "i" => Ok(OpenTag::Italic),
        "color" => Ok(OpenTag::Color(WHITE)),
        "bgcolor" => Ok(OpenTag::BgColor(BLACK)),
        "s" => Ok(OpenTag::Strike),
        "u" => Ok(OpenTag::Underline),
        "size" => Ok(OpenTag::Size(1.0)),
        "border" => Ok(OpenTag::Border {
            color: BLACK,
            width: 1.0,
        }),
        "kerning" => Ok(OpenTag::Kerning(0.0)),
        "offset" => Ok(OpenTag::Offset([0.0, 0.0])),
        "wave" => Ok(OpenTag::Wave(WaveEffect {
            amp: 0.0,
            freq: 0.0,
            speed: 0.0,
        })),
        "shake" => Ok(OpenTag::Shake(ShakeEffect { amp: 0.0, speed: 0.0 })),
        "rainbow" => Ok(OpenTag::Rainbow(RainbowEffect { speed: 0.0 })),
        "pulse" => Ok(OpenTag::Pulse(PulseEffect { amp: 0.0, speed: 0.0 })),
        other => Err(OpticError::new(
            OpticErrorKind::Custom,
            &format!("unknown BBCode close tag: [{other}]"),
        )),
    }
}

fn parse_open_tag(body: &str) -> OpticResult<OpenTag> {
    let (name, args) = split_tag(body);
    match name.to_lowercase().as_str() {
        "b" => Ok(OpenTag::Bold),
        "i" => Ok(OpenTag::Italic),
        "color" => {
            let hex = args.ok_or_else(|| tag_err("color", "expected #rrggbbaa"))?;
            let c = RGBA::from_hex(hex).map_err(|e| tag_err("color", e))?;
            Ok(OpenTag::Color(c))
        }
        "bgcolor" => {
            let hex = args.ok_or_else(|| tag_err("bgcolor", "expected #rrggbbaa"))?;
            let c = RGBA::from_hex(hex).map_err(|e| tag_err("bgcolor", e))?;
            Ok(OpenTag::BgColor(c))
        }
        "s" => Ok(OpenTag::Strike),
        "u" => Ok(OpenTag::Underline),
        "size" => {
            let n = args
                .ok_or_else(|| tag_err("size", "expected size=N"))?
                .parse::<f32>()
                .map_err(|_| tag_err("size", "invalid number"))?;
            if n <= 0.0 {
                return Err(tag_err("size", "size must be positive"));
            }
            Ok(OpenTag::Size(n))
        }
        "border" => {
            let raw = args.ok_or_else(|| tag_err("border", "expected #rrggbbaa,width"))?;
            let (hex, width) = raw
                .split_once(',')
                .ok_or_else(|| tag_err("border", "expected #rrggbbaa,width"))?;
            let color = RGBA::from_hex(hex.trim()).map_err(|e| tag_err("border", e))?;
            let width = width
                .trim()
                .parse::<f32>()
                .map_err(|_| tag_err("border", "invalid width"))?;
            Ok(OpenTag::Border { color, width })
        }
        "kerning" => {
            let n = args
                .ok_or_else(|| tag_err("kerning", "expected kerning=N"))?
                .parse::<f32>()
                .map_err(|_| tag_err("kerning", "invalid number"))?;
            Ok(OpenTag::Kerning(n))
        }
        "offset" => {
            let raw = args.ok_or_else(|| tag_err("offset", "expected offset=x,y"))?;
            let (xs, ys) = raw
                .split_once(',')
                .ok_or_else(|| tag_err("offset", "expected offset=x,y"))?;
            let x = xs
                .trim()
                .parse::<f32>()
                .map_err(|_| tag_err("offset", "invalid x"))?;
            let y = ys
                .trim()
                .parse::<f32>()
                .map_err(|_| tag_err("offset", "invalid y"))?;
            Ok(OpenTag::Offset([x, y]))
        }
        "wave" => Ok(OpenTag::Wave(parse_wave(args)?)),
        "shake" => Ok(OpenTag::Shake(parse_shake(args)?)),
        "rainbow" => Ok(OpenTag::Rainbow(parse_rainbow(args)?)),
        "pulse" => Ok(OpenTag::Pulse(parse_pulse(args)?)),
        other => Err(OpticError::new(
            OpticErrorKind::Custom,
            &format!("unknown BBCode tag: [{other}]"),
        )),
    }
}

impl OpenTag {
    fn is_dynamic(&self) -> bool {
        matches!(
            self,
            OpenTag::Wave(_) | OpenTag::Shake(_) | OpenTag::Rainbow(_) | OpenTag::Pulse(_)
        )
    }
}

fn split_tag(body: &str) -> (&str, Option<&str>) {
    let body = body.trim();
    let name = body.split_whitespace().next().unwrap_or(body);
    if name.len() < body.len() {
        let args = body[name.len()..].trim();
        (name, if args.is_empty() { None } else { Some(args) })
    } else if let Some(idx) = body.find('=') {
        (&body[..idx], Some(body[idx + 1..].trim()))
    } else {
        (body, None)
    }
}

fn tag_err(tag: &str, msg: &str) -> OpticError {
    OpticError::new(OpticErrorKind::Custom, &format!("[{tag}]: {msg}"))
}

fn parse_kv(args: Option<&str>, key: &str) -> OpticResult<f32> {
    let raw = args.ok_or_else(|| tag_err(key, "missing arguments"))?;
    for part in raw.split(',') {
        let part = part.trim();
        if let Some((k, v)) = part.split_once('=') {
            if k.trim().eq_ignore_ascii_case(key) {
                return v.trim().parse::<f32>().map_err(|_| {
                    tag_err(key, &format!("invalid value for {key}"))
                });
            }
        }
    }
    Err(tag_err(key, &format!("missing {key}=")))
}

fn parse_wave(args: Option<&str>) -> OpticResult<WaveEffect> {
    Ok(WaveEffect {
        amp: parse_kv(args, "amp").unwrap_or(4.0),
        freq: parse_kv(args, "freq").unwrap_or(0.1),
        speed: parse_kv(args, "speed").unwrap_or(2.0),
    })
}

fn parse_shake(args: Option<&str>) -> OpticResult<ShakeEffect> {
    Ok(ShakeEffect {
        amp: parse_kv(args, "amp").unwrap_or(2.0),
        speed: parse_kv(args, "speed").unwrap_or(8.0),
    })
}

fn parse_rainbow(args: Option<&str>) -> OpticResult<RainbowEffect> {
    Ok(RainbowEffect {
        speed: parse_kv(args, "speed").unwrap_or(1.0),
    })
}

fn parse_pulse(args: Option<&str>) -> OpticResult<PulseEffect> {
    Ok(PulseEffect {
        amp: parse_kv(args, "amp").unwrap_or(0.2),
        speed: parse_kv(args, "speed").unwrap_or(3.0),
    })
}

fn apply_open_tag(mut base: TextStyle, tag: &OpenTag) -> TextStyle {
    match tag {
        OpenTag::Bold => base.bold = true,
        OpenTag::Italic => base.italic = true,
        OpenTag::Color(c) => base.color = Some(*c),
        OpenTag::BgColor(c) => base.bgcolor = Some(*c),
        OpenTag::Strike => base.strikethrough = true,
        OpenTag::Underline => base.underline = true,
        OpenTag::Size(s) => base.size = Some(*s),
        OpenTag::Border { color, width } => {
            base.border_color = Some(*color);
            base.border_width = *width;
        }
        OpenTag::Kerning(k) => base.kerning = *k,
        OpenTag::Offset(o) => base.offset = *o,
        OpenTag::Wave(w) => base.wave = Some(w.clone()),
        OpenTag::Shake(s) => base.shake = Some(s.clone()),
        OpenTag::Rainbow(r) => base.rainbow = Some(r.clone()),
        OpenTag::Pulse(p) => base.pulse = Some(p.clone()),
    }
    base
}

fn pop_tag(stack: &mut Vec<TextStyle>, tag: &OpenTag) -> OpticResult<()> {
    if stack.len() <= 1 {
        return Err(OpticError::new(
            OpticErrorKind::Custom,
            "BBCode close tag without open",
        ));
    }
    stack.pop();
    let _ = tag;
    Ok(())
}

/// Apply rainbow color for a glyph at the given index.
pub fn rainbow_color(effect: &RainbowEffect, time: f32, index: usize, alpha: f32) -> RGBA {
    let hue = (time * effect.speed * 360.0 + index as f32 * 30.0) % 360.0;
    HSV::new(hue, 1.0, 1.0).to_rgba_alpha(alpha)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_plain_text() {
        let p = parse("hello").unwrap();
        assert_eq!(p.spans.len(), 1);
        assert_eq!(p.spans[0].text, "hello");
        assert!(!p.is_dynamic);
    }

    #[test]
    fn parse_bold_color() {
        let p = parse("hi [b]bold[/b]").unwrap();
        assert_eq!(p.spans.len(), 2);
        assert!(!p.spans[0].style.bold);
        assert!(p.spans[1].style.bold);
    }

    #[test]
    fn parse_color_hex() {
        let p = parse("[color=#ff0000ff]red[/color]").unwrap();
        let c = p.spans[0].style.color.unwrap();
        assert!((c.0 - 1.0).abs() < 0.01);
        assert!((c.3 - 1.0).abs() < 0.01);
    }

    #[test]
    fn parse_error_bad_color() {
        assert!(parse("[color=notahex]x[/color]").is_err());
    }

    #[test]
    fn parse_error_unknown_tag() {
        assert!(parse("[foo]bar[/foo]").is_err());
    }

    #[test]
    fn parse_error_unclosed() {
        assert!(parse("[b]oops").is_err());
    }

    #[test]
    fn parse_error_bad_size() {
        assert!(parse("[size=0]x[/size]").is_err());
    }

    #[test]
    fn is_dynamic_detection() {
        assert!(detect_dynamic("plain [wave amp=2]x[/wave]"));
        assert!(!detect_dynamic("plain [b]x[/b]"));
    }

    #[test]
    fn rainbow_uses_hsv() {
        let c = rainbow_color(&RainbowEffect { speed: 1.0 }, 0.0, 0, 1.0);
        assert!((c.0 - 1.0).abs() < 0.01);
        let c2 = rainbow_color(&RainbowEffect { speed: 1.0 }, 0.0, 2, 0.5);
        assert!((c2.3 - 0.5).abs() < 0.01);
    }
}
