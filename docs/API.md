# Optic Engine — Public API Reference

> **Note:** Throughout the API, `OpticResult<T>` is `Result<T, OpticError>`.

> **Note:** Import everything via `use optic::*;` — the `optic` crate re-exports all sub-crates
> unmodified. Each crate below curates its own public surface; sub-module items use qualified
> paths (e.g. `optic::asset::ShaderFile`, `optic::cgmath::Vector3`).
>
> **Features:** `optic` has feature flags for each sub-crate.
> - Enable `online` for networking: `optic = { features = ["online"] }`.
> - Enable `sound` for audio: `optic = { features = ["sound"] }`.
> The `NetworkEvents` field on `Events` and the `audio` field on `Game` are always compiled
> (zero-cost empty vectors / always-present engine).

---

## Table of Contents

1. [Core Types (`optic_core`)](#1-core-types-optic_core)
   - [Color Types (re-exported)](#color-types-re-exported)
   - [Geometry Types](#geometry-types)
   - [Coordinate Types](#coordinate-types)
   - [Enums](#enums)
   - [Error Types](#error-types)
   - [Process Helpers](#process-helpers)
   - [Logging Macros](#logging-macros)
   - [Path & Format Constants](#path--format-constants)
   - [Re-exports](#re-exports)
2. [Color Crate (`optic_color`)](#2-color-crate-optic_color)
   - [RGBA](#rgba)
   - [RGB](#rgb)
   - [HSV](#hsv)
   - [HSL](#hsl)
   - [Gradient](#gradient)
   - [Traits: ToRgba / FromRgba / ColorInfo](#traits-torgba--fromrgba--colorinfo)
   - [ChannelArray & channel_lerp](#channelarray--channel_lerp)
   - [Named Color Constants (all RGBA)](#named-color-constants-all-rgba)
3. [Window (`optic_window`)](#3-window-optic_window)
   - [Window](#window)
   - [Events & Input](#events--input)
     - [Custom Events](#custom-events)
   - [ScreenInfo](#screeninfo)
   - [Re-exports](#re-exports-1)
4. [Renderer (`optic_render`)](#4-renderer-optic_render)
   - [GPU](#gpu)
   - [RenderContext](#rendercontext)
   - [GL Static Methods](#gl-static-methods)
   - [Camera](#camera)
   - [CamTransform](#camtransform)
   - [Mesh3D](#mesh3d)
   - [Mesh2D](#mesh2d)
   - [MeshHandle](#meshandle)
   - [Shader](#shader)
   - [Texture2D](#texture2d)
   - [StorageBuffer](#storagebuffer)
   - [Transform2D](#transform2d)
   - [Transform3D](#transform3d)
   - [Canvas](#canvas)
   - [CanvasDesc](#canvasdesc)
   - [RenderTarget](#rendertarget)
   - [Slot](#slot)
   - [Workers](#workers)
   - [Free Functions](#free-functions)
5. [Asset Types (`optic_render::asset`)](#5-asset-types-optic_renderasset)
   - [ShaderFile](#shaderfile)
   - [ShaderType](#shadertype)
   - [Mesh3DFile](#mesh3dfile)
   - [Mesh2DFile](#mesh2dfile)
   - [TextureFile](#texturefile)
   - [Attribute Types](#attribute-types)
   - [DataType Trait](#datatype-trait)
   - [Dirty](#dirty)
   - [Center](#center)
6. [Text Rendering (`optic_render::text`)](#6-text-rendering-optic_rendertext)
   - [FontFamilyFile](#fontfamilyfile)
   - [BakedFont](#bakedfont)
   - [GlyphMetrics](#glyphmetrics)
   - [BitmapFontLayout](#bitmapfontlayout)
   - [FontFamily (GPU)](#fontfamily-gpu)
   - [FontStyle](#fontstyle)
   - [BBCode Types](#bbcode-types)
   - [Layout Types](#layout-types)
   - [Text2D](#text2d)
   - [Text3D](#text3d)
7. [Game Loop (`optic_loop`)](#7-game-loop-optic_loop) -- three-phase execution
   - [Runtime Trait](#runtime-trait)
   - [FpsLimit](#fpslimit)
   - [Game](#game)
   - [Time](#time)
   - [Timer](#timer)
   - [Timers](#timers)
   - [FrameState](#framestate)
   - [WindowState](#windowstate)
   - [GameLoop](#gameloop)
   - [Standalone `run()`](#standalone-run)
8. [File Utilities (`optic_file`)](#8-file-utilities-optic_file)
9. [Networking (`optic_online`)](#9-networking-optic_online)
   - [NetworkConfig](#networkconfig)
   - [NetworkMode](#networkmode)
   - [PeerId](#peerid)
   - [NetworkEvents](#networkevents)
   - [NetworkHandle](#networkhandle)
10. [Sound (`optic_sound`)](#10-sound-optic_sound)
   - [AudioEngine](#audioengine)
   - [SoundFile](#soundfile)
   - [Sound2D](#sound2d)
   - [Sound3D](#sound3d)
11. [Removed: `optic_signals`](#11-removed-optic_signals)

---

## 1. Core Types (`optic_core`)

### Color Types (re-exported)

Color types (`RGBA`, `RGB`, `HSV`, `HSL`, `Gradient`, traits, channel_lerp, and all named
constants) are defined in the standalone [`optic_color`](#2-color-crate-optic_color) crate and
re-exported unchanged by `optic_core`. Import from either crate:

```rust
use optic_core::RGBA;       // works
use optic_color::RGBA;      // also works
use optic_color::Gradient;
```

### Geometry Types

```rust
pub trait Components<T, const N: usize>: Copy {
    fn to_array(self) -> [T; N];
    fn from_array(a: [T; N]) -> Self;
}
```

```rust
pub fn componentwise_min<T: PartialOrd + Copy, C: Components<T, N>, const N: usize>(a: C, b: C) -> C;
pub fn componentwise_max<T: PartialOrd + Copy, C: Components<T, N>, const N: usize>(a: C, b: C) -> C;
```

`Components` is implemented for `Size2D`, `Size3D`, and `CoordOffset` (via macro), providing array/tuple conversion and generic min/max operations.

#### Size2D

Derives:
- Copy
- Clone
- Debug
- PartialEq

Implements:
- Components
- From
- Add
- Sub
- Mul

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size2D {
    pub w: u32,
    pub h: u32,
}
impl Components<u32, 2> for Size2D {}
impl From<[u32; 2]> for Size2D {}
impl From<Size2D> for [u32; 2] {}
impl From<(u32, u32)> for Size2D {}
impl Add for Size2D { /* saturating */ }
impl Sub for Size2D { /* saturating */ }
impl Mul<f32> for Size2D {}
```

#### Size3D

Derives:
- Copy
- Clone
- Debug
- PartialEq

Implements:
- Components
- From
- Add
- Sub
- Mul

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Size3D {
    pub w: u32,
    pub h: u32,
    pub d: u32,
}
impl Components<u32, 3> for Size3D {}
impl From<[u32; 3]> for Size3D {}
impl From<Size3D> for [u32; 3] {}
impl From<(u32, u32, u32)> for Size3D {}
impl Add for Size3D { /* saturating */ }
impl Sub for Size3D { /* saturating */ }
impl Mul<f32> for Size3D {}
```

#### ClipDist

Derives:
- Clone
- Copy
- Debug

Implements:
- Default

```rust
#[derive(Clone, Copy, Debug)]
pub struct ClipDist {
    pub near: f32,
    pub far: f32,
}
impl Default for ClipDist { /* near: 0.01, far: 1000.0 */ }
```

#### CamProj

Derives:
- Clone
- Copy
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CamProj {
    Ortho,
    Persp,
}
```

#### Size2D

| Signature | Description |
|-----------|-------------|
| `pub fn zero() -> Size2D` | `Size2D { w: 0, h: 0 }` |
| `pub fn new(w: u32, h: u32) -> Self` | New from dimensions |
| `pub fn from(arr: [u32; 2]) -> Self` | From array (via `From` trait) |
| `pub fn from(tup: (u32, u32)) -> Self` | From tuple |
| `pub fn shave(&self, n: u32) -> Size2D` | Subtract `n` from each side (saturating) |
| `pub fn aspect_ratio(&self) -> f32` | `w as f32 / h as f32` (clamped to `0.001`) |
| `pub fn is_empty(&self) -> bool` | True if `w == 0` or `h == 0` |
| `pub fn area(&self) -> u64` | `w * h` |
| `pub fn min(&self, other) -> Size2D` | Component-wise minimum |
| `pub fn max(&self, other) -> Size2D` | Component-wise maximum |
| `pub fn fit_within(&self, max: Size2D) -> Size2D` | Scale down to fit within `max` while preserving aspect ratio |
| `pub fn scaled_to_width(&self, w: u32) -> Size2D` | Scale to target width while preserving aspect ratio |
| `pub fn scaled_to_height(&self, h: u32) -> Size2D` | Scale to target height while preserving aspect ratio |
| `pub fn to_size3d(&self, depth: u32) -> Size3D` | Promote to 3D with the given depth |
| `a + b` → `Size2D` | Saturating addition |
| `a - b` → `Size2D` | Saturating subtraction |
| `s * f32` → `Size2D` | Scalar multiplication (rounded, clamped to ≥ 0) |

#### Size3D

| Signature | Description |
|-----------|-------------|
| `pub fn zero() -> Size3D` | Zero-initialized |
| `pub fn new(w: u32, h: u32, d: u32) -> Self` | New from dimensions |
| `pub fn from(arr: [u32; 3]) -> Self` | From array |
| `pub fn from(tup: (u32, u32, u32)) -> Self` | From tuple |
| `pub fn shave(&self, n: u32) -> Size3D` | Subtract n from each side (saturating) |
| `pub fn is_empty(&self) -> bool` | True if w==0 \|\| h==0 \|\| d==0 |
| `pub fn volume(&self) -> u64` | `w * h * d` |
| `pub fn min(&self, other) -> Size3D` | Componentwise min |
| `pub fn max(&self, other) -> Size3D` | Componentwise max |
| `pub fn to_size2d(&self) -> Size2D` | Drop depth |
| `a + b` → `Size3D` | Saturating addition |
| `a - b` → `Size3D` | Saturating subtraction |
| `s * f32` → `Size3D` | Scalar multiplication (rounded, clamped ≥0) |

#### ClipDist

- `impl Default` → `ClipDist { near: 0.01, far: 1000.0 }`
- `pub fn new(near: f32, far: f32) -> ClipDist`

### Coordinate Types

#### Coord2D

Derives:
- Copy
- Clone
- Debug

Implements:
- Components
- From
- Sub
- Add

```rust
#[derive(Copy, Clone, Debug)]
pub struct Coord2D {
    pub x: f64,
    pub y: f64,
}
impl Components<f64, 2> for Coord2D {}
impl From<[f64; 2]> for Coord2D {}
impl From<Coord2D> for [f64; 2] {}
impl From<(f64, f64)> for Coord2D {}
impl Sub for Coord2D { type Output = CoordOffset; }
impl Add<CoordOffset> for Coord2D { type Output = Coord2D; }
impl Sub<CoordOffset> for Coord2D { type Output = Coord2D; }
```

#### CoordOffset

Derives:
- Copy
- Clone
- Debug

Implements:
- Components
- From
- Add
- Sub
- Mul
- Neg

```rust
#[derive(Copy, Clone, Debug)]
pub struct CoordOffset {
    pub x: f64,
    pub y: f64,
}
impl Components<f64, 2> for CoordOffset {}
impl From<[f64; 2]> for CoordOffset {}
impl From<CoordOffset> for [f64; 2] {}
impl From<(f64, f64)> for CoordOffset {}
impl Add for CoordOffset {}
impl Sub for CoordOffset {}
impl Mul<f64> for CoordOffset {}
impl Neg for CoordOffset {}
```

| Method | Coord2D (point) | CoordOffset (vector) |
|--------|-----------------|----------------------|
| `zero()` | (0,0) | (0,0) |
| `new(x, y)` | New coord | New coord |
| `from_tup((x, y))` | From tuple | From tuple |
| `is_inside(size: Size2D)` | Checks bounds | — |
| `is_zero()` | — | Returns true if both zero |
| `distance_to(other)` | Distance to another point | — |
| `midpoint(other)` | Midpoint to another point | — |
| `lerp(other, t)` | Point interpolation (0..1) | Vector interpolation (0..1) |
| `min(other)` | Componentwise min | Componentwise min |
| `max(other)` | Componentwise max | Componentwise max |
| — `Sub` → | `Coord2D - Coord2D = CoordOffset` | `CoordOffset - CoordOffset = CoordOffset` |
| — `Add` → | `Coord2D + CoordOffset = Coord2D` | `CoordOffset + CoordOffset = CoordOffset` |
| — `Sub` → | `Coord2D - CoordOffset = Coord2D` | — |
| `length()` | — | Euclidean norm |
| `length_squared()` | — | Squared norm |
| `normalize()` | — | Unit vector (zero→zero, no NaN) |
| `dot(other)` | — | Dot product |
| — `Mul<f64>` | — | Scalar multiplication |
| — `Neg` | — | Negation |

### Enums

#### PolygonMode

Derives:
- Copy
- Clone
- Debug
- PartialEq

Implements:
- None

#### CullFace

Derives:
- Copy
- Clone
- Debug
- PartialEq

Implements:
- None

#### DrawMode

Derives:
- Copy
- Clone
- Debug
- Default
- PartialEq

Implements:
- None

#### ImgFormat

Derives:
- Copy
- Clone
- Debug
- PartialEq

Implements:
- None

#### ImgFilter

Derives:
- Debug
- Clone
- Copy
- PartialEq

Implements:
- None

#### ImgWrap

Derives:
- Debug
- Clone
- Copy
- PartialEq

Implements:
- None

#### ATTRType

Derives:
- Clone
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum PolygonMode { Points, WireFrame, Filled }

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CullFace { Clock, AntiClock }

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum DrawMode { Points, Lines, #[default] Triangles, Strip }

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ImgFormat { R(u8), RG(u8), RGB(u8), RGBA(u8) }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgFilter { Closest, Linear }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImgWrap { Repeat, Extend, Clip }

#[derive(Clone, Debug, PartialEq)]
pub enum ATTRType { U8, I8, U16, I16, U32, I32, F32, F64 }
```

#### ImgFormat methods

| Signature | Description |
|-----------|-------------|
| `pub fn channels(&self) -> u8` | Number of color channels (1-4) |
| `pub fn bit_depth(&self) -> u8` | Bits per channel |
| `pub fn pixel_size(&self) -> u8` | Total bytes per pixel (channels × bit_depth/8) |
| `pub fn from(channels: u8, bit_depth: u8) -> ImgFormat` | Construct from channel count and bit depth |

### Error Types

#### OpticErrorKind

Derives:
- Debug
- Clone
- PartialEq

Implements:
- None

#### OpticError

Derives:
- Debug
- Clone

Implements:
- Display

```rust
#[derive(Debug, Clone, PartialEq)]
pub enum OpticErrorKind {
    Init, OpenGL, Shader, Asset, File, Framebuffer, Custom,
}

#[derive(Debug, Clone)]
pub struct OpticError {
    pub kind: OpticErrorKind,
    pub msg: String,
}
impl fmt::Display for OpticError {}

pub type OpticResult<T> = Result<T, OpticError>;
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(kind: OpticErrorKind, msg: &str) -> Self` | Construct an error |
| `pub fn custom(msg: &str) -> Self` | Shorthand for `OpticErrorKind::Custom` |
| `pub fn shader(msg: &str) -> Self` | Shorthand for `OpticErrorKind::Shader` |
| `pub fn asset(msg: &str) -> Self` | Shorthand for `OpticErrorKind::Asset` |
| `pub fn file(msg: &str) -> Self` | Shorthand for `OpticErrorKind::File` |
| `pub fn init(msg: &str) -> Self` | Shorthand for `OpticErrorKind::Init` |
| `impl fmt::Display for OpticError` | Formatted as `"{kind}: {msg}"` |

### Error Handling Pattern

All fallible functions return `OpticResult<T>`. Errors propagate upward via `?` and converge at
the application's entry point. `Game::run()` and the standalone `run()` handle errors internally —
they log the error and exit cleanly, so callers just need:

```rust
fn main() {
    Game::run(MyRuntime { ... });
}
```

Or with the standalone loop:

```rust
fn main() {
    optic_loop::run("My App", Size2D::from(800, 600), |frame| {
        // per-frame logic
    });
}
```

### Process Helpers

```rust
pub const SUCCESS: i32 = 0;
pub const ERROR: i32 = 1;

pub fn end(code: i32) -> !            // Print code, exit process
pub fn end_success() -> !             // end(SUCCESS)
pub fn end_error() -> !               // end(ERROR)
```

### Logging Macros

```rust
log_color!(color: ANSI, "fmt", ...)    // Colored output
log_event!("fmt", ...)                  // "[EVENT] ..."
log_info!("fmt", ...)                   // "[INFO] ..."
log_warn!("fmt", ...)                   // "[WARN] ..."
log_error!("fmt", ...)                  // "[ERROR] ..."
log_fatal!("fmt", ...)                  // "[FATAL] ..." then end_error()
```

### ANSI Terminal Colors

#### ANSI

Derives:
- None

Implements:
- None

```rust
pub struct ANSI {
    pub prefix: &'static str,
    pub suffix: &'static str,
}
// No derives — used only as const values.
```

Used with the `log_color!` macro. All constants are `pub const NAME: ANSI`.

**Foreground:** `RED`, `GREEN`, `YELLOW`, `BLUE`, `MAGENTA`, `CYAN`
**Bold foreground:** `BOLD_RED`, `BOLD_GREEN`, `BOLD_YELLOW`, `BOLD_BLUE`, `BOLD_MAGENTA`, `BOLD_CYAN`
**Dark foreground:** `DARK_RED`, `DARK_GREEN`, `DARK_YELLOW`, `DARK_BLUE`, `DARK_MAGENTA`, `DARK_CYAN`
**Bold dark foreground:** `BOLD_DARK_RED`, `BOLD_DARK_GREEN`, `BOLD_DARK_YELLOW`, `BOLD_DARK_BLUE`, `BOLD_DARK_MAGENTA`, `BOLD_DARK_CYAN`

**Background:** `BG_RED`, `BG_GREEN`, `BG_YELLOW`, `BG_BLUE`, `BG_MAGENTA`, `BG_CYAN`
**Bold background:** `BOLD_BG_RED`, `BOLD_BG_GREEN`, `BOLD_BG_YELLOW`, `BOLD_BG_BLUE`, `BOLD_BG_MAGENTA`, `BOLD_BG_CYAN`
**Dark background:** `BG_DARK_RED`, `BG_DARK_GREEN`, `BG_DARK_YELLOW`, `BG_DARK_BLUE`, `BG_DARK_MAGENTA`, `BG_DARK_CYAN`
**Bold dark background:** `BOLD_BG_DARK_RED`, `BOLD_BG_DARK_GREEN`, `BOLD_BG_DARK_YELLOW`, `BOLD_BG_DARK_BLUE`, `BOLD_BG_DARK_MAGENTA`, `BOLD_BG_DARK_CYAN`

### Path & Format Constants

```rust
// Asset directories
pub const ASSET: &str        = "opt/";
pub const TEMP: &str         = "opt/temp/";
pub const SHDR_ASSET: &str   = "opt/shdr/";
pub const MESH_ASSET: &str   = "opt/mesh/";
pub const TXTR_ASSET: &str   = "opt/txtr/";

// Source file extensions
pub const VERT: &str = "vert";
pub const FRAG: &str = "frag";
pub const GLSL: &str = "glsl";
pub const OBJ: &str  = "obj";
pub const PNG: &str  = "png";
pub const WAV: &str  = "wav";

// Optic cache extensions
pub const OSHDR: &str  = "oshdr";
pub const OMESH: &str  = "omesh";
pub const OTXTR: &str  = "otxtr";
pub const OFONT: &str  = "ofont";
pub const OMUSIC: &str = "omusic";

// Binary cache magic & version
pub const OPTIC_MAGIC: [u8; 8]       = *b"/0PTIC_x";   // Never changes
pub const OPTIC_CACHE_VERSION: u16   = 1;                // Bump when format changes

// Shader sub-types
pub const SHADER_PIPELINE: u8      = 0;
pub const SHADER_COMPUTE: u8       = 1;

// Mesh attribute flags
pub const MESH_FLAG_HAS_NORMALS: u8 = 0b0001;
pub const MESH_FLAG_HAS_UVS: u8     = 0b0010;
```

### Re-exports

`optic_core` re-exports the entire `cgmath` crate.

---

## 2. Color Crate (`optic_color`)

Standalone zero-dependency color library. Types are re-exported by `optic_core` — you can use
`optic_core::RGBA` or `optic_color::RGBA` interchangeably.

```rust
use optic_color::prelude::*;     // RGBA, RGB, HSV, HSL, Gradient, ToRgba, FromRgba, ColorInfo
// or import individually:
use optic_color::{RGBA, RGB, HSV, HSL, Gradient, ToRgba, FromRgba, ColorInfo, channel_lerp};
```

### RGBA

Derives:
- Copy
- Clone
- Debug

Implements:
- ToRgba
- FromRgba
- ColorInfo
- ChannelArray
- Add
- Sub
- Mul
- Div
- From

```rust
#[derive(Copy, Clone, Debug)]
pub struct RGBA(pub f32, pub f32, pub f32, pub f32);
impl ToRgba for RGBA {}
impl FromRgba for RGBA {}
impl ColorInfo for RGBA {}
impl ChannelArray<f32, 4> for RGBA {}
impl Add, Sub, Mul, Div for RGBA {}
impl Mul<f32>, Div<f32> for RGBA {}
impl From<[f32; 4]> for RGBA {}
impl From<RGB> for RGBA {}
impl From<HSV> for RGBA {}
impl From<HSL> for RGBA {}
```

**Fields:** `r` (red), `g` (green), `b` (blue), `a` (alpha) — all `0..1`.

| Method | Description |
|--------|-------------|
| `new(r, g, b, a)` | Constructor |
| `grey(lum)` | Grey with alpha 1.0 |
| `from_rgb(rgb, alpha)` | RGB + alpha → RGBA |
| `to_rgb()` | Drop alpha |
| `with_alpha(a)` | Replace alpha |
| `from_hex("#RRGGBB" or "#RGB" or "#RRGGBBAA" or "#RGBA")` | Parse hex string |
| `from_hex_u32(0xRRGGBBAA)` | Parse hex u32 |
| `to_hex_u32()` | Encode as `0xRRGGBBAA` |
| `from_bytes(r, g, b, a)` | `u8` 0..255 inputs |
| `to_bytes()` | `(u8, u8, u8, u8)` |
| `lighten(amount)` | Increase value (HSV) by amount |
| `darken(amount)` | Decrease value (HSV) by amount |
| `saturate(amount)` | Increase saturation (HSV) by amount |
| `desaturate(amount)` | Decrease saturation (HSV) by amount |
| `invert()` | `1 - r, 1 - g, 1 - b` (alpha unchanged) |
| `to_linear()` | sRGB EOTF (piecewise) → linear RGB |
| `to_srgb()` | Linear RGB → sRGB OETF (piecewise) |

Implements `ToRgba`, `FromRgba`, `ColorInfo`, `channel_lerp`, `Add`, `Sub`, `Mul`, `Div` (`f32`),
`From<RGB>`, `From<HSV>`, `From<HSL>`, `From<[f32; 4]>`.

### RGB

Derives:
- Copy
- Clone
- Debug

Implements:
- ToRgba
- FromRgba
- ColorInfo
- ChannelArray
- Add
- Sub
- Mul
- Div
- From

```rust
#[derive(Copy, Clone, Debug)]
pub struct RGB(pub f32, pub f32, pub f32);
impl ToRgba for RGB {}
impl FromRgba for RGB {}
impl ColorInfo for RGB {}
impl ChannelArray<f32, 3> for RGB {}
impl Add, Sub, Mul, Div for RGB {}
impl Mul<f32>, Div<f32> for RGB {}
impl From<[f32; 3]> for RGB {}
```

| Signature | Description |
|-----------|-------------|
| `RGB(r, g, b)` | Construct via tuple struct (pub fields) |
| `pub fn grey(lum) -> Self` | Grey |
| `pub fn from_rgba(rgba) -> Self` | Drop alpha |
| `pub fn to_rgba(&self, alpha: f32) -> RGBA` | RGBA with given alpha |

Implements `ToRgba`, `FromRgba`, `ColorInfo`, `channel_lerp`, `Add`, `Sub`, `Mul`, `Div` (`f32`),
`From<[f32; 3]>`.

### HSV

Derives:
- Copy
- Clone
- Debug

Implements:
- ToRgba
- FromRgba
- ColorInfo
- From

```rust
#[derive(Copy, Clone, Debug)]
pub struct HSV {
    pub h: f32,
    pub s: f32,
    pub v: f32,
}
impl ToRgba for HSV {}
```

- `h`: 0..360 (wraps), `s`/`v`: 0..1
- `new(h, s, v)` → constructor
- `to_rgba_alpha(alpha)` → RGBA with given alpha

HSV intentionally does **not** implement `ChannelArray`, `Add`, `Sub`, `Mul`, or `lerp` — hue
wraparound makes componentwise arithmetic produce wrong colors. Convert to RGBA (`.into()`) for
arithmetic, or use `Gradient` with `GradientColorSpace::Hsv` for hue-aware interpolation.

Implements `ToRgba`, `FromRgba`, `ColorInfo`, `From<RGBA>`.

### HSL

Derives:
- Copy
- Clone
- Debug

Implements:
- ToRgba
- FromRgba
- ColorInfo
- From

```rust
#[derive(Copy, Clone, Debug)]
pub struct HSL {
    pub h: f32,
    pub s: f32,
    pub l: f32,
}
impl ToRgba for HSL {}
```

- `h`: 0..360 (wraps), `s`/`l`: 0..1
- `new(h, s, l)` → constructor
- `to_rgba_alpha(alpha)` → RGBA with given alpha

Same arithmetic caveat as HSV. Implements `ToRgba`, `FromRgba`, `ColorInfo`, `From<RGBA>`.

### Gradient

#### GradientStop

Derives:
- Copy
- Clone
- Debug

Implements:
- None

#### GradientInterp

Derives:
- Copy
- Clone
- Debug

Implements:
- None

#### GradientColorSpace

Derives:
- Copy
- Clone
- Debug

Implements:
- None

#### GradientWrap

Derives:
- Copy
- Clone
- Debug

Implements:
- None

```rust
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

```

#### Gradient

Derives:
- None

Implements:
- None

```rust
// No derives — contains gradient configuration.
pub struct Gradient {
    stops: Vec<GradientStop>,
    interp: GradientInterp,
    color_space: GradientColorSpace,
    wrap: GradientWrap,
}
```

| Method | Description |
|--------|-------------|
| `new()` | Empty gradient (linear, RGB, clamp) |
| `add_stop(position, color: impl ToRgba)` | Insert stop (sorted) |
| `remove_stop(index)` | Remove stop by index |
| `stops()` | `&[GradientStop]` |
| `len()` | Number of stops |
| `is_empty()` | True if no stops |
| `first()` | `Option<&GradientStop>` — first stop |
| `last()` | `Option<&GradientStop>` — last stop |
| `clear()` | Remove all stops |
| `sample(t)` | Sample at `t` (0..1) with wrap/interp/color-space |
| `sample_n(count)` | `Vec<RGBA>` of count evenly-spaced samples |
| `set_interp(mode)` | Set interpolation mode |
| `set_color_space(space)` | Set RGB or HSV interpolation |
| `set_wrap(wrap)` | Set wrapping mode |
| `reverse()` | Reverse stop order |
| `from_colors(&[impl ToRgba])` | Construct from evenly-spaced colors |
| `two_color(a, b)` | Two-stop gradient |
| `rainbow()` | HSV rainbow (red→red via 360°) |
| `fire()` | Black→red→orange→yellow→white |
| `grayscale()` | Black→white |

### Traits: ToRgba / FromRgba / ColorInfo

```rust
pub trait ToRgba: Copy {
    fn to_rgba(self) -> RGBA;
}

pub trait FromRgba: Sized {
    fn from_rgba(rgba: RGBA) -> Self;
}

pub trait ColorInfo: ToRgba {
    fn luminance(self) -> f32 { /* ... */ }
    fn is_light(self, threshold: f32) -> bool { /* ... */ }
    fn contrast_ratio(self, other: impl ToRgba) -> f32 { /* ... */ }
    fn to_hex(self) -> String { /* ... */ }
    fn to_bytes(self) -> (u8, u8, u8, u8) { /* ... */ }
}
```

- `ToRgba` is implemented for `RGBA`, `RGB`, `HSV`, `HSL`.
- `FromRgba` is implemented for `RGBA`, `RGB`, `HSV`, `HSL`.
- `ColorInfo` has a **blanket impl**: `impl<T: ToRgba> ColorInfo for T {}` — any type that
  implements `ToRgba` automatically gets `luminance()`, `is_light()`, `contrast_ratio()`,
  `to_hex()`, `to_bytes()`.

**Conversion rules:**
- `From<RGB> for RGBA` → adds alpha 1.0
- `From<HSV> for RGBA` → hue/sat/value → RGBA
- `From<HSL> for RGBA` → hue/sat/lightness → RGBA
- `From<RGBA> for RGB` → drops alpha
- `From<RGBA> for HSV` → RGBA → HSV
- `From<RGBA> for HSL` → RGBA → HSL

### ChannelArray & channel_lerp

```rust
pub trait ChannelArray<const N: usize>: Copy {
    fn to_array(self) -> [f32; N];
    fn from_array(arr: [f32; N]) -> Self;
}

pub fn channel_lerp<T: ChannelArray<N>, const N: usize>(a: T, b: T, t: f32) -> T;
pub fn channel_add<T: ChannelArray<N>, const N: usize>(a: T, b: T) -> T;
pub fn channel_sub<T: ChannelArray<N>, const N: usize>(a: T, b: T) -> T;
pub fn channel_mul<T: ChannelArray<N>, const N: usize>(a: T, b: T) -> T;
pub fn channel_mul_scalar<T: ChannelArray<N>, const N: usize>(a: T, s: f32) -> T;
pub fn channel_div_scalar<T: ChannelArray<N>, const N: usize>(a: T, s: f32) -> T;
```

- `ChannelArray<4>` implemented for `RGBA`, `ChannelArray<3>` for `RGB`.
- `channel_lerp` is the building block for RGB gradient interpolation and general use.

Channel operators (`Add`, `Sub`, `Mul`, `Div` with `f32`) are generated by the
`impl_channel_ops!` macro for `RGBA` and `RGB`.

### Named Color Constants (all `RGBA`)

`RED`, `CRIMSON`, `PINK`, `BLUSH`, `CORAL`, `ORANGE`, `AMBER`, `GOLD`,
`YELLOW`, `LIME`, `SPRING`, `SEA`, `FOREST`, `GREEN`, `TEAL`, `AQUA`,
`SKY`, `CYAN`, `BLUE`, `MIDNIGHT`, `INDIGO`, `PURPLE`, `PLUM`, `DUSK`,
`MAGENTA`, `FERN`, `SALMON`, `BROWN`, `GRAY`, `SILVER`, `WHITE`, `BLACK`,
`OBSIDIAN`, `MAROON`, `BURGUNDY`, `SCARLET`, `PEACH`, `APRICOT`, `TANGERINE`,
`MANGO`, `MUSTARD`, `OLIVE`, `CELADON`, `MINT`, `TURQUOISE`, `COBALT`,
`NAVY`, `LAPIS`, `LAVENDER`, `VIOLET`, `WISTERIA`, `MULBERRY`, `ROSEWOOD`,
`MAHOGANY`, `KHAKI`, `BEIGE`, `SAND`, `COPPER`, `BRONZE`, `SLATE`,
`CHARCOAL`, `IVORY`, `ALABASTER`, `SNOW`

---

## 3. Window (`optic_window`)

### Window

Derives:
- Debug

Implements:
- None

```rust
#[derive(Debug)]
pub struct Window {
    inner: Option<std::sync::Arc<WinitWindow>>,
    prev_cursor_pos: Coord2D,
    cursor_delta: CoordOffset,
    prev_position: Coord2D,
    position_delta: CoordOffset,
    prev_size: Size2D,
    cursor_inside: bool,
    tracking_started: bool,
    cursor_pos: Coord2D,
    cursor_visible: bool,
    cursor_grabbed: bool,
    cursor_confined: bool,
    cursor_loopback: bool,
    min_size: Option<Size2D>,
    max_size: Option<Size2D>,
}
```

#### Construction & Lifecycle

| Signature | Description |
|-----------|-------------|
| `pub fn new(el: &EventLoop<()>, title: &str, size: Size2D) -> Self` | Create a new window (starts hidden, opaque) |
| `pub fn new_transparent(el: &EventLoop<()>, title: &str, size: Size2D) -> Self` | Create a transparent window (X11 depth-32 ARGB visual) |
| `pub fn close(&mut self)` | Close the window |
| `pub fn is_closed(&self) -> bool` | Returns true if window handle was dropped |
| `pub fn is_running(&self) -> bool` | Returns `!is_closed()` |
| `pub fn id(&self) -> Option<WindowId>` | Winit window ID |
| `pub fn request_redraw(&self)` | Request a redraw on the next frame |

#### Raw Handles

| Signature | Description |
|-----------|-------------|
| `pub fn raw_handle(&self) -> Option<RawWindowHandle>` | Platform-specific window handle |
| `pub fn raw_display_handle(&self) -> Option<RawDisplayHandle>` | Platform-specific display handle |

#### Sizing

| Signature | Description |
|-----------|-------------|
| `pub fn size(&self) -> Size2D` | Current inner size (live winit query) |
| `pub fn set_size(&self, size: Size2D)` | Set inner size |
| `pub fn prev_size(&self) -> Size2D` | Last cached size |
| `pub fn min_size(&self) -> Option<Size2D>` | Minimum window size |
| `pub fn set_min_size(&mut self, size: Option<Size2D>)` | Set minimum size |
| `pub fn max_size(&self) -> Option<Size2D>` | Maximum window size |
| `pub fn set_max_size(&mut self, size: Option<Size2D>)` | Set maximum size |
| `pub fn resizable(&self) -> bool` | Whether the window is resizable |
| `pub fn set_resizable(&self, enable: bool)` | Toggle resizability |

#### Position

| Signature | Description |
|-----------|-------------|
| `pub fn position(&self) -> Coord2D` | Outer position on desktop (live winit query) |
| `pub fn set_position(&self, pos: Coord2D)` | Set outer position |
| `pub fn center_on_screen(&self)` | Center window on the current monitor |
| `pub fn prev_position(&self) -> Coord2D` | Last cached position |
| `pub fn position_delta(&mut self) -> CoordOffset` | Movement since last polled (resets to zero on read) |

#### Title

| Signature | Description |
|-----------|-------------|
| `pub fn title(&self) -> String` | Current window title |
| `pub fn set_title(&self, title: &str)` | Set window title |

#### Fullscreen

| Signature | Description |
|-----------|-------------|
| `pub fn is_fullscreen(&self) -> bool` | Is fullscreen? |
| `pub fn set_fullscreen(&self, enable: bool)` | Toggle fullscreen on/off |
| `pub fn toggle_fullscreen(&self)` | Toggle fullscreen state |

#### State

| Signature | Description |
|-----------|-------------|
| `pub fn is_visible(&self) -> bool` | Is window visible? |
| `pub fn set_visible(&self, visible: bool)` | Show/hide window |
| `pub fn toggle_visible(&self)` | Toggle window visibility |
| `pub fn is_minimized(&self) -> bool` | Is minimized? |
| `pub fn minimize(&self)` | Minimize window |
| `pub fn restore(&self)` | Restore from minimized |
| `pub fn toggle_minimized(&self)` | Toggle minimized state |
| `pub fn is_maximized(&self) -> bool` | Is maximized? |
| `pub fn maximize(&self)` | Maximize window |
| `pub fn unmaximize(&self)` | Restore from maximized |
| `pub fn toggle_maximized(&self)` | Toggle maximized state |
| `pub fn has_focus(&self) -> bool` | Does the window have keyboard focus? |
| `pub fn focus(&self)` | Request focus |

#### Cursor

| Signature | Description |
|-----------|-------------|
| `pub fn cursor_pos(&self) -> Coord2D` | Last-known cursor position (cached from events/setters) |
| `pub fn set_cursor_pos(&mut self, pos: Coord2D)` | Set cursor position (also updates cache) |
| `pub fn cursor_delta(&self) -> CoordOffset` | Difference from previous frame's cursor position |
| `pub fn cursor_pos_normalized(&self) -> Coord2D` | Cursor pos normalized to [0,1] by window size |
| `pub fn is_cursor_inside(&self) -> bool` | Is cursor inside the window? |
| `pub fn is_cursor_visible(&self) -> bool` | Is cursor visible? |
| `pub fn set_cursor_visible(&mut self, visible: bool)` | Show/hide cursor |
| `pub fn toggle_cursor_visible(&mut self)` | Toggle cursor visibility |
| `pub fn is_cursor_grabbed(&self) -> bool` | Is cursor grabbed? |
| `pub fn set_cursor_grab(&mut self, grab: bool) -> Result<(), ()>` | Set cursor grab mode |
| `pub fn toggle_cursor_grab(&mut self)` | Toggle cursor grab |
| `pub fn is_cursor_confined(&self) -> bool` | Is cursor confined to window? |
| `pub fn set_cursor_confine(&mut self, confine: bool) -> Result<(), ()>` | Confine/free cursor |
| `pub fn toggle_cursor_confine(&mut self)` | Toggle confine |
| `pub fn is_cursor_loopback(&self) -> bool` | Is cursor loopback enabled? |
| `pub fn set_cursor_loopback(&mut self, loopback: bool)` | Enable/disable edge-wrapping loopback |
| `pub fn set_cursor(&self, cursor: CursorIcon)` | Set the cursor icon |

#### Screen Info

| Signature | Description |
|-----------|-------------|
| `pub fn screen_info(&self) -> Option<ScreenInfo>` | Information about the current monitor |
| `pub fn dpi_scale(&self) -> f64` | The DPI scale factor for this window |

#### Frame Update

| Signature | Description |
|-----------|-------------|
| `pub fn update_frame(&mut self)` | Call once per frame to compute cursor delta and handle loopback teleport |

#### Internal (doc-hidden, used by the event loop)

| Signature | Description |
|-----------|-------------|
| `pub fn notify_cursor_moved(&mut self, pos: Coord2D)` | Update cached cursor position from event |
| `pub fn notify_cursor_inside(&mut self, inside: bool)` | Update cursor-enter/leave state |

### Events & Input

#### EventPayload

Derives:
- Debug
- Clone

Implements:
- None

```rust
#[derive(Debug, Clone)]
pub enum EventPayload {
    None,
    Bytes(Vec<u8>),
}
```

A value carried by a named custom event — either empty (`None`) or typed bytes (`Bytes`).

#### Events

Derives:
- Debug

Implements:
- None

```rust
#[derive(Debug)]
pub struct Events {
    pub keys: [ButtonState; 256],
    pub mouse_buttons: [ButtonState; 8],
    pub mouse_scroll_line: Option<(f32, f32)>,
    pub mouse_scroll_pixel: Option<(f64, f64)>,
    pub modifiers: ModifiersState,
    pub gamepad_connected: [bool; MAX_GAMEPADS],
    pub gamepad_buttons: [[ButtonState; 20]; MAX_GAMEPADS],
    pub gamepad_axes: [[f32; 6]; MAX_GAMEPADS],
    pub resize_event: Option<Size2D>,
    pub close_requested: bool,
    pub focused: bool,
    pub frame: u64,
    pub network: NetworkEvents,
    // custom_events: HashMap<String, Vec<EventPayload>>,  // (private)
}
```

#### ButtonState

Derives:
- Copy
- Clone

Implements:
- None

```rust
#[derive(Copy, Clone)]
pub struct ButtonState {
    pub held: bool,
    pub press_frame: u64,
    pub release_frame: u64,
}
```

#### Enums

##### Is

Derives:
- Debug
- Clone
- Copy

Implements:
- None

##### Mouse

Derives:
- Debug
- Clone
- PartialEq

Implements:
- None

##### GamepadButton

Derives:
- Debug
- Clone

Implements:
- None

##### GamepadAxis

Derives:
- Debug
- Clone

Implements:
- None

```rust
#[derive(Debug, Clone, Copy)]
pub enum Is { Pressed, Released, Held }

#[derive(Debug, Clone, PartialEq)]
pub enum Mouse { Left, Right, Middle, Back, Forward, Other(u16) }

#[derive(Debug, Clone)]
pub enum GamepadButton {
    A, B, X, Y, LB, RB, LT, RT, Back, Start, Guide,
    LeftStick, RightStick, DPadUp, DPadDown, DPadLeft, DPadRight,
    Other(u8),
}

#[derive(Debug, Clone)]
pub enum GamepadAxis {
    LeftX, LeftY, RightX, RightY, LeftTrigger, RightTrigger,
}
```

#### Constants

```rust
pub const MAX_GAMEPADS: usize = 4;
pub const GAMEPAD_AXIS_DEADZONE: f32 = 0.15;  // (associated const on Events)
```

#### Methods

| Signature | Description |
|-----------|-------------|
| `pub fn new() -> Self` | Create new input state |
| `pub fn clear(&mut self)` | Reset everything to defaults |
| `pub fn end_frame(&mut self)` | Call at end of frame (increments `frame`, clears scroll/resize/network/`close_requested`/custom events) |
| `pub fn process_window_event(&mut self, event: &WindowEvent, _window: &Window)` | Process a winit event |
| `pub fn process_gilrs_event(&mut self, event: &gilrs::Event)` | Process a gilrs gamepad event |
| `pub fn key(&self, kc: KeyCode, action: Is) -> bool` | Check key state |
| `pub fn key_combo(&self, primary: KeyCode, modifier: KeyCode, action: Is) -> bool` | Check combo (e.g. Ctrl+S) |
| `pub fn key_combo_n(&self, keys: &[(KeyCode, Is)]) -> bool` | Check multiple keys |
| `pub fn any_key(&self, action: Is) -> bool` | Any key matches? |
| `pub fn mouse(&self, m: Mouse, action: Is) -> bool` | Check mouse button state |
| `pub fn any_mouse(&self, action: Is) -> bool` | Any mouse button matches? |
| `pub fn gamepad_connected(&self, id: usize) -> bool` | Is gamepad connected? |
| `pub fn gamepad_count(&self) -> usize` | Number of connected gamepads |
| `pub fn gamepad_button(&self, id: usize, button: GamepadButton, action: Is) -> bool` | Check gamepad button |
| `pub fn any_gamepad_button(&self, id: usize, action: Is) -> bool` | Any button on gamepad? |
| `pub fn any_gamepad(&self, action: Is) -> bool` | Any button on any gamepad? |
| `pub fn gamepad_axis_raw(&self, id: usize, axis: GamepadAxis) -> f32` | Raw axis value [-1, 1] |
| `pub fn gamepad_axis(&self, id: usize, axis: GamepadAxis) -> f32` | Axis value with default deadzone |
| `pub fn gamepad_axis_deadzoned(&self, id: usize, axis: GamepadAxis, deadzone: f32) -> f32` | Axis value with custom deadzone |

#### Custom Events

| Signature | Description |
|-----------|-------------|
| `pub fn emit_event(&mut self, name: &str)` | Emit a named custom event (no payload) |
| `pub fn emit_event_with<D: DataType>(&mut self, name: &str, value: D)` | Emit a named custom event with typed payload |
| `pub fn was_event_emitted(&self, name: &str) -> bool` | Check if a custom event was emitted this frame |
| `pub fn event_emitted_count(&self, name: &str) -> u32` | How many times emitted this frame |
| `pub fn event_payload<D: DataType>(&self, name: &str) -> OpticResult<D>` | Decode first payload as `D` (errors on mismatch) |
| `pub fn event_payloads<D: DataType>(&self, name: &str) -> OpticResult<Vec<D>>` | Decode all payloads as `D` in emission order |

Custom events persist for the current frame and are cleared at `end_frame`. Use them
for interaction callbacks, timer completions, or any named one-shot communication. The
`_event`/`event_` naming is deliberately explicit to avoid confusion with keyboard,
mouse, or gamepad input events.

**Note:** `KeyCode` is re-exported from `winit::keyboard::KeyCode`.

### ScreenInfo

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct ScreenInfo {
    pub name: String,
    pub size: Size2D,
    pub refresh_rate: u32,
    pub scale_factor: f64,
    pub position: Coord2D,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn from_handle(handle: &winit::monitor::MonitorHandle) -> Self` | Build from winit monitor handle |

### Re-exports

`optic_window` re-exports `winit` and `gilrs` crates.

---

## 4. Renderer (`optic_render`)

### Rendering Pipeline Overview

Optic is a **forward renderer** built on OpenGL 4.6. Every frame, the engine
clears the screen, iterates over meshes, and issues draw calls through a
shader-programmed vertex-to-fragment pipeline.

#### Draw Call Flow

```text
gpu.render3d(&mesh, &camera)
  │
  ├─ 1. Visibility check (skip if hidden/empty)
  ├─ 2. Shader bind (glUseProgram)
  ├─ 3. Uniform upload (uView, uProj, uTfm)
  ├─ 4. Texture bind (glActiveTexture + glBindTexture per slot)
  ├─ 5. SSBO bind (glBindBufferBase per slot)
  └─ 6. Draw call (glDrawElements / glDrawArrays / *Instanced)
```

For 2D meshes, `gpu.render2d(&mesh)` uses an orthographic projection derived
from the canvas size, and additionally sets a `uLayer` uniform for z-ordering.

#### Mesh Buffer Lifecycle (VAO/VBO/IBO)

When a mesh is uploaded from an asset type to the GPU:

1. **Create VAO** — `glGenVertexArrays` produces a vertex array object
2. **Create VBO** — `glGenBuffers` produces a vertex buffer object
3. **Upload data** — `glBufferData` fills the VBO with interleaved vertex
   data (positions, normals, UVs, colours)
4. **Configure attributes** — `glVertexAttribPointer` +
   `glEnableVertexAttribArray` for each attribute in the layout
5. **Create IBO** (if indexed) — `glGenBuffers` + `glBufferData` for the
   element array buffer

```text
Mesh3DFile (CPU)          GPU Buffers
┌──────────────┐         ┌──────────────────────────────┐
│ positions[]  │ ──────▶ │ VBO (interleaved)            │
│ normals[]    │         │  [pos|nrm|uv|col|...] × N    │
│ uvs[]        │         │                              │
│ colours[]    │         │  VAO references VBO + layout │
│ indices[]    │ ──────▶ │ IBO (element array)          │
└──────────────┘         └──────────────────────────────┘
```

#### Shader Compilation

Shaders are compiled from GLSL source. Pipeline shaders use comment markers
(`// V`, `// F`, `// VERT`, `// FRAG`, etc.) to delimit vertex and fragment
sections in a single `.glsl` file. Compute shaders use the entire file.

```text
.glsl source → compile_shader(vert) + compile_shader(frag)
             → link_program(v, f) → GL program ID
             → Shader { id, bound_textures[16], bound_storages[16] }
```

Each shader maintains 16 texture slots and 16 SSBO slots. Textures are
auto-assigned via `attach_texture` or explicitly placed via
`bind_texture(tex, Slot::S3)` to match `layout(binding = 3)` in GLSL.

#### Instanced Rendering

For drawing many copies of the same mesh, Optic uses GPU instancing. An
`InstanceBuffer` holds per-instance data (position, rotation, scale, colour,
custom attributes) interleaved in a single VBO with `glVertexAttribDivisor(1)`.

```text
Single draw call renders N instances:
  MeshHandle
    ├─ vertex VBO (shared geometry)
    ├─ IBO (shared indices)
    └─ instance VBO (per-instance transforms, divisor=1)
         │
         ▼
  glDrawElementsInstanced(TRIANGLES, count, UNSIGNED_INT, null, N)
```

The `InstanceBuffer` maintains a CPU mirror for instant reads and partial
writes without GPU round-trips.

#### Canvas (Render-to-Texture)

The `Canvas` type wraps OpenGL FBOs for off-screen rendering. Supports
multiple colour attachments (MRT), depth/stencil, MSAA resolve, pixel
readback, and disk export.

```text
gpu.set_render_target(&RenderTarget::Canvas(&canvas))?;
gpu.clear();
gpu.render3d(&mesh, &camera);
canvas.blit_to_screen(window_size);  // present to screen
```

#### GL State Defaults

| State | Default | Controlled by |
|---|---|---|
| Depth testing | Enabled | `GL::enable_depth()` |
| Alpha blending | Enabled (SRC_ALPHA, ONE_MINUS_SRC_ALPHA) | `GL::enable_alpha()` |
| Back-face culling | Enabled (counter-clockwise) | `gpu.set_culling()` |
| MSAA | Enabled (4 samples) | `gpu.set_msaa()` |
| Polygon mode | Filled | `gpu.set_poly_mode()` |
| Clear colour | Grey (0.5) | `gpu.set_bg_color()` |

### Viewport

Derives:
- None

Implements:
- None

A rectangular region of the render target, measured in pixels from the lower-left
corner. Used by [`GPU::viewport`] / [`GPU::set_viewport`] and by canvases for
split-screen or picture-in-picture effects.

```rust
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
```

### GPU

Derives:
- None

Implements:
- None

```rust
// No derives — owns GL resources.
pub struct GPU {
    pub(crate) ctx: RenderContext,
    poly_mode: PolygonMode,
    cull_face: CullFace,
    bg_color: RGBA,
    msaa: bool,
    msaa_samples: u32,
    culling: bool,
    fallback_shader2d: Shader,
    fallback_shader3d: Shader,
    fallback_texture: Texture2D,
    fallback_font: FontFamily,
    canvas_size: Size2D,
    pub(crate) current_target_size: Size2D,
    pub(crate) max_color_attachments: i32,
    pub(crate) max_draw_buffers: i32,
    pub(crate) max_samples: i32,
    fallback_shader_text2d: Shader,
    fallback_shader_text3d: Shader,
}
```

#### Construction

| Signature | Description |
|-----------|-------------|
| `pub fn new_headless() -> OpticResult<Self>` | Create a headless GPU context (pbuffer only) |
| `pub fn new_windowed(handle: RawWindowHandle, display_handle: RawDisplayHandle, size: Size2D) -> OpticResult<Self>` | Create GPU context from a window |

#### Info

| Signature | Description |
|-----------|-------------|
| `pub fn version(&self) -> &str` | GL version string |
| `pub fn lang_version(&self) -> &str` | GLSL version string |
| `pub fn name(&self) -> &str` | Renderer name |
| `pub fn log_backend_info(&self)` | Log GL info to stdout |
| `pub fn log_info(&self)` | Log all GPU info |

#### State

| Signature | Description |
|-----------|-------------|
| `pub fn clear(&self)` | Clear color + depth buffers |
| `pub fn set_msaa_samples(&mut self, samples: u32)` | Set MSAA sample count |
| `pub fn set_bg_color(&mut self, color: RGBA)` | Set background clear color |
| `pub fn set_poly_mode(&mut self, mode: PolygonMode)` | Set polygon mode |
| `pub fn toggle_wireframe(&mut self)` | Toggle between Filled and WireFrame |
| `pub fn set_msaa(&mut self, enable: bool)` | Enable/disable MSAA |
| `pub fn toggle_msaa(&mut self)` | Toggle MSAA |
| `pub fn set_culling(&mut self, enable: bool)` | Enable/disable backface culling |
| `pub fn toggle_culling(&mut self)` | Toggle culling |
| `pub fn set_cull_face(&mut self, cull_face: CullFace)` | Set which face to cull |
| `pub fn flip_cull_face(&mut self)` | Swap cull face |
| `pub fn set_canvas_size(&mut self, size: Size2D)` | Set canvas/render size |
| `pub fn set_wire_width(&mut self, width: f32)` | Wireframe line width |
| `pub fn set_point_size(&self, size: f32)` | Point size |
| `pub fn reset_state(&mut self)` | Reset all GPU state to defaults |
| `pub fn viewport(&self) -> Viewport` | Get the current viewport rect |
| `pub fn set_viewport(&self, vp: Viewport)` | Set the viewport rect |
| `pub fn flush(&self)` | Flush OpenGL commands (non-blocking) |
| `pub fn finish(&self)` | Block until all OpenGL commands complete |

#### Fallback Assets

| Signature | Description |
|-----------|-------------|
| `pub fn fallback_shader3d(&self) -> Shader` | Default 3D shader |
| `pub fn fallback_shader2d(&self) -> Shader` | Default 2D shader |

#### GPU Resource Upload

| Signature | Description |
|-----------|-------------|
| `pub fn upload_mesh3d(&self, file: &Mesh3DFile) -> Mesh3D` | Upload 3D mesh to GPU |
| `pub fn upload_mesh2d(&self, file: &Mesh2DFile) -> Mesh2D` | Upload 2D mesh to GPU |
| `pub fn upload_shader(&self, asset: &ShaderFile) -> OpticResult<Shader>` | Compile and upload shader |
| `pub fn upload_texture(&self, image: &TextureFile) -> Texture2D` | Upload texture to GPU |
| `pub fn upload_gradient(&self, gradient: &Gradient, resolution: u32) -> Texture2D` | Bake gradient to 1D RGBA texture |
| `pub fn upload_canvas(&mut self, desc: &CanvasDesc) -> OpticResult<Canvas>` | Create offscreen render target |

#### Rendering

| Signature | Description |
|-----------|-------------|
| `pub fn set_render_target(&mut self, target: &RenderTarget) -> OpticResult<()>` | Set active render target |
| `pub fn clear_target(&mut self, color: Option<RGBA>, depth: bool)` | Clear current render target (if color is None, uses bg_color) |
| `pub fn current_render_target_size(&self) -> Size2D` | Size of current render target |
| `pub fn render3d(&self, mesh: &Mesh3D, camera: &Camera)` | Render a 3D mesh |
| `pub fn render2d(&self, mesh: &Mesh2D)` | Render a 2D mesh (orthographic) |
| `pub fn render_text2d(&self, text: &mut Text2D, camera: &Camera)` | Render screen-space text |
| `pub fn render_text3d(&self, text: &mut Text3D, camera: &Camera)` | Render world-space text |

### RenderContext

Derives:
- None

Implements:
- None

```rust
// No derives — owns EGL resources.
pub struct RenderContext {
    pub(crate) display: egl::Display,
    pub(crate) context: egl::Context,
    config: egl::Config,
    pub(crate) surfaces: Vec<WindowSurface>,
    active_index: Option<usize>,
    gl_ver: String,
    glsl_ver: String,
    device: String,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new_headless() -> OpticResult<Self>` | Create headless context |
| `pub fn new_windowed(handle: RawWindowHandle, display_handle: RawDisplayHandle, size: Size2D) -> OpticResult<Self>` | Create context attached to a window |
| `pub fn attach_window(&mut self, handle: RawWindowHandle, size: Size2D) -> OpticResult<usize>` | Attach an additional window surface |
| `pub fn resize_window(&mut self, index: usize, size: Size2D)` | Resize a window surface |
| `pub fn make_current(&self, index: usize) -> OpticResult<()>` | Make a surface current |
| `pub fn swap_buffers(&self, index: usize) -> OpticResult<()>` | Swap buffers for a surface |
| `pub fn clear(&self)` | Clear color + depth |
| `pub fn set_vsync(&self, enable: bool)` | Enable/disable vsync |
| `pub fn set_clear_color(&self, color: RGBA)` | Set clear color |

#### WindowSurface

Derives:
- None

Implements:
- None

```rust
// No derives — owns EGL surface.
pub struct WindowSurface {
    pub(crate) surface: egl::Surface,
    size: Size2D,
}
```

### GL Static Methods

#### GL

Derives:
- None

Implements:
- None

```rust
// No derives — unit struct, all methods are associated functions.
pub struct GL;
```

| Signature | Description |
|-----------|-------------|
| `pub fn clear()` | Clear color + depth buffers |
| `pub fn set_clear(color: RGBA)` | Set clear color via `glClearColor` |
| `pub fn resize(size: Size2D)` | Set viewport |
| `pub fn poly_mode(mode: PolygonMode)` | Set polygon render mode |
| `pub fn enable_msaa(enable: bool)` | Enable/disable MSAA |
| `pub fn enable_depth(enable: bool)` | Enable/disable depth testing |
| `pub fn enable_alpha(enable: bool)` | Enable/disable alpha blending (SRC_ALPHA, ONE_MINUS_SRC_ALPHA) |
| `pub fn enable_cull(enable: bool)` | Enable/disable face culling |
| `pub fn set_cull_face(face: CullFace)` | Set which face to cull |
| `pub fn set_point_size(size: f32)` | Set point size |
| `pub fn set_wire_width(width: f32)` | Set line width |
| `pub fn bind_shader(id: u32)` | Bind shader program |
| `pub fn unbind_shader()` | Unbind shader |
| `pub fn bind_texture_at(tex_id: u32, slot: u32)` | Bind texture to slot |
| `pub fn unbind_texture()` | Unbind texture |
| `pub fn bind_vao(id: u32)` | Bind VAO |
| `pub fn unbind_vao()` | Unbind VAO |
| `pub fn bind_buffer(id: u32)` | Bind VBO |
| `pub fn unbind_buffer()` | Unbind VBO |
| `pub fn bind_ebo(id: u32)` | Bind EBO |
| `pub fn unbind_ebo()` | Unbind EBO |
| `pub fn bind_ssbo(id: u32)` | Bind SSBO |
| `pub fn unbind_ssbo()` | Unbind SSBO |

### Camera

Derives:
- None

Implements:
- None

```rust
// No derives — owns transform state.
pub struct Camera {
    pub transform: CamTransform,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(size: Size2D, proj: CamProj) -> Self` | Create camera with size and projection type |
| `pub fn match_canvas_size(canvas: &Canvas, proj: CamProj) -> Self` | Create camera matching canvas size |
| `pub fn pre_update(&mut self)` | Recalculate view/projection matrices |
| `pub fn fov(&self) -> f32` | Field of view (degrees) |
| `pub fn ortho_scale(&self) -> f32` | Orthographic scale |
| `pub fn proj(&self) -> CamProj` | Projection type |
| `pub fn clip(&self) -> ClipDist` | Clip distances |
| `pub fn set_clip(&mut self, clip: ClipDist)` | Set clip distances |
| `pub fn set_clip_near(&mut self, near: f32)` | Set near clip |
| `pub fn set_clip_far(&mut self, far: f32)` | Set far clip |
| `pub fn set_size(&mut self, size: Size2D)` | Set viewport size |
| `pub fn set_proj(&mut self, proj: CamProj)` | Set projection type |
| `pub fn set_fov(&mut self, fov: f32)` | Set FOV |
| `pub fn add_fov(&mut self, value: f32)` | Add to FOV |
| `pub fn set_ortho_scale(&mut self, value: f32)` | Set ortho scale |
| `pub fn add_ortho_scale(&mut self, value: f32)` | Add to ortho scale |
| `pub fn fly_forward(&mut self, speed: f32)` | Move camera forward |
| `pub fn fly_back(&mut self, speed: f32)` | Move camera backward |
| `pub fn fly_left(&mut self, speed: f32)` | Move camera left |
| `pub fn fly_right(&mut self, speed: f32)` | Move camera right |
| `pub fn fly_up(&mut self, speed: f32)` | Move camera up |
| `pub fn fly_down(&mut self, speed: f32)` | Move camera down |
| `pub fn spin_x(&mut self, speed: f32)` | Pitch (degrees) |
| `pub fn spin_y(&mut self, speed: f32)` | Yaw (degrees) |
| `pub fn spin_z(&mut self, speed: f32)` | Roll (degrees) |

### CamTransform

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct CamTransform {
    pub pos: Vector3<f32>,
    pub rot: Vector3<f32>,
    pub fov: f32,
    pub clip: ClipDist,
    pub size: Size2D,
    pub proj: CamProj,
    pub view_matrix: Matrix4<f32>,
    pub ortho_scale: f32,
    pub front: Vector3<f32>,
    pub persp_matrix: Matrix4<f32>,
    pub ortho_matrix: Matrix4<f32>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn calc_matrices(&mut self)` | Recalculate view/projection matrices from position/rotation |
| `pub fn view_matrix(&self) -> Matrix4<f32>` | Current view matrix |
| `pub fn proj_matrix(&self) -> Matrix4<f32>` | Current projection matrix |

### Mesh3D

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct Mesh3D {
    hidden: bool,          // (private)
    handle: MeshHandle,    // (private)
    shader: Option<Shader>,// (private)
    transform: Transform3D,// (private)
    draw_mode: DrawMode,   // (private)
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(handle: MeshHandle) -> Self` | Create from GPU handle with default settings |
| `pub fn set_shader(&mut self, shader: Shader)` | Attach a shader |
| `pub fn remove_shader(&mut self)` | Detach the current shader |
| `pub fn shader(&self) -> Option<&Shader>` | Reference to the shader, if any |
| `pub fn handle(&self) -> &MeshHandle` | Reference to the GPU handle |
| `pub fn transform(&self) -> &Transform3D` | Reference to the transform |
| `pub fn transform_mut(&mut self) -> &mut Transform3D` | Mutable reference to the transform |
| `pub fn draw_mode(&self) -> DrawMode` | Current draw mode |
| `pub fn set_draw_mode(&mut self, draw_mode: DrawMode)` | Set draw mode |
| `pub fn index_count(&self) -> u32` | Number of indices |
| `pub fn vertex_count(&self) -> u32` | Number of vertices |
| `pub fn has_indices(&self) -> bool` | Has index buffer? |
| `pub fn is_empty(&self) -> bool` | Zero vertices? |
| `pub fn is_visible(&self) -> bool` | Not hidden and non-empty |
| `pub fn set_visibility(&mut self, enable: bool)` | Show or hide |
| `pub fn toggle_visibility(&mut self)` | Toggle hidden flag |
| `pub fn update(&mut self)` | Recalculate transform matrix |
| `pub fn delete(self)` | Delete GPU resources |
| `pub fn log_info(&self)` | Print mesh info |
| `pub fn render(&self, view: &Matrix4<f32>, proj: &Matrix4<f32>)` | Render with view and projection matrices |

### Mesh2D

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct Mesh2D {
    hidden: bool,            // (private)
    handle: MeshHandle,      // (private)
    shader: Option<Shader>,  // (private)
    transform: Transform2D,  // (private)
    draw_mode: DrawMode,     // (private)
}
```

Same methods as Mesh3D, plus:

| Signature | Description |
|-----------|-------------|
| `pub fn render(&self, proj: &Matrix4<f32>)` | Render with explicit projection matrix |

### MeshHandle

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct MeshHandle {
    // All fields are pub(crate).
    // Use accessor methods below for read access.
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn draw_as(&self, mode: DrawMode)` | Draw with a specific draw mode |
| `pub fn layouts(&self) -> &Vec<(ATTRInfo, u32)>` | Vertex layouts |
| `pub fn has_indices(&self) -> bool` | Uses indexed drawing? |
| `pub fn vertex_count(&self) -> u32` | Vertex count |
| `pub fn index_count(&self) -> u32` | Index count |
| `pub fn vao_id(&self) -> u32` | VAO ID |
| `pub fn vertex_buffer_id(&self) -> u32` | Vertex buffer ID |
| `pub fn index_buffer_id(&self) -> u32` | Index buffer ID |
| `pub fn vertex_stride(&self) -> u32` | Vertex stride in bytes |
| `pub fn instance_buf_id(&self) -> u32` | Instance buffer ID (0 if not instanced) |
| `pub fn instance_count(&self) -> u32` | Instance count (0 if not instanced) |
| `pub fn set_instances(&mut self, buffer: &InstanceBuffer)` | Bind an instance buffer for instanced rendering |
| `pub fn update_vertex<D: DataType>(&self, index: u32, attr_index: usize, value: D) -> OpticResult<()>` | Update a single vertex attribute |
| `pub fn vertex<D: DataType>(&self, index: u32, attr_index: usize) -> OpticResult<D>` | Read back a single vertex attribute |
| `pub fn write_range(&self, start_vertex: u32, data: &[u8]) -> OpticResult<()>` | Write raw bytes at a vertex offset |
| `pub fn delete(self)` | Free GPU resources |

### InstanceDesc3D

Derives:
- None

Implements:
- None

```rust
// No derives — describes GPU instance layout.
pub struct InstanceDesc3D {
    pub pos_attr: Pos3DATTR,
    pub rot_attr: Rot3DATTR,
    pub scale_attr: Scale3DATTR,
    pub col_attr: ColorATTR,
    pub custom_attrs: Vec<CustomATTR>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn empty() -> Self` | Empty descriptor |
| `pub fn from_positions(positions: &[Vector3<f32>]) -> Self` | Initialize from position vectors |
| `pub fn from_transforms(transforms: &[Matrix4<f32>]) -> Self` | Extract pos/rot/scale from 4×4 matrices |
| `pub fn add_custom_attr(&mut self, attr: CustomATTR) -> &mut Self` | Add a custom per-instance attribute |
| `pub fn upload(&self) -> OpticResult<InstanceBuffer>` | Upload to GPU |

### InstanceDesc2D

Derives:
- None

Implements:
- None

```rust
// No derives — describes GPU instance layout.
pub struct InstanceDesc2D {
    pub pos_attr: Pos2DATTR,
    pub rot_attr: Rot2DATTR,
    pub scale_attr: Scale2DATTR,
    pub col_attr: ColorATTR,
    pub custom_attrs: Vec<CustomATTR>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn empty() -> Self` | Empty descriptor |
| `pub fn add_custom_attr(&mut self, attr: CustomATTR) -> &mut Self` | Add a custom per-instance attribute |
| `pub fn upload(&self) -> OpticResult<InstanceBuffer>` | Upload to GPU |

### InstanceBuffer

Derives:
- None

Implements:
- None

```rust
// No derives — owns GPU buffer.
pub struct InstanceBuffer {
    pub(crate) buf_id: u32,
    pub(crate) capacity: u32,
    pub(crate) count: u32,
    pub(crate) stride: u32,
    pub(crate) layouts: Vec<(ATTRInfo, u32)>,
    pub(crate) cpu_mirror: Vec<u8>,
    pub(crate) kind: InstanceKind,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn count(&self) -> u32` | Number of instances currently stored |
| `pub fn len(&self) -> usize` | Number of instances as usize |
| `pub fn is_empty(&self) -> bool` | True if no instances |
| `pub fn clear(&mut self)` | Remove all instances (O1) |
| `pub fn capacity(&self) -> u32` | Allocated capacity (instances) |
| `pub fn layouts(&self) -> &[(ATTRInfo, u32)]` | Attribute layout descriptions |
| `pub fn update_instance<D: DataType>(&mut self, index: u32, attr_index: usize, value: D) -> OpticResult<()>` | Update a single attribute on one instance |
| `pub fn instance<D: DataType>(&self, index: u32, attr_index: usize) -> OpticResult<D>` | Read a single attribute from one instance |
| `pub fn update_custom<D: DataType>(&mut self, index: u32, name: &str, value: D) -> OpticResult<()>` | Update a custom attribute by name |
| `pub fn custom_attr<D: DataType>(&self, index: u32, name: &str) -> OpticResult<D>` | Read a custom attribute by name |
| `pub fn set_position(&mut self, index: u32, pos: Vector3<f32>) -> OpticResult<()>` | Set instance position (3D) |
| `pub fn position(&self, index: u32) -> OpticResult<Vector3<f32>>` | Get instance position (3D) |
| `pub fn set_rotation(&mut self, index: u32, rot: Vector4<f32>) -> OpticResult<()>` | Set instance rotation as quaternion |
| `pub fn rotation(&self, index: u32) -> OpticResult<Vector4<f32>>` | Get instance rotation as quaternion |
| `pub fn set_scale(&mut self, index: u32, scale: Vector3<f32>) -> OpticResult<()>` | Set instance scale (3D) |
| `pub fn scale(&self, index: u32) -> OpticResult<Vector3<f32>>` | Get instance scale (3D) |
| `pub fn set_color(&mut self, index: u32, color: RGBA) -> OpticResult<()>` | Set instance color |
| `pub fn color(&self, index: u32) -> OpticResult<RGBA>` | Get instance color |
| `pub fn set_instance_count(&mut self, new_count: u32)` | Update visible instance count (truncates or extends) |
| `pub fn reserve(&mut self, additional: u32)` | Pre-allocate capacity |
| `pub fn shrink_to_fit(&mut self)` | Shrink capacity to match count |
| `pub fn push_raw(&mut self, bytes: &[u8]) -> OpticResult<u32>` | Append raw bytes as a new instance; returns index |
| `pub fn pop(&mut self) -> OpticResult<()>` | Remove the last instance |
| `pub fn remove_instance(&mut self, index: u32) -> OpticResult<()>` | Remove instance (swap-remove — last moves into slot) |
| `pub fn remove_instance_ordered(&mut self, index: u32) -> OpticResult<()>` | Remove instance (shift elements, preserves order) |
| `pub fn write_all(&mut self, desc: &InstanceDesc3D) -> OpticResult<()>` | Bulk-write all instances from a descriptor |
| `pub fn write_range(&mut self, start: u32, bytes: &[u8]) -> OpticResult<()>` | Write raw bytes starting at a given instance index |
| `pub fn delete(self)` | Free GPU resources |

### Shader

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct Shader {
    pub workers: Workers,
    pub id: u32,
    pub is_compute: bool,
    pub bound_textures: Vec<Option<u32>>,
    pub bound_storages: Vec<Option<u32>>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(id: u32, is_compute: bool) -> Self` | Wrap a shader program ID |
| `pub fn attach_texture(&mut self, tex: &Texture2D)` | Attach texture to next available slot |
| `pub fn attach_storage(&mut self, sbo: &StorageBuffer)` | Attach SSBO to next available slot |
| `pub fn bind_texture(&mut self, tex: &Texture2D, slot: Slot)` | Bind texture to specific slot |
| `pub fn bind_storage(&mut self, sbo: &StorageBuffer, slot: Slot)` | Bind SSBO to specific slot |
| `pub fn delete(self)` | Delete the shader program |
| `pub fn bind(&self)` | Use this shader |
| `pub fn unbind(&self)` | Unbind shader |
| `pub fn compute(&self)` | Dispatch compute shader with worker group counts |
| `pub fn uniform_location(&self, name: &str) -> Option<u32>` | Query uniform location |
| `pub fn bound_textures_info(&self) -> Vec<(u32, u32)>` | List of (tex_id, slot) bindings |
| `pub fn bound_storages_info(&self) -> Vec<(u32, u32)>` | List of (sbo_id, slot) bindings |
| `pub fn bind_textures(&self)` | Bind all attached textures |
| `pub fn bind_storages(&self)` | Bind all attached SSBOs |
| `pub fn set_i32(&self, name: &str, v: i32)` | Set int uniform |
| `pub fn set_u32(&self, name: &str, v: u32)` | Set uint uniform |
| `pub fn set_f32(&self, name: &str, v: f32)` | Set float uniform |
| `pub fn set_vec2_f32(&self, name: &str, v: Vector2<f32>)` | Set vec2 uniform |
| `pub fn set_vec3_f32(&self, name: &str, v: Vector3<f32>)` | Set vec3 uniform |
| `pub fn set_vec4_f32(&self, name: &str, v: Vector4<f32>)` | Set vec4 uniform |
| `pub fn set_vec2_i32(&self, name: &str, v: Vector2<i32>)` | Set ivec2 uniform |
| `pub fn set_vec3_i32(&self, name: &str, v: Vector3<i32>)` | Set ivec3 uniform |
| `pub fn set_vec4_i32(&self, name: &str, v: Vector4<i32>)` | Set ivec4 uniform |
| `pub fn set_vec2_u32(&self, name: &str, v: Vector2<u32>)` | Set uvec2 uniform |
| `pub fn set_vec3_u32(&self, name: &str, v: Vector3<u32>)` | Set uvec3 uniform |
| `pub fn set_vec4_u32(&self, name: &str, v: Vector4<u32>)` | Set uvec4 uniform |
| `pub fn set_m2_f32(&self, name: &str, m: Matrix2<f32>)` | Set mat2 uniform |
| `pub fn set_m3_f32(&self, name: &str, m: Matrix3<f32>)` | Set mat3 uniform |
| `pub fn set_m4_f32(&self, name: &str, m: Matrix4<f32>)` | Set mat4 uniform |

### Texture2D

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct Texture2D {
    pub id: u32,
    pub size: Size2D,
    pub fmt: ImgFormat,
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(id: u32, size: Size2D, fmt: ImgFormat, filter: ImgFilter, wrap: ImgWrap) -> Self` | Wrap a GL texture ID |
| `pub fn size(&self) -> Size2D` | Texture size |
| `pub fn wrap(&self) -> ImgWrap` | Current wrap mode |
| `pub fn set_wrap(&mut self, wrap: ImgWrap)` | Set wrap mode |
| `pub fn filter(&self) -> ImgFilter` | Current filter mode |
| `pub fn set_filter(&mut self, filter: ImgFilter)` | Set filter mode |
| `pub fn delete(self)` | Delete the GL texture |

### StorageBuffer

Derives:
- None

Implements:
- None

```rust
// No derives — owns GPU buffer.
pub struct StorageBuffer {
    pub id: u32,
    pub size: usize,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(size: usize) -> Self` | Create SSBO of given size |
| `pub fn resize(&mut self, size: usize)` | Resize buffer |
| `pub fn fill(&mut self, data: &[u8])` | Fill buffer with byte data |
| `pub fn subfill(&mut self, offset: usize, data: &[u8])` | Write data at offset |
| `pub fn fetch(&self) -> Vec<u8>` | Read back buffer contents |
| `pub fn delete(self)` | Delete buffer |

### Transform2D

Derives:
- Clone
- Debug

Implements:
- Default

```rust
#[derive(Clone, Debug)]
pub struct Transform2D {
    pos: Vector2<f32>,
    rot: f32,
    scale: Vector2<f32>,
    layer: u8,
    aspect: f32,
    matrix: Matrix4<f32>,
}
impl Default for Transform2D { /* identity transform */ }
```

| Signature | Description |
|-----------|-------------|
| `pub fn calc_matrix(&mut self)` | Recalculate transform matrix from pos/rot/scale |
| `pub fn pos(&self) -> Vector2<f32>` | Position |
| `pub fn rotation(&self) -> f32` | Rotation (degrees) |
| `pub fn scale_factor(&self) -> Vector2<f32>` | Scale |
| `pub fn layer(&self) -> u8` | Z layer |
| `pub fn matrix(&self) -> Matrix4<f32>` | Current 4×4 transform matrix |
| `pub fn translate(&mut self, x: f32, y: f32)` | Translate |
| `pub fn translate_x(&mut self, x: f32)` | Translate X |
| `pub fn translate_y(&mut self, y: f32)` | Translate Y |
| `pub fn set_position(&mut self, x: f32, y: f32)` | Set position |
| `pub fn set_position_x(&mut self, x: f32)` | Set position X |
| `pub fn set_position_y(&mut self, y: f32)` | Set position Y |
| `pub fn rotate(&mut self, rot: f32)` | Add rotation |
| `pub fn set_rotation(&mut self, rot: f32)` | Set rotation |
| `pub fn set_layer(&mut self, layer: u8)` | Set Z layer |
| `pub fn scale(&mut self, x: f32, y: f32)` | Add scale |
| `pub fn scale_uniform(&mut self, xy: f32)` | Uniform add scale |
| `pub fn scale_x(&mut self, x: f32)` | Add scale X |
| `pub fn scale_y(&mut self, y: f32)` | Add scale Y |
| `pub fn set_scale(&mut self, x: f32, y: f32)` | Set scale |
| `pub fn set_scale_uniform(&mut self, xy: f32)` | Uniform set scale |
| `pub fn set_scale_x(&mut self, x: f32)` | Set scale X |
| `pub fn set_scale_y(&mut self, y: f32)` | Set scale Y |

### Transform3D

Derives:
- Clone
- Debug

Implements:
- Default

```rust
#[derive(Clone, Debug)]
pub struct Transform3D {
    pos: Vector3<f32>,
    rot: Vector3<f32>,
    scale: Vector3<f32>,
    matrix: Matrix4<f32>,
}
impl Default for Transform3D {}
```

| Signature | Description |
|-----------|-------------|
| `pub fn calc_matrix(&mut self)` | Recalculate transform matrix |
| `pub fn pos(&self) -> Vector3<f32>` | Position |
| `pub fn rotation(&self) -> Vector3<f32>` | Rotation (degrees) |
| `pub fn scale_factor(&self) -> Vector3<f32>` | Scale |
| `pub fn matrix(&self) -> Matrix4<f32>` | Current 4×4 transform matrix |
| `pub fn translate(&mut self, x: f32, y: f32, z: f32)` | Translate |
| `pub fn translate_x(&mut self, x: f32)` | Translate X |
| `pub fn translate_y(&mut self, y: f32)` | Translate Y |
| `pub fn translate_z(&mut self, z: f32)` | Translate Z |
| `pub fn set_position(&mut self, x: f32, y: f32, z: f32)` | Set position |
| `pub fn set_position_x(&mut self, x: f32)` | Set position X |
| `pub fn set_position_y(&mut self, y: f32)` | Set position Y |
| `pub fn set_position_z(&mut self, z: f32)` | Set position Z |
| `pub fn rotate(&mut self, x: f32, y: f32, z: f32)` | Add rotation |
| `pub fn rotate_x(&mut self, x: f32)` | Add rotation X |
| `pub fn rotate_y(&mut self, y: f32)` | Add rotation Y |
| `pub fn rotate_z(&mut self, z: f32)` | Add rotation Z |
| `pub fn set_rotation(&mut self, x: f32, y: f32, z: f32)` | Set rotation |
| `pub fn set_rotation_x(&mut self, x: f32)` | Set rotation X |
| `pub fn set_rotation_y(&mut self, y: f32)` | Set rotation Y |
| `pub fn set_rotation_z(&mut self, z: f32)` | Set rotation Z |
| `pub fn scale(&mut self, x: f32, y: f32, z: f32)` | Add scale |
| `pub fn scale_uniform(&mut self, xyz: f32)` | Uniform add scale |
| `pub fn scale_x(&mut self, x: f32)` | Add scale X |
| `pub fn scale_y(&mut self, y: f32)` | Add scale Y |
| `pub fn scale_z(&mut self, z: f32)` | Add scale Z |
| `pub fn set_scale(&mut self, x: f32, y: f32, z: f32)` | Set scale |
| `pub fn set_scale_uniform(&mut self, xyz: f32)` | Uniform set scale |
| `pub fn set_scale_x(&mut self, x: f32)` | Set scale X |
| `pub fn set_scale_y(&mut self, y: f32)` | Set scale Y |
| `pub fn set_scale_z(&mut self, z: f32)` | Set scale Z |

### Canvas

Derives:
- None

Implements:
- None

```rust
// No derives — owns GL framebuffer resources.
pub struct Canvas {
    fbo_id: u32,              // (pub(crate))
    resolve_fbo_id: u32,      // (pub(crate))
    msaa_rbos: Vec<u32>,      // (pub(crate))
    depth_stencil_rbo: u32,   // (pub(crate))
    color_texs: Vec<Texture2D>, // (pub(crate))
    depth_tex: Option<Texture2D>, // (pub(crate))
    size: Size2D,             // (pub(crate))
    samples: u32,             // (pub(crate))
    has_stencil: bool,        // (pub(crate))
    has_depth: bool,          // (pub(crate))
    depth_as_texture: bool,   // (pub(crate))
    desc: CanvasDesc,         // (pub(crate))
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(desc: &CanvasDesc) -> OpticResult<Self>` | Create offscreen framebuffer |
| `pub fn size(&self) -> Size2D` | Canvas size |
| `pub fn color_tex(&self, index: usize) -> OpticResult<&Texture2D>` | Get color attachment as texture |
| `pub fn depth_tex(&self) -> Option<&Texture2D>` | Get depth attachment as texture (if depth_as_texture) |
| `pub fn set_size(&mut self, new_size: Size2D) -> OpticResult<()>` | Resize canvas |
| `pub fn resolve(&self)` | Resolve MSAA to single-sample textures |
| `pub fn blit_to_screen(&self, window_size: Size2D)` | Copy canvas to screen |
| `pub fn set_renderable_area(&self, x: i32, y: i32, size: Size2D) -> OpticResult<()>` | Set scissor/viewport area |
| `pub fn read_pixels(&self, index: usize) -> OpticResult<Vec<u8>>` | Read pixel data from color attachment |
| `pub fn save_to_disk(&self, index: usize, path: &str) -> OpticResult<()>` | Save color attachment to PNG |
| `pub fn delete(&mut self)` | Free GPU resources |

### CanvasDesc

Derives:
- Clone
- Debug

Implements:
- Default

```rust
#[derive(Clone, Debug)]
pub struct CanvasDesc {
    pub size: Size2D,
    pub color_formats: Vec<ImgFormat>,
    pub depth: bool,
    pub depth_as_texture: bool,
    pub depth_compare: bool,
    pub stencil: bool,
    pub samples: u32,
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}
```

- `impl Default` — size: 1×1, one RGBA8 color, depth: true, samples: 1, etc.

### RenderTarget

Derives:
- None

Implements:
- None

```rust
// No derives — borrows Canvas.
pub enum RenderTarget<'a> {
    Screen,
    Canvas(&'a Canvas),
}
```

### Slot

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub enum Slot {
    S0, S1, S2, S3, S4, S5, S6, S7,
    S8, S9, S10, S11, S12, S13, S14, S15,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn as_index(&self) -> usize` | Slot as index (0-15) |
| `pub fn total_slots() -> usize` | Always 16 |

### Workers

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct Workers {
    // All fields are private.
    // Use groups()/set_groups() for access.
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn empty() -> Self` | All groups zero (no dispatch) |
| `pub fn one() -> Self` | All groups one |
| `pub fn set_groups(&mut self, x: u32, y: u32, z: u32)` | Set all three dimensions |
| `pub fn groups(&self) -> (u32, u32, u32)` | Get all three as `(x, y, z)` |

### Free Functions

```rust
// Shader compilation
pub fn compile_shader(src: &str, shader_type: GLenum) -> OpticResult<u32>;
pub fn link_program(vert: &str, frag: &str) -> OpticResult<u32>;
pub fn link_compute_program(src: &str) -> OpticResult<u32>;
pub fn delete_program(id: u32);

// Texture creation
pub fn create_texture(bytes: &[u8], size: Size2D, fmt: &ImgFormat, filter: &ImgFilter, wrap: &ImgWrap) -> u32;
pub fn delete_texture(id: u32);

// Mesh buffer helpers
pub fn create_mesh_buffer() -> (u32, u32);           // (vao_id, buf_id)
pub fn set_attr_layout(attr: &ATTRInfo, attr_id: u32, stride: usize, local_offset: usize);
pub fn fill_buffer(id: u32, data: &[u8]);
pub fn subfill_buffer(id: u32, offset: usize, data: &[u8]);
pub fn resize_buffer(id: u32, size: usize);
pub fn create_index_buffer() -> u32;
pub fn fill_index_buffer(id: u32, data: &[u32]);
```

---

## 5. Asset Types (`optic_render::asset`)

### Asset Handling Overview

Optic uses **runtime binary caching** — not build-time asset compilation. There
are no `build.rs` scripts, no CLI baking tools, and no asset manifest files.
Caching happens transparently inside each type's `from_disk()` constructor.

#### How It Works

```text
┌──────────────────────────────────────────────────────────────┐
│                  Debug Build                                 │
│                                                              │
│  Source file ──parse/bake──▶ CPU data ──save──▶ Binary cache │
│  (PNG/OBJ/GLSL/TTF/OGG)                   (optc/*.otxtr…)    │
│                                                              │
├──────────────────────────────────────────────────────────────┤
│                  Release Build                               │
│                                                              │
│  Binary cache ──load directly──▶ CPU data                    │
│  (optc/*.otxtr…)                                             │
│  Source file is never touched                                │
│                                                              │
└──────────────────────────────────────────────────────────────┘
```

- **Debug builds**: `from_disk(path)` re-parses the source file and overwrites
  the binary cache. Editing source assets takes effect on next run.
- **Release builds**: `from_disk(path)` loads directly from the binary cache.
  The source file is never read. If the cache is missing, loading fails.

#### Cache Path Convention

All asset types use `optic_file::cached_path(source, ext)` to compute cache
locations. Cache files live in an `optc/` subdirectory next to the source:

| Source file | Cache file |
|---|---|
| `assets/tex/foo.png` | `assets/tex/optc/foo.otxtr` |
| `models/cube.obj` | `models/optc/cube.omesh` |
| `shaders/main.glsl` | `shaders/optc/main.oshdr` |
| `fonts/arial.ttf` | `fonts/optc/arial.ofont` |
| `sound/bgm.ogg` | `sound/optc/bgm.omusic` |

#### Binary Cache Format

Every binary cache file shares a common header:

| Offset | Size | Field | Description |
|---|---|---|---|
| 0 | 8 | Magic | `b"/0PTIC_x"` — never changes |
| 8 | 2 | Version | `OPTIC_CACHE_VERSION` (currently `1`, u16 LE) |
| 10+ | varies | Payload | Asset-specific data |

The version field allows the cache format to evolve without breaking old caches.
If the version doesn't match, the cache is rejected and (in debug) regenerated.

#### Asset Types Summary

| Type | Source formats | Cache ext | Cache contents |
|---|---|---|---|
| `TextureFile` | `.png`, `.jpg`, … | `.otxtr` | Raw pixels + dimensions + format + filter + wrap |
| `Mesh3DFile` | `.obj`, `.stl`, procedural | `.omesh` | Vertex arrays + index buffer |
| `Mesh2DFile` | procedural | `.omesh` | 2D vertices + indices + layer |
| `ShaderFile` | `.glsl` | `.oshdr` | Vertex + fragment source strings |
| `FontFamilyFile` | `.ttf`, `.otf` | `.ofont` | MSDF atlas + glyph metrics + TTF bytes |
| `SoundFile` | `.ogg`, … | `.omusic` | Interleaved PCM samples |

#### Font Baking Pipeline (MSDF)

Fonts are the most complex asset type. When a TTF/OTF font is loaded for the
first time, the engine performs a multi-step baking process:

1. **Parse** — `ttf_parser::Face::parse()` reads the TrueType font data.
2. **Extract edges** — For each codepoint (default: ASCII 32..126),
   `extract_glyph_edges` converts the glyph outline into `Contour`s made of
   `EdgeSegment`s (line, quadratic bezier, cubic bezier).
3. **Bake MSDF** — `bake_msdf` rasterises each glyph into a **multi-channel
   signed distance field** — a 3-channel (RGB) texture where each channel stores
   signed distance classified by the nearest edge's normal direction. This
   preserves sharp corners at any scale.
4. **Pack atlas** — Individual glyph textures are packed into a 512×512 atlas
   grid. `GlyphMetrics` record the UV rect, size, bearing, and advance.
5. **Cache** — The assembled `FontFamilyFile` is saved as a `.ofont` binary
   containing atlas textures, glyph metric tables, font metrics, and the raw
   TTF source bytes (needed by rustybuzz for text shaping).

For **bitmap fonts**, `bake_sdf_from_bitmap` converts a binary sprite sheet
into a single-channel SDF atlas using distance transform.

#### GPU Upload

CPU-side asset types are uploaded to the GPU via the `GPU` renderer:

```ignore
let tex_file = TextureFile::from_disk("assets/grass.png")?;
let tex: Texture2D = gpu.upload_texture(&tex_file);

let cube = Mesh3DFile::cube(2.0);
let mesh: Mesh3D = gpu.upload_mesh3d(&cube);
```

The `GPU` loads fallback assets on construction — built-in shaders, a
checkerboard texture, and an 8×8 bitmap font — so rendering works immediately
without any custom assets.

#### Fallback Assets

| Asset | Source | Behaviour |
|---|---|---|
| 2D shader | `optic/assets/shdr/fallback2d.glsl` | Used by `upload_mesh2d` |
| 3D shader | `optic/assets/shdr/fallback3d.glsl` | Used by `upload_mesh3d` |
| Text shaders | `assets/shdr/fallback_text{2d,3d}.glsl` | Used by `Text2D`/`Text3D` |
| Texture | `optic/assets/txtr/fallback.png` | Checkerboard pattern |
| Font | Hardcoded 8×8 bitmap | ASCII 32–126, always available |

### ShaderFile

Derives:
- None

Implements:
- None

```rust
// No derives — contains shader source strings.
pub struct ShaderFile {
    pub v_src: String,
    pub f_src: String,
    pub is_compute: bool,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn from_src(src: &str, typ: ShaderType) -> OpticResult<Self>` | Parse a combined GLSL file (marker-based: `@vertex`/`@fragment`/`@compute`) |
| `pub fn from_vert_frag(v_src: &str, f_src: &str) -> Self` | Build from separate vertex and fragment source strings |
| `pub fn compile(&self) -> OpticResult<Shader>` | Compile shader and return a GPU handle |
| `pub fn from_disk(path: &str, typ: ShaderType) -> OpticResult<Self>` | Load from disk (debug: loads + overwrites cache; release: loads cache only) |
| `pub fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary cache (`.oshdr`) |
| `pub fn default_3d() -> OpticResult<Self>` | Get built-in 3D fallback shader |
| `pub fn default_2d() -> OpticResult<Self>` | Get built-in 2D fallback shader |

### ShaderType

Derives:
- None

Implements:
- None

```rust
// No derives — used as a parameter only.
pub enum ShaderType {
    Pipeline,  // vertex + fragment
    Compute,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn is_compute(&self) -> bool` | Is this a compute shader? |

### Mesh3DFile

Derives:
- None

Implements:
- None

```rust
// No derives — contains raw mesh data.
pub struct Mesh3DFile {
    pub pos_attr: Pos3DATTR,
    pub col_attr: ColorATTR,
    pub uvm_attr: UVMapATTR,
    pub nrm_attr: NormalATTR,
    pub ind_attr: IndicesATTR,
    pub custom_attrs: Vec<CustomATTR>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn empty() -> Self` | Empty mesh |
| `pub fn from_obj_src(src: &str) -> OpticResult<Self>` | Parse Wavefront OBJ (triangles only) |
| `pub fn from_stl_src(data: &[u8]) -> OpticResult<Self>` | Parse STL (ASCII or binary, triangles only) |
| `pub fn from_disk(path: &str) -> OpticResult<Self>` | Load from disk (debug: loads source + overwrites cache; release: loads cache only) |
| `pub fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary cache (`.omesh`) |
| `pub fn cube(side: f32) -> Self` | Generate a cube |
| `pub fn cuboid(w: f32, h: f32, d: f32) -> Self` | Generate a cuboid |
| `pub fn sphere(radius: f32, stacks: u32, sectors: u32) -> Self` | Generate a UV sphere |
| `pub fn cylinder(radius: f32, height: f32, segments: u32, cap: bool) -> Self` | Generate a cylinder |
| `pub fn cone(radius: f32, height: f32, segments: u32, cap: bool) -> Self` | Generate a cone |
| `pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> Self` | Generate a torus |
| `pub fn plane(width: f32, depth: f32) -> Self` | Generate a flat plane (XZ, Y=0) |
| `pub fn add_custom_attr(&mut self, attr: CustomATTR)` | Add a custom vertex attribute |
| `pub fn has_no_attr(&self) -> bool` | Check if mesh has no vertex data |
| `pub fn starts_with_custom(&self) -> bool` | Does the first attribute layout start with a custom attribute? |
| `pub fn upload(&self) -> MeshHandle` | Upload to GPU and get a handle |

### Mesh2DFile

Derives:
- None

Implements:
- None

```rust
// No derives — contains raw mesh data.
pub struct Mesh2DFile {
    pub pos_attr: Pos2DATTR,
    pub layer: u8,
    pub aspect: f32,
    pub col_attr: ColorATTR,
    pub uvm_attr: UVMapATTR,
    pub ind_attr: IndicesATTR,
    pub custom_attrs: Vec<CustomATTR>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn empty() -> Self` | Empty mesh |
| `pub fn set_pos_attr(&mut self, attr: Pos2DATTR)` | Set positions |
| `pub fn set_layer(&mut self, layer: u8)` | Set Z layer |
| `pub fn set_center(&mut self, center: Center)` | Recenter around a pivot |
| `pub fn set_col_attr(&mut self, attr: ColorATTR)` | Set vertex colors |
| `pub fn set_uvm_attr(&mut self, attr: UVMapATTR)` | Set UV coordinates |
| `pub fn set_ind_attr(&mut self, attr: IndicesATTR)` | Set indices |
| `pub fn quad(size: &Size2D) -> Self` | Generate a textured quad |
| `pub fn fullscreen_quad() -> Self` | Fullscreen quad (positions [-1,1]) |
| `pub fn circle(radius: f32, segments: u32) -> Self` | Generate a filled circle |
| `pub fn polygon(radius: f32, sides: u32) -> Self` | Generate a regular polygon |
| `pub fn ring(inner_radius: f32, outer_radius: f32, segments: u32) -> Self` | Generate a ring |
| `pub fn rect(width: f32, height: f32) -> Self` | Generate a rectangle |
| `pub fn add_custom_attr(&mut self, attr: CustomATTR)` | Add custom attribute |
| `pub fn starts_with_custom(&self) -> bool` | Does the first layout start with a custom attribute? |
| `pub fn upload(&self) -> MeshHandle` | Upload to GPU and get a handle |

### TextureFile

Derives:
- None

Implements:
- None

```rust
// No derives — contains raw image data.
pub struct TextureFile {
    pub bytes: Vec<u8>,
    pub size: Size2D,
    pub fmt: ImgFormat,
    pub filter: ImgFilter,
    pub wrap: ImgWrap,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn pixel_count(&self) -> usize` | Total pixels |
| `pub fn set_wrap(&mut self, wrap: ImgWrap)` | Set wrap mode |
| `pub fn set_filter(&mut self, filter: ImgFilter)` | Set filter mode |
| `pub fn upload(&self) -> Texture2D` | Upload to GPU |
| `pub fn fallback() -> OpticResult<Self>` | Create checkerboard fallback texture |
| `pub fn from_disk(path: &str) -> OpticResult<Self>` | Load from disk (debug: loads source + overwrites cache; release: loads cache only) |
| `pub fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary cache (`.otxtr`) |

### Attribute Types

#### Pos3DATTR

Derives:
- Debug
- Clone

Implements:
- None

#### Pos2DATTR

Derives:
- Debug
- Clone

Implements:
- None

#### ColorATTR

Derives:
- Debug
- Clone

Implements:
- None

#### UVMapATTR

Derives:
- Debug
- Clone

Implements:
- None

#### NormalATTR

Derives:
- Debug
- Clone

Implements:
- None

#### Rot2DATTR

Derives:
- Debug
- Clone

Implements:
- None

#### Rot3DATTR

Derives:
- Debug
- Clone

Implements:
- None

#### Scale2DATTR

Derives:
- Debug
- Clone

Implements:
- None

#### Scale3DATTR

Derives:
- Debug
- Clone

Implements:
- None

#### IndicesATTR

Derives:
- Debug
- Clone

Implements:
- None

#### CustomATTR

Derives:
- Debug

Implements:
- None

```rust
#[derive(Debug, Clone)]
pub struct Pos3DATTR   { pub data: Vec<[f32; 3]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct Pos2DATTR   { pub data: Vec<[f32; 2]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct ColorATTR     { pub data: Vec<[f32; 4]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct UVMapATTR     { pub data: Vec<[f32; 2]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct NormalATTR     { pub data: Vec<[f32; 3]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct Rot2DATTR   { pub data: Vec<f32>,       pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct Rot3DATTR   { pub data: Vec<[f32; 4]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct Scale2DATTR { pub data: Vec<[f32; 2]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct Scale3DATTR { pub data: Vec<[f32; 3]>, pub info: ATTRInfo }
#[derive(Debug, Clone)]
pub struct IndicesATTR     { pub data: Vec<u32>,      pub info: ATTRInfo }
#[derive(Debug)]
pub struct CustomATTR {
    pub data: Vec<u8>,
    pub info: ATTRInfo,
}
```

All `*ATTR` structs share these methods via macro:

| Signature | Description |
|-----------|-------------|
| `pub fn empty() -> Self` | Empty attribute |
| `pub fn new(vec: Vec<T>) -> Self` | From a Vec |
| `pub fn from_array(array: &[T]) -> Self` | From a slice |
| `pub fn push(&mut self, elem: T)` | Add one element |
| `pub fn is_empty(&self) -> bool` | No data? |

`CustomATTR` additionally has generic methods:

| Signature | Description |
|-----------|-------------|
| `pub fn empty<D: DataType>(name: &str) -> Self` | Empty custom attribute with name |
| `pub fn new<D: DataType>(name: &str, vec: Vec<D>) -> Self` | From typed Vec |
| `pub fn from_array<D: DataType + Clone>(name: &str, array: &[D]) -> Self` | From typed slice |
| `pub fn push<D: DataType>(&mut self, elem: D)` | Push a typed element |

#### ATTRInfo

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct ATTRInfo {
    pub name: ATTRName,
    pub typ: ATTRType,
    pub byte_count: usize,
    pub elem_count: usize,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new() -> Self` | Zeroed info |
| `pub fn fmt_as_string(&self) -> String` | Formatted as `"{name}:{typ}:{byte_count}:{elem_count}"` |

#### ATTRName

Derives:
- Clone
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ATTRName {
    Custom(String),
    Pos2D, Pos3D, Color, UVMap, Normal, Indices,
    Rot2D, Rot3D, Scale2D, Scale3D,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn as_string(&self) -> String` | Name as string |

### DataType Trait

```rust
pub trait DataType {
    const ATTR_FORMAT: ATTRType;
    const BYTE_COUNT: usize;
    const ELEM_COUNT: usize;
    fn u8ify(&self) -> Vec<u8>;
    fn from_bytes(bytes: &[u8]) -> Self;
}
```

`from_bytes` reconstitutes a value from a raw byte slice — the inverse of `u8ify`.
Implemented for: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `f32`, `f64` (scalar) and
`[T; 2]`, `[T; 3]`, `[T; 4]` for each.

### Dirty

Derives:
- None

Implements:
- DataType

```rust
// No derives — generic over T.
pub struct Dirty<T> {
    value: T,
    dirty: bool,
}
impl<T: DataType> DataType for Dirty<T> {}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(value: T) -> Self` | Create with initial value (dirty) |
| `pub fn value(&self) -> &T` | Get reference to value |
| `pub fn set(&mut self, value: T)` | Set value and mark dirty |
| `pub fn is_dirty(&self) -> bool` | Has been set since last `clear_dirty`? |
| `pub fn clear_dirty(&mut self)` | Clear dirty flag |

`Dirty<T: DataType>` implements `DataType`, so it can be stored directly in
`CustomATTR` and uploaded as a per-instance buffer attribute. The `dirty` flag
enables incremental GPU updates — only re-upload attributes that have changed.

Previously named `Signal<T>` in the now-removed `optic_signals` crate.

### Center

Derives:
- None

Implements:
- None

```rust
// No derives — pivot point for 2D mesh generation.
pub enum Center {
    TopLeft, TopRight, BottomLeft, BottomRight,
    Middle, Custom(f32, f32),
}
```

---

## 6. Text Rendering (`optic_render::text`)

Text rendering uses BBCode markup, MSDF atlas fonts, and instanced quad drawing.

### FontFamilyFile

Asset-level font family — metrics, style variants, and optional TTF source bytes.

Derives:
- None

Implements:
- None

```rust
// No derives — contains baked font data and optional TTF bytes.
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
```

| Signature | Description |
|-----------|-------------|
| `pub fn from_ttf_file(regular_bytes: &[u8], codepoint_range: (u32, u32), atlas_resolution: u32) -> OpticResult<Self>` | Bake a TTF font into an MSDF atlas |
| `pub fn with_bold(self, bytes: &[u8]) -> OpticResult<Self>` | Add bold variant from TTF bytes |
| `pub fn with_italic(self, bytes: &[u8]) -> OpticResult<Self>` | Add italic variant from TTF bytes |
| `pub fn with_bold_italic(self, bytes: &[u8]) -> OpticResult<Self>` | Add bold-italic variant from TTF bytes |
| `pub fn from_bitmap_layout(layout: BitmapFontLayout) -> OpticResult<Self>` | Construct from a bitmap font layout |
| `pub fn from_disk(path: &str) -> OpticResult<Self>` | Load from disk (auto-caches to `.ofont`) |
| `pub fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary `.ofont` cache |
| `pub fn from_cached(path: &str) -> OpticResult<Self>` | Load from binary `.ofont` cache |
| `pub fn fallback() -> OpticResult<Self>` | Built-in 8×8 bitmap fallback font (ASCII 32..126) |
| `pub fn units_per_em(&self) -> f32` | `line_height` for bitmap, `1.0` for TTF |

### BakedFont

Single baked font style — atlas texture + glyph metrics.

Derives:
- None

Implements:
- None

```rust
// No derives — contains GPU-ready texture data.
pub struct BakedFont {
    pub atlas: TextureFile,
    pub glyphs: HashMap<u32, GlyphMetrics>,
    pub edge_softness: f32,
}
```

### GlyphMetrics

Per-glyph atlas lookup data.

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct GlyphMetrics {
    pub uv_rect: (f32, f32, f32, f32),  // [u0, v0, u1, v1]
    pub size: Size2D,
    pub bearing: (f32, f32),
    pub advance: f32,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn zero() -> Self` | Zero-initialized |
| `pub fn size_arr(&self) -> [f32; 2]` | Size as `[w, h]` |
| `pub fn bearing_arr(&self) -> [f32; 2]` | Bearing as `[x, y]` |

### BitmapFontLayout

Describes a bitmap font tile grid for `FontFamilyFile::from_bitmap_layout`.

Derives:
- None

Implements:
- None

```rust
// No derives — contains texture data.
pub struct BitmapFontLayout {
    pub texture: TextureFile,
    pub glyph_size: Size2D,
    pub columns: u32,
    pub codepoint_order: Vec<u32>,
    pub advance: Option<u32>,
}
```

### FontFamily (GPU)

GPU-uploaded font family — one atlas per style variant.

Derives:
- Clone
- Debug

Implements:
- None

```rust
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
```

| Signature | Description |
|-----------|-------------|
| `pub fn fallback_bitmap() -> OpticResult<Self>` | Built-in 8×8 bitmap font |
| `pub fn line_height(&self) -> f32` | Line height in font units or pixels |
| `pub fn ascent(&self) -> f32` | Ascent above baseline |
| `pub fn descent(&self) -> f32` | Descent below baseline (negative) |
| `pub fn is_bitmap(&self) -> bool` | Bitmap font? |
| `pub fn units_per_em(&self) -> f32` | `line_height` (bitmap) or `1.0` (TTF) |
| `pub fn face_data(&self) -> Option<&[u8]>` | Raw TTF bytes for rustybuzz |
| `pub fn atlas(&self, style: FontStyle) -> &Texture2D` | Atlas for given style |
| `pub fn primary_atlas(&self) -> &Texture2D` | Regular atlas |
| `pub fn edge_softness(&self, style: FontStyle) -> f32` | MSDF edge softness |
| `pub fn glyph(&self, style: FontStyle, gid: u32) -> Option<&GlyphMetrics>` | Look up glyph |
| `pub fn has_style(&self, style: FontStyle) -> bool` | Has dedicated atlas? |
| `pub fn resolve_style(&self, bold: bool, italic: bool) -> (FontStyle, bool, bool)` | Best style + faux flags |
| `pub fn delete(self)` | Free GPU textures |

### FontStyle

Derives:
- Clone
- Copy
- Debug
- PartialEq
- Eq
- Hash

Implements:
- None

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum FontStyle {
    Regular,
    Bold,
    Italic,
    BoldItalic,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn with_bold(self, bold: bool) -> Self` | Toggle bold |
| `pub fn with_italic(self, italic: bool) -> Self` | Toggle italic |

### BBCode Types

```rust
pub const FAUX_BOLD: u32;     // 1 << 0
pub const FAUX_ITALIC: u32;   // 1 << 1
pub const BORDER: u32;        // 1 << 2
```

TextStyle:

Derives:
- Clone
- Debug
- Default

Implements:
- None

```rust
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
```

WaveEffect:

Derives:
- Clone
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct WaveEffect {
    pub amp: f32,
    pub freq: f32,
    pub speed: f32,
}```

ShakeEffect:

Derives:
- Clone
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ShakeEffect {
    pub amp: f32,
    pub speed: f32,
}```

RainbowEffect:

Derives:
- Clone
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct RainbowEffect {
    pub speed: f32,
}```

PulseEffect:

Derives:
- Clone
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct PulseEffect {
    pub amp: f32,
    pub speed: f32,
}
```

StyledSpan:

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct StyledSpan {
    pub text: String,
    pub style: TextStyle,
}```

ParsedText:

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct ParsedText {
    pub spans: Vec<StyledSpan>,
    pub is_dynamic: bool,
}
```

| Function | Description |
|----------|-------------|
| `pub fn parse(raw: &str) -> OpticResult<ParsedText>` | Parse BBCode string |
| `pub fn detect_dynamic(raw: &str) -> bool` | Has dynamic effects? |
| `pub fn rainbow_color(effect, time, index, alpha) -> RGBA` | Compute rainbow color |

**Supported tags:** `[b]`, `[i]`, `[color=#rrggbbaa]`, `[bgcolor=#rrggbbaa]`, `[size=N]`, `[s]`, `[u]`, `[border=#rrggbbaa,W]`, `[kerning=N]`, `[offset=x,y]`, `[wave amp=,freq=,speed=]`, `[shake amp=,speed=]`, `[rainbow speed=]`, `[pulse amp=,speed=]`

### Layout Types

ShapedGlyph:

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct ShapedGlyph {
    pub gid: u32,
    pub cluster_start: usize,
    pub x_offset: f32,
    pub y_offset: f32,
    pub x_advance: f32,
}
```

LayoutGlyph:

Derives:
- Clone
- Debug

Implements:
- None

```rust
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
```

LayoutDecoration:

Derives:
- Clone
- Debug

Implements:
- None

```rust
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
```

DecorationKind:

Derives:
- Clone
- Copy
- Debug
- PartialEq

Implements:
- None

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DecorationKind {
    Background,
    Underline,
    Strikethrough,
}
```

TextLayout:

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub struct TextLayout {
    pub parsed: ParsedText,
    pub glyphs: Vec<LayoutGlyph>,
    pub decorations: Vec<LayoutDecoration>,
    pub width: f32,
    pub height: f32,
    pub is_dynamic: bool,
}
```

| Function | Description |
|----------|-------------|
| `pub fn layout_text(raw, font, base_size, wrap_width) -> OpticResult<TextLayout>` | Layout BBCode text |
| `pub fn shape_span(span, font) -> Vec<ShapedGlyph>` | Shape a single span |
| `pub fn build_glyph_desc_2d(layout, time) -> InstanceDesc2D` | 2D glyph instances |
| `pub fn build_glyph_desc_3d(layout, time) -> InstanceDesc3D` | 3D glyph instances |
| `pub fn build_decoration_desc_2d(layout, time) -> InstanceDesc2D` | 2D decoration instances |
| `pub fn build_decoration_desc_3d(layout, time) -> InstanceDesc3D` | 3D decoration instances |

### Text2D

Screen-space (HUD / UI) text rendered with instanced quads.

Derives:
- None

Implements:
- None

```rust
// No derives — owns GPU instance buffers and layout state.
pub struct Text2D {
    raw_text: String,
    font: FontFamily,
    shader: Option<Shader>,
    base_size: f32,
    wrap_width: Option<f32>,
    transform: Transform2D,
    quad_mesh: MeshHandle,
    glyph_instances: Option<InstanceBuffer>,
    decoration_instances: Option<InstanceBuffer>,
    layout: Option<TextLayout>,
    is_dynamic: bool,
    time: f32,
    visibility: bool,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(font: FontFamily) -> Self` | Create with font |
| `pub fn set_text(&mut self, text: &str) -> OpticResult<()>` | Set BBCode text, rebuild layout |
| `pub fn text(&self) -> &str` | Get raw text |
| `pub fn set_font(&mut self, font: FontFamily) -> OpticResult<()>` | Replace font |
| `pub fn set_shader(&mut self, shader: Shader)` | Assign shader |
| `pub fn remove_shader(&mut self)` | Remove shader |
| `pub fn shader(&self) -> Option<&Shader>` | Current shader |
| `pub fn set_base_size(&mut self, size: f32) -> OpticResult<()>` | Set base font size |
| `pub fn set_wrap_width(&mut self, width: Option<f32>) -> OpticResult<()>` | Set wrap width (`None` = no wrap) |
| `pub fn transform(&self) -> &Transform2D` | 2D transform |
| `pub fn transform_mut(&mut self) -> &mut Transform2D` | Mutable 2D transform |
| `pub fn update(&mut self, time: f32) -> OpticResult<()>` | Update dynamic effects |
| `pub fn is_dynamic(&self) -> bool` | Has dynamic effects? |
| `pub fn set_visibility(&mut self, visible: bool)` | Show/hide |
| `pub fn is_visible(&self) -> bool` | Is visible? |
| `pub fn render(&mut self, proj: &Matrix4<f32>)` | Render with projection matrix |
| `pub fn delete(self)` | Free GPU resources |

### Text3D

World-space billboard text. Same API as Text2D, plus a 3D transform.

Derives:
- None

Implements:
- None

```rust
// No derives — owns GPU instance buffers and layout state.
pub struct Text3D {
    raw_text: String,
    font: FontFamily,
    shader: Option<Shader>,
    base_size: f32,
    wrap_width: Option<f32>,
    transform: Transform3D,
    quad_mesh: MeshHandle,
    glyph_instances: Option<InstanceBuffer>,
    decoration_instances: Option<InstanceBuffer>,
    layout: Option<TextLayout>,
    is_dynamic: bool,
    time: f32,
    visibility: bool,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(font: FontFamily) -> Self` | Create with font |
| `pub fn set_text(&mut self, text: &str) -> OpticResult<()>` | Set BBCode text |
| `pub fn text(&self) -> &str` | Get raw text |
| `pub fn set_font(&mut self, font: FontFamily) -> OpticResult<()>` | Replace font |
| `pub fn set_shader(&mut self, shader: Shader)` | Assign shader |
| `pub fn remove_shader(&mut self)` | Remove shader |
| `pub fn shader(&self) -> Option<&Shader>` | Current shader |
| `pub fn set_base_size(&mut self, size: f32) -> OpticResult<()>` | Set base font size |
| `pub fn set_wrap_width(&mut self, width: Option<f32>) -> OpticResult<()>` | Set wrap width (`None` = no wrap) |
| `pub fn transform(&self) -> &Transform3D` | 3D transform |
| `pub fn transform_mut(&mut self) -> &mut Transform3D` | Mutable 3D transform |
| `pub fn update(&mut self, time: f32) -> OpticResult<()>` | Update dynamic effects |
| `pub fn is_dynamic(&self) -> bool` | Has dynamic effects? |
| `pub fn set_visibility(&mut self, visible: bool)` | Show/hide |
| `pub fn is_visible(&self) -> bool` | Is visible? |
| `pub fn render(&mut self, view: &Matrix4<f32>, proj: &Matrix4<f32>)` | Render with view+projection |
| `pub fn delete(self)` | Free GPU resources |

---

## 7. Game Loop (`optic_loop`)

The engine runs a three-phase frame: **Physics → Update → Render**. Each phase runs at its own independently configurable rate.

```
┌──────────────────────────────────────────────┐
│  Frame                                       │
│                                              │
│  Physics (fixed timestep, default 60 Hz)     │
│    ├ physics()                               │
│    ├ physics()     ← catch-up if slow frame  │
│    └ physics()                               │
│                                              │
│  Update (optional fixed timestep)            │
│    └ update()       ← once per frame default │
│                                              │
│  Render (presented once per frame)           │
│    └ render()                                │
│                                              │
│  FPS limiter (VSync / Limited / Uncapped)    │
└──────────────────────────────────────────────┘
```

### Runtime Trait

```rust
pub trait Runtime {
    fn start(&mut self, game: &mut Game);
    fn physics(&mut self, game: &mut Game) {}
    fn update(&mut self, game: &mut Game);
    fn render(&mut self, game: &mut Game) {}
    fn end(&mut self, game: &mut Game);
}
```

| Method | Called | Purpose |
|--------|--------|---------|
| `start` | Once, before first frame | One-time setup, configure rates |
| `physics` | 0+ times per frame | Fixed-timestep simulation (integration, collision) |
| `update` | According to TPS | Input, AI, gameplay logic |
| `render` | Exactly once per frame | Draw calls, use `physics_alpha()` for interpolation |
| `end` | Once, on shutdown | Cleanup, save state |

`physics()` and `render()` have default empty implementations. Existing code that only implements `start`, `update`, and `end` compiles unchanged (though draw calls in `update()` must be moved to `render()` for visibility).

### FpsLimit

Rendering frame-rate policy.

Derives:
- None

Implements:
- Default

```rust
// No derives — defines frame-rate policy.
pub enum FpsLimit {
    Uncapped,
    VSync,
    Limited(f64),
}
impl Default for FpsLimit { /* VSync */ }
```

| Variant | Behaviour |
|---------|-----------|
| `Uncapped` | Render as fast as the platform allows |
| `VSync` | Swap interval determines pacing |
| `Limited(fps)` | Sleep to approximate the given frame rate |

### Game

Derives:
- None

Implements:
- None

```rust
// No derives — owns engine subsystems.
pub struct Game {
    pub renderer: GPU,
    pub camera: Camera,
    pub events: Events,
    pub time: Time,
    pub window: Window,
    pub audio: AudioEngine,
    event_loop: Option<EventLoop<()>>,
    surface_index: usize,
    gilrs: Gilrs,
    runtime: Option<Box<dyn Runtime>>,
    running: bool,
    started: bool,
    requested_size: Size2D,
    resized_once: bool,
    #[cfg(feature = "online")]
    pub(crate) network: Option<NetworkHandle>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new<R: Runtime + 'static>(runtime: R) -> OpticResult<Game>` | Initialize everything |
| `pub fn run<R: Runtime + 'static>(runtime: R)` | Create and run (convenience) |
| `pub fn exit(&mut self)` | Request exit on next frame |
| `#[cfg(feature = "online")] pub fn enable_networking(&mut self, config: NetworkConfig) -> OpticResult<()>` | Start networking |

### Time

Derives:
- None

Implements:
- None

```rust
// No derives — owns timing state.
pub struct Time {
    pub fps: f64,
    pub delta: f64,
    pub tick_count: u64,
    pub elapsed: f64,
    pub start_time: Instant,
    pub prev_time: Instant,
    pub prev_sec: Instant,
    pub local_tick: u32,
    prev_deltas: VecDeque<f64>,
    physics_stepper: FixedStepper,
    update_stepper: FixedStepper,
    fps_limit: FpsLimit,
    physics_delta: f64,
    physics_alpha: f32,
    frame_start: Instant,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new() -> Self` | Create (physics 60 Hz, update once/frame, VSync) |
| `pub fn update(&mut self)` | Advance time (called automatically) |
| **Physics rate** | |
| `pub fn set_target_physics_rate(&mut self, hz: f64)` | Set physics Hz (default 60) |
| `pub fn target_physics_rate(&self) -> f64` | Current physics Hz |
| `pub fn physics_delta(&self) -> f64` | Fixed dt (`1/hz`), constant across all callbacks in a frame |
| `pub fn physics_alpha(&self) -> f32` | Interpolation alpha `[0, 1)` for rendering |
| **Update rate** | |
| `pub fn set_target_tps(&mut self, hz: Option<f64>)` | `None` = once per frame (default), `Some(hz)` = fixed |
| `pub fn target_tps(&self) -> Option<f64>` | Current target TPS |
| **FPS limit** | |
| `pub fn set_fps_limit(&mut self, limit: FpsLimit)` | Set rendering frame-rate policy |
| `pub fn fps_limit(&self) -> &FpsLimit` | Current FPS limit |
| **Timing** | |
| `pub fn fps(&self) -> f64` | Smoothed FPS (32-frame average) |
| `pub fn delta(&self) -> f64` | Seconds since last frame |
| `pub fn elapsed(&self) -> f64` | Seconds since `Time::new` |
| `pub fn now(&self) -> f64` | Re-query wall-clock seconds since start |
| `pub fn now_ms(&self) -> u64` | Milliseconds since start |
| `pub fn now_as_ms(&self) -> u64` | Alias for `now_ms` |
| `pub fn now_as_ns(&self) -> u64` | Nanoseconds since start |
| `pub fn sleep(&self, secs: f64)` | Sleep for seconds |
| `pub fn sleep_ms(&self, millis: u64)` | Sleep for milliseconds |
| `pub fn sleep_ns(&self, nanos: u64)` | Sleep for nanoseconds |

**Interpolation:** When physics runs slower than rendering, use `physics_alpha()` to lerp between previous and current simulation state in `render()`. The engine performs no automatic interpolation.

**Rate changes:** Taking effect immediately — the next scheduler invocation observes the new rate. Existing accumulator contents are preserved.

**Spiral-of-death:** If a frame requires more than 240 physics/update steps, excess is discarded with phase preservation and a warning is logged.

### Timer

A countdown timer polled explicitly each frame.

Derives:
- None

Implements:
- None

```rust
// No derives — mutable countdown state.
pub struct Timer {
    wait_time: f64,    // (private)
    time_left: f64,    // (private)
    repeating: bool,   // (private)
    active: bool,      // (private)
    paused: bool,      // (private)
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(wait_time: f64) -> Self` | One-shot timer that fires after `wait_time` seconds |
| `pub fn new_repeating(wait_time: f64) -> Self` | Repeating timer that fires every `wait_time` seconds |
| `pub fn tick(&mut self, dt: f64) -> bool` | Advance by `dt` seconds; returns `true` on completion |
| `pub fn reduce(&mut self, amount: f64) -> bool` | Reduce remaining time; returns `true` if completed |
| `pub fn extend(&mut self, amount: f64)` | Add seconds to remaining time; re-activates if finished |
| `pub fn reset(&mut self)` | Reset to initial state (full wait time, un-paused, active) |
| `pub fn tick_and_emit(&mut self, dt: f64, name: &str, events: &mut Events) -> bool` | Tick + emit a named custom event on completion |
| `pub fn pause(&mut self)` | Pause the timer (retains remaining time) |
| `pub fn resume(&mut self)` | Resume from a paused state |
| `pub fn is_running(&self) -> bool` | `true` if actively counting down (not paused, not finished) |
| `pub fn is_paused(&self) -> bool` | `true` if paused |
| `pub fn is_active(&self) -> bool` | `true` if not yet finished (paused timers are still active) |
| `pub fn start(&mut self)` | Un-pause and activate |
| `pub fn stop(&mut self)` | Pause the timer |
| `pub fn is_looping(&self) -> bool` | `true` if timer repeats after each completion |
| `pub fn set_looping(&mut self, repeating: bool)` | Set whether the timer repeats |
| `pub fn time_left(&self) -> f64` | Remaining time in seconds |
| `pub fn elapsed(&self) -> f64` | Elapsed time (wait_time - time_left) in seconds |
| `pub fn progress(&self) -> f64` | Completion progress as `0.0..=1.0` |
| `pub fn wait_time(&self) -> f64` | Total wait time |
| `pub fn set_wait_time(&mut self, wait_time: f64)` | Set new wait time and reset remaining |

All time values are `f64` to match `Time::delta()`. Repeating timers auto-reset
on completion. Use `tick_and_emit` to bridge with the custom event system.

### Timers

A collection of timers managed as a group.

Derives:
- None

Implements:
- None

```rust
// No derives — owns timer instances.
pub struct Timers {
    timers: Vec<Timer>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new() -> Self` | Empty collection |
| `pub fn add(&mut self, timer: Timer)` | Add a timer |
| `pub fn remove(&mut self, index: usize) -> OpticResult<()>` | Remove timer at index |
| `pub fn clear(&mut self)` | Remove all timers |
| `pub fn len(&self) -> usize` | Number of timers |
| `pub fn is_empty(&self) -> bool` | Empty? |
| `pub fn get(&self, index: usize) -> Option<&Timer>` | Get timer by index |
| `pub fn get_mut(&mut self, index: usize) -> Option<&mut Timer>` | Get mutable timer by index |
| `pub fn tick_all(&mut self, dt: f64) -> Vec<usize>` | Tick all active timers; returns indices that elapsed |
| `pub fn tick_and_emit_all(&mut self, dt: f64, name: &str, events: &mut Events)` | Tick all and emit named events |
| `pub fn iter(&self) -> impl Iterator<Item = &Timer>` | Iterator over all timers |
| `pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Timer>` | Mutable iterator over all timers |

### FrameState

Derives:
- None

Implements:
- None

```rust
// No derives — temporary borrow bundle passed to closures.
pub struct FrameState<'a> {
    pub time: &'a Time,
    pub windows: &'a mut [WindowState],
    pub gpu: &'a mut GPU,
    pub camera: &'a mut Camera,
}
```

### WindowState

Derives:
- None

Implements:
- None

```rust
// No derives — owns window and event state.
pub struct WindowState {
    pub window: Window,
    pub events: Events,
    pub surface_index: usize,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(el: &EventLoop<()>, title: &str, size: Size2D) -> Self` | Create window state |
| `pub fn close(&mut self)` | Close the window |
| `pub fn is_closed(&self) -> bool` | Check closed |
| `pub fn surface_index(&self) -> usize` | Surface index for renderer |

### GameLoop

Low-level game loop that drives a closure once per frame. FPS limiting applies via `time.fps_limit()`.

Derives:
- None

Implements:
- None

```rust
// No derives — owns event loop and engine subsystems.
pub struct GameLoop<F: FnMut(&mut FrameState)> {
    event_loop: Option<EventLoop<()>>,
    windows: Vec<WindowState>,
    gpu: Option<GPU>,
    camera: Camera,
    time: Time,
    gilrs: Gilrs,
    frame_fn: F,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(el, gpu, camera, windows: Vec<WindowState>, frame_fn: F) -> OpticResult<Self>` | Create multi-window game loop |
| `pub fn run(mut self)` | Start the event loop |

### Standalone `run()`

```rust
pub fn run<F>(title: &str, size: Size2D, frame_fn: F)
where
    F: FnMut(&mut FrameState) + 'static;
```

Convenience function for single-window applications. Errors are logged via `log_error!` and the process exits with code 1 on failure.

---

## 8. File Utilities (`optic_file`)

All functions are free functions in the `optic_file` crate:

| Signature | Description |
|-----------|-------------|
| `pub fn name(path: &str) -> Option<String>` | File stem (name without extension) |
| `pub fn extension(path: &str) -> Option<String>` | File extension |
| `pub fn exists(path: &str) -> bool` | Check if path exists |
| `pub fn read_bytes(path: &str) -> OpticResult<Vec<u8>>` | Read file as bytes |
| `pub fn read_string(path: &str) -> OpticResult<String>` | Read file as UTF-8 string |
| `pub fn write_bytes(path: &str, data: &[u8]) -> OpticResult<()>` | Write bytes (creates directories) |
| `pub fn write_string(path: &str, data: &str) -> OpticResult<()>` | Write string |
| `pub fn cached_path(source: &str, ext: &str) -> String` | Generate cache path: `{dir}/optc/{stem}.{ext}` |
| `pub fn create_dir(path: &str) -> OpticResult<()>` | Create directory recursively |

---

## 9. Networking (`optic_online`)

Enables optional dedicated-server-style UDP multiplayer via the `online` feature on the `optic`
facade crate (`optic = { features = ["online"] }`). The `NetworkEvents` field on `Events` is always
present (zero cost — always-empty vectors when networking is off).

Usage from a `Runtime` implementation:

```rust
use optic::*;

// At some point before the first frame:
game.enable_networking(NetworkConfig {
    mode: NetworkMode::Host { port: 7777 },
    max_peers: 16,
})?;

// Each frame, game.events.network contains:
for (peer_id, data) in &game.events.network.packets {
    // handle incoming data
}
for peer_id in &game.events.network.peers_connected { /* ... */ }
for peer_id in &game.events.network.peers_disconnected { /* ... */ }

// Send data:
game.network().unwrap().send_all(b"hello");
```

### NetworkConfig

Derives:
- Clone
- Debug

Implements:
- Default

```rust
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub mode: NetworkMode,
    pub max_peers: u32,
}
```

### NetworkMode

Derives:
- Clone
- Debug

Implements:
- None

```rust
#[derive(Clone, Debug)]
pub enum NetworkMode {
    Host { port: u16 },
    Client { addr: String },
}
```

- `Host` — binds a UDP socket on all interfaces at the given port. Use `0` for OS-assigned port.
- `Client` — connects to the host at `IP:port`. UDP is connectionless, so the client is immediately
  usable (receives `Connected` event right away).

### PeerId

Derives:
- Copy
- Clone
- Debug
- PartialEq
- Eq
- Hash

Implements:
- None

```rust
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerId(pub u64);
```

| Constant | Description |
|----------|-------------|
| `PeerId::SERVER` | `PeerId(0)` — sentinel for server in client mode |

### NetworkEvents

Derives:
- Clone
- Debug
- Default

Implements:
- None

```rust
#[derive(Clone, Debug, Default)]
pub struct NetworkEvents {
    pub peers_connected: Vec<PeerId>,
    pub peers_disconnected: Vec<PeerId>,
    pub packets: Vec<(PeerId, Vec<u8>)>,
}
```

Cleared at the end of each frame in `Events::end_frame()`. Poll the `NetworkHandle` once per frame
to populate this struct — the game loop does this automatically when networking is enabled.

### NetworkHandle

Derives:
- None

Implements:
- None

```rust
// No derives — owns network thread and channels.
pub struct NetworkHandle {
    thread: Option<JoinHandle<()>>,
    inbound_data_rx: tokio_mpsc::UnboundedReceiver<(PeerId, Vec<u8>)>,
    lifecycle_rx: tokio_mpsc::UnboundedReceiver<LifecycleEvent>,
    outbound_tx: tokio_mpsc::UnboundedSender<TransportCommand>,
    local_addr: Option<SocketAddr>,
    shutdown_flag: Arc<AtomicBool>,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new(config: NetworkConfig) -> OpticResult<Self>` | Spawn network thread and bind socket (blocks until bound) |
| `pub fn poll(&mut self, out: &mut NetworkEvents)` | Drain all queued events (non-blocking — call once per frame) |
| `pub fn send(&self, peer: PeerId, bytes: &[u8]) -> OpticResult<()>` | Send to a specific peer |
| `pub fn send_all(&self, bytes: &[u8]) -> OpticResult<()>` | Broadcast to all connected peers |
| `pub fn send_all_except(&self, exclude: PeerId, bytes: &[u8]) -> OpticResult<()>` | Broadcast to all except one |
| `pub fn disconnect(&self, peer: PeerId)` | Kick a peer (host) or disconnect from server (client) |
| `pub fn peers(&self) -> Vec<PeerId>` | Snapshot of known peer IDs (best-effort) |
| `pub fn local_addr(&self) -> Option<SocketAddr>` | Bound local socket address |
| `pub fn shutdown(&mut self)` | Graceful shutdown (waits for network thread) |
| `pub fn is_shutdown(&self) -> bool` | Has the network thread exited? |

**Notes:**
- All send methods are non-blocking (`try_send` on unbounded channel). If the outbound channel is
  closed (thread exited), an error is returned.
- `poll()` uses `try_recv` loops — never blocks, returns in microseconds.
- Host mode sends 0-byte heartbeats every 2 seconds; peers are considered stale after 10 seconds
  of silence.
- Client mode has a 5-second connection timeout for the first packet; if no data arrives from the
  server in that window, a `Disconnected(PeerId::SERVER)` lifecycle event fires. Once connected,
  the client stays connected until explicitly disconnected or the socket errors.
- Reachability (`IP:port` being reachable) is entirely the operator's responsibility — no STUN,
   UPnP, or NAT traversal is attempted. Use direct connections, port forwarding, VPS, or
   ZeroTier/Tailscale.

---

## 10. Sound (`optic_sound`)

Enabled via the `sound` feature on the `optic` facade crate
(`optic = { features = ["sound"] }`). `Game` always has a `pub audio: AudioEngine`
field (no feature gate — always compiled).

### AudioEngine

Derives:
- None

Implements:
- None

```rust
// No derives — owns Kira audio manager.
pub struct AudioEngine {
    manager: AudioManager<DefaultBackend>,
    listener: ListenerHandle,
    master_volume: f32,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn new() -> OpticResult<Self>` | Create the audio engine and spawn the audio render thread |
| `pub fn upload_sound2d(&mut self, file: &SoundFile) -> OpticResult<Sound2D>` | Upload a sound file as a 2D sound handle |
| `pub fn upload_sound3d(&mut self, file: &SoundFile) -> OpticResult<Sound3D>` | Upload a sound file as a 3D spatial sound handle |
| `pub fn set_master_volume(&mut self, volume: f32)` | Set master volume (0.0..1.0) |
| `pub fn set_listener(&mut self, pos: Vector3<f32>, forward: Vector3<f32>, up: Vector3<f32>)` | Set 3D listener position/orientation |
| `pub fn set_listener_from_camera(&mut self, camera: &Camera)` | Position the 3D listener from the active camera |

### SoundFile

A decoded sound file loaded from disk or cache.

Derives:
- None

Implements:
- None

```rust
// No derives — contains decoded audio samples.
pub struct SoundFile {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u8,
    pub duration_secs: f32,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn from_disk(path: &str) -> OpticResult<Self>` | Load from disk (debug: decodes + writes `.omusic` cache; release: loads cache only) |
| `pub fn from_cached(path: &str) -> OpticResult<Self>` | Load from binary cache only |
| `pub fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary cache (`.omusic`) |

### Sound2D

A handle to a playing 2D sound.

Derives:
- None

Implements:
- None

```rust
// No derives — owns playback state.
pub struct Sound2D {
    handle: Option<StaticSoundHandle>,
    volume: f32,
    pitch: f32,
    looping: bool,
    pan: f32,
    duration_secs: f32,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn play(&mut self)` | Start or restart playback |
| `pub fn pause(&mut self)` | Pause (position preserved) |
| `pub fn resume(&mut self)` | Resume from pause |
| `pub fn stop(&mut self)` | Stop playback |
| `pub fn is_playing(&self) -> bool` | Is currently playing? |
| `pub fn is_paused(&self) -> bool` | Is paused? |
| `pub fn set_volume(&mut self, v: f32)` | Set volume (0.0..1.0) |
| `pub fn set_pitch(&mut self, p: f32)` | Set pitch multiplier (1.0 = normal) |
| `pub fn set_looping(&mut self, l: bool)` | Set looping |
| `pub fn set_pan(&mut self, pan: f32)` | Set stereo pan (-1.0 left, 0.0 center, 1.0 right) |
| `pub fn seek(&mut self, secs: f32) -> OpticResult<()>` | Seek to position (seconds) |
| `pub fn position_secs(&self) -> f32` | Current playback position |
| `pub fn duration_secs(&self) -> f32` | Total sound duration |
| `pub fn delete(mut self)` | Stop and free resources |

### Sound3D

A handle to a playing 3D sound with spatial audio.

Derives:
- None

Implements:
- None

```rust
// No derives — owns playback and spatial state.
pub struct Sound3D {
    handle: Option<StaticSoundHandle>,
    spatial_track: Option<SpatialTrackHandle>,
    volume: f32,
    pitch: f32,
    looping: bool,
    transform: Transform3D,
    min_distance: f32,
    max_distance: f32,
    duration_secs: f32,
}
```

| Signature | Description |
|-----------|-------------|
| `pub fn play(&mut self)` | Start or restart playback |
| `pub fn pause(&mut self)` | Pause (position preserved) |
| `pub fn resume(&mut self)` | Resume from pause |
| `pub fn stop(&mut self)` | Stop playback |
| `pub fn is_playing(&self) -> bool` | Is currently playing? |
| `pub fn is_paused(&self) -> bool` | Is paused? |
| `pub fn set_volume(&mut self, v: f32)` | Set volume (0.0..1.0) |
| `pub fn set_pitch(&mut self, p: f32)` | Set pitch multiplier (1.0 = normal) |
| `pub fn set_looping(&mut self, l: bool)` | Set looping |
| `pub fn seek(&mut self, secs: f32) -> OpticResult<()>` | Seek to position (seconds) |
| `pub fn position_secs(&self) -> f32` | Current playback position |
| `pub fn duration_secs(&self) -> f32` | Total sound duration |
| `pub fn set_min_max_distance(&mut self, min: f32, max: f32)` | Set spatial attenuation range |
| `pub fn update(&mut self)` | Push `transform.pos` to spatial audio backend (call each frame) |
| `pub fn delete(mut self)` | Stop and free resources |

**3D Audio Notes:**
- The listener position is set via `AudioEngine::set_listener` or `set_listener_from_camera`.
- `Sound3D::update()` must be called each frame to sync the emitter position.
- `min_distance` / `max_distance` control attenuation: full volume within `min`, silent beyond `max`.

---

## 11. Removed: `optic_signals`

The `optic_signals` crate has been removed. Its sole type, `Signal<T>`, has been
relocated and renamed to `Dirty<T>` in the `optic_render::asset::attr` module —
see [Dirty](#dirty) in the Renderer section.

