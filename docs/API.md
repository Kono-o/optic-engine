# Optic Engine — Public API Reference

> **Note:** Throughout the API, `OpticResult<T>` is `Result<T, OpticError>`.

> **Note:** Import everything via `use optic::*;` — the `optic` crate re-exports all sub-crates
> unmodified. Each crate below curates its own public surface; sub-module items use qualified
> paths (e.g. `optic::asset::ShaderFile`, `optic::cgmath::Vector3`).

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
   - [Center](#center)
6. [Game Loop (`optic_loop`)](#6-game-loop-optic_loop)
   - [Runtime Trait](#runtime-trait)
   - [Game](#game)
   - [Time](#time)
   - [FrameState](#framestate)
   - [WindowState](#windowstate)
   - [GameLoop](#gameloop)
   - [Standalone `run()`](#standalone-run)
7. [File Utilities (`optic_file`)](#7-file-utilities-optic_file)

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
pub struct Size2D { pub w: u32, pub h: u32 }
pub struct Size3D { pub w: u32, pub h: u32, pub d: u32 }
pub struct ClipDist { pub near: f32, pub far: f32 }
pub enum CamProj { Ortho, Persp }
```

#### Size2D

| Signature | Description |
|-----------|-------------|
| `fn empty() -> Size2D` | `Size2D { w: 0, h: 0 }` |
| `fn from(w: u32, h: u32) -> Self` | New from dimensions |
| `fn from(arr: [u32; 2]) -> Self` | From array (via `Components` trait) |
| `fn from(tup: (u32, u32)) -> Self` | From tuple |
| `fn shave(&self, n: u32) -> Size2D` | Subtract n from each side (saturating) |
| `fn aspect_ratio(&self) -> f32` | `w as f32 / h as f32` (clamped to 0.001) |
| `fn is_empty(&self) -> bool` | True if w==0 || h==0 |
| `fn area(&self) -> u64` | `w * h` |
| `fn min(&self, other) -> Size2D` | Componentwise min |
| `fn max(&self, other) -> Size2D` | Componentwise max |
| `fn fit_within(&self, max: Size2D) -> Size2D` | Scale down to fit max (preserve aspect) |
| `fn scaled_to_width(&self, w: u32) -> Size2D` | Scale to target width (preserve aspect) |
| `fn scaled_to_height(&self, h: u32) -> Size2D` | Scale to target height (preserve aspect) |
| `fn to_size3d(&self, depth: u32) -> Size3D` | Promote to 3D with given depth |
| `a + b` → `Size2D` | Saturating addition |
| `a - b` → `Size2D` | Saturating subtraction |
| `s * f32` → `Size2D` | Scalar multiplication (rounded, clamped ≥0) |

#### Size3D

| Signature | Description |
|-----------|-------------|
| `fn empty() -> Size3D` | Zero-initialized |
| `fn from(w: u32, h: u32, d: u32) -> Self` | New from dimensions |
| `fn from(arr: [u32; 3]) -> Self` | From array |
| `fn from(tup: (u32, u32, u32)) -> Self` | From tuple |
| `fn shave(&self, n: u32) -> Size3D` | Subtract n from each side (saturating) |
| `fn is_empty(&self) -> bool` | True if w==0 \|\| h==0 \|\| d==0 |
| `fn volume(&self) -> u64` | `w * h * d` |
| `fn min(&self, other) -> Size3D` | Componentwise min |
| `fn max(&self, other) -> Size3D` | Componentwise max |
| `fn to_size2d(&self) -> Size2D` | Drop depth |
| `a + b` → `Size3D` | Saturating addition |
| `a - b` → `Size3D` | Saturating subtraction |
| `s * f32` → `Size3D` | Scalar multiplication (rounded, clamped ≥0) |

#### ClipDist

- `impl Default` → `ClipDist { near: 0.01, far: 1000.0 }`
- `fn from(near: f32, far: f32) -> ClipDist`

### Coordinate Types

```rust
pub struct Coord2D { pub x: f64, pub y: f64 }
pub struct CoordOffset { pub x: f64, pub y: f64 }
```

| Method | Coord2D (point) | CoordOffset (vector) |
|--------|-----------------|----------------------|
| `empty()` | (0,0) | (0,0) |
| `from(x, y)` | New coord | New coord |
| `from_tup((x, y))` | From tuple | From tuple |
| `is_inside(size: Size2D)` | Checks bounds | — |
| `is_zero()` | — | Returns true if both zero |
| `distance_to(other)` | Distance to another point | — |
| `midpoint(other)` | Midpoint to another point | — |
| `lerp(other, t)` | Point interpolation (0..1) | Vector interpolation (0..1) |
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

```rust
pub enum PolyMode { Points, WireFrame, Filled }
pub enum Cull { Clock, AntiClock }
pub enum DrawMode { Points, Lines, Triangles, Strip }        // Default: Triangles
pub enum ImgFormat { R(u8), RG(u8), RGB(u8), RGBA(u8) }
pub enum ImgFilter { Closest, Linear }
pub enum ImgWrap { Repeat, Extend, Clip }
pub enum ATTRType { U8, I8, U16, I16, U32, I32, F32, F64 }
```

#### ImgFormat methods

| Signature | Description |
|-----------|-------------|
| `fn channels(&self) -> u8` | Number of color channels (1-4) |
| `fn bit_depth(&self) -> u8` | Bits per channel |
| `fn pixel_size(&self) -> u8` | Total bytes per pixel (channels × bit_depth/8) |
| `fn from(channels: u8, bit_depth: u8) -> ImgFormat` | Construct from channel count and bit depth |

### Error Types

```rust
pub enum OpticErrorKind {
    Init, OpenGL, Shader, Asset, File, Framebuffer, Custom,
}

pub struct OpticError {
    pub kind: OpticErrorKind,
    pub msg: String,
}

pub type OpticResult<T> = Result<T, OpticError>;
```

| Signature | Description |
|-----------|-------------|
| `fn new(kind: OpticErrorKind, msg: &str) -> Self` | Construct an error |
| `fn custom(msg: &str) -> Self` | Shorthand for `OpticErrorKind::Custom` |
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

// Optic cache extensions
pub const OSHDR: &str = "oshdr";
pub const OMESH: &str = "omesh";
pub const OTXTR: &str = "otxtr";

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

```rust
pub struct RGBA(pub f32, pub f32, pub f32, pub f32);
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

```rust
pub struct RGB(pub f32, pub f32, pub f32);
```

| Method | Description |
|--------|-------------|
| `new(r, g, b)` | Constructor |
| `grey(lum)` | Grey |
| `from_rgba(rgba)` | Drop alpha |
| `to_rgba()` | RGBA with alpha 1.0 |

Implements `ToRgba`, `FromRgba`, `ColorInfo`, `channel_lerp`, `Add`, `Sub`, `Mul`, `Div` (`f32`),
`From<[f32; 3]>`.

### HSV

```rust
pub struct HSV { pub h: f32, pub s: f32, pub v: f32 }
```

- `h`: 0..360 (wraps), `s`/`v`: 0..1
- `new(h, s, v)` → constructor
- `to_rgba_alpha(alpha)` → RGBA with given alpha

HSV intentionally does **not** implement `ChannelArray`, `Add`, `Sub`, `Mul`, or `lerp` — hue
wraparound makes componentwise arithmetic produce wrong colors. Convert to RGBA (`.into()`) for
arithmetic, or use `Gradient` with `GradientColorSpace::Hsv` for hue-aware interpolation.

Implements `ToRgba`, `FromRgba`, `ColorInfo`, `From<RGBA>`.

### HSL

```rust
pub struct HSL { pub h: f32, pub s: f32, pub l: f32 }
```

- `h`: 0..360 (wraps), `s`/`l`: 0..1
- `new(h, s, l)` → constructor
- `to_rgba_alpha(alpha)` → RGBA with given alpha

Same arithmetic caveat as HSV. Implements `ToRgba`, `FromRgba`, `ColorInfo`, `From<RGBA>`.

### Gradient

```rust
pub struct Gradient { /* fields private */ }

pub struct GradientStop { pub position: f32, pub color: RGBA }

pub enum GradientInterp { Linear, Step, SmoothStep }
pub enum GradientColorSpace { Rgb, Hsv }
pub enum GradientWrap { Clamp, Repeat, PingPong }
```

| Method | Description |
|--------|-------------|
| `new()` | Empty gradient (linear, RGB, clamp) |
| `add_stop(position, color: impl ToRgba)` | Insert stop (sorted) |
| `remove_stop(index)` | Remove stop by index |
| `stops()` | `&[GradientStop]` |
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
    fn channels(&self) -> [f32; N];
    fn from_channels(ch: [f32; N]) -> Self;
}

pub fn channel_lerp<T: ChannelArray<N>, const N: usize>(a: T, b: T, t: f32) -> T;
pub fn rgba_to_hsv(rgba: RGBA) -> HSV;
pub fn hsv_to_rgba(hsv: HSV) -> RGBA;
pub fn rgba_to_hsl(rgba: RGBA) -> HSL;
pub fn hsl_to_rgba(hsl: HSL) -> RGBA;
```

- `ChannelArray<4>` implemented for `RGBA`, `ChannelArray<3>` for `RGB`.
- `channel_lerp` is the building block for RGB gradient interpolation and general use.
- `hsv_to_rgba`/`rgba_to_hsv`/`hsl_to_rgba`/`rgba_to_hsl` are `pub(crate)` — use
  `ToRgba`/`FromRgba` traits instead.

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

```rust
pub struct Window { /* all fields private */ }
```

#### Construction & Lifecycle

| Signature | Description |
|-----------|-------------|
| `fn new(el: &EventLoop<()>, title: &str, size: Size2D) -> Self` | Create a new window (starts hidden, opaque) |
| `fn new_transparent(el: &EventLoop<()>, title: &str, size: Size2D) -> Self` | Create a transparent window (X11 depth-32 ARGB visual) |
| `fn close(&mut self)` | Close the window |
| `fn is_closed(&self) -> bool` | Returns true if window handle was dropped |
| `fn is_running(&self) -> bool` | Returns `!is_closed()` |
| `fn id(&self) -> Option<WindowId>` | Winit window ID |
| `fn request_redraw(&self)` | Request a redraw on the next frame |

#### Raw Handles

| Signature | Description |
|-----------|-------------|
| `fn raw_handle(&self) -> Option<RawWindowHandle>` | Platform-specific window handle |
| `fn raw_display_handle(&self) -> Option<RawDisplayHandle>` | Platform-specific display handle |

#### Sizing

| Signature | Description |
|-----------|-------------|
| `fn size(&self) -> Size2D` | Current inner size (live winit query) |
| `fn set_size(&self, size: Size2D)` | Set inner size |
| `fn prev_size(&self) -> Size2D` | Last cached size |
| `fn min_size(&self) -> Option<Size2D>` | Minimum window size |
| `fn set_min_size(&mut self, size: Option<Size2D>)` | Set minimum size |
| `fn max_size(&self) -> Option<Size2D>` | Maximum window size |
| `fn set_max_size(&mut self, size: Option<Size2D>)` | Set maximum size |
| `fn resizable(&self) -> bool` | Whether the window is resizable |
| `fn set_resizable(&self, enable: bool)` | Toggle resizability |

#### Position

| Signature | Description |
|-----------|-------------|
| `fn position(&self) -> Coord2D` | Outer position on desktop (live winit query) |
| `fn set_position(&self, pos: Coord2D)` | Set outer position |
| `fn center_on_screen(&self)` | Center window on the current monitor |
| `fn prev_position(&self) -> Coord2D` | Last cached position |
| `fn position_delta(&mut self) -> CoordOffset` | Movement since last polled (resets to zero on read) |

#### Title

| Signature | Description |
|-----------|-------------|
| `fn title(&self) -> String` | Current window title |
| `fn set_title(&self, title: &str)` | Set window title |

#### Fullscreen

| Signature | Description |
|-----------|-------------|
| `fn is_fullscreen(&self) -> bool` | Is fullscreen? |
| `fn set_fullscreen(&self, enable: bool)` | Toggle fullscreen on/off |
| `fn toggle_fullscreen(&self)` | Toggle fullscreen state |

#### State

| Signature | Description |
|-----------|-------------|
| `fn is_visible(&self) -> bool` | Is window visible? |
| `fn set_visible(&self, visible: bool)` | Show/hide window |
| `fn is_minimized(&self) -> bool` | Is minimized? |
| `fn minimize(&self)` | Minimize window |
| `fn restore(&self)` | Restore from minimized |
| `fn is_maximized(&self) -> bool` | Is maximized? |
| `fn maximize(&self)` | Maximize window |
| `fn unmaximize(&self)` | Restore from maximized |
| `fn has_focus(&self) -> bool` | Does the window have keyboard focus? |
| `fn focus(&self)` | Request focus |

#### Cursor

| Signature | Description |
|-----------|-------------|
| `fn cursor_pos(&self) -> Coord2D` | Last-known cursor position (cached from events/setters) |
| `fn set_cursor_pos(&mut self, pos: Coord2D)` | Set cursor position (also updates cache) |
| `fn cursor_delta(&self) -> CoordOffset` | Difference from previous frame's cursor position |
| `fn cursor_pos_normalized(&self) -> Coord2D` | Cursor pos normalized to [0,1] by window size |
| `fn is_cursor_inside(&self) -> bool` | Is cursor inside the window? |
| `fn is_cursor_visible(&self) -> bool` | Is cursor visible? |
| `fn set_cursor_visible(&mut self, visible: bool)` | Show/hide cursor |
| `fn toggle_cursor_visible(&mut self)` | Toggle cursor visibility |
| `fn is_cursor_grabbed(&self) -> bool` | Is cursor grabbed? |
| `fn set_cursor_grab(&mut self, grab: bool) -> Result<(), ()>` | Set cursor grab mode |
| `fn toggle_cursor_grab(&mut self)` | Toggle cursor grab |
| `fn is_cursor_confined(&self) -> bool` | Is cursor confined to window? |
| `fn set_cursor_confine(&mut self, confine: bool) -> Result<(), ()>` | Confine/free cursor |
| `fn toggle_cursor_confine(&mut self)` | Toggle confine |
| `fn is_cursor_loopback(&self) -> bool` | Is cursor loopback enabled? |
| `fn set_cursor_loopback(&mut self, loopback: bool)` | Enable/disable edge-wrapping loopback |

#### Screen Info

| Signature | Description |
|-----------|-------------|
| `fn screen_info(&self) -> Option<ScreenInfo>` | Information about the current monitor |

#### Frame Update

| Signature | Description |
|-----------|-------------|
| `fn update_frame(&mut self)` | Call once per frame to compute cursor delta and handle loopback teleport |

#### Internal (doc-hidden, used by the event loop)

| Signature | Description |
|-----------|-------------|
| `fn notify_cursor_moved(&mut self, pos: Coord2D)` | Update cached cursor position from event |
| `fn notify_cursor_inside(&mut self, inside: bool)` | Update cursor-enter/leave state |

### Events & Input

```rust
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
}

pub struct ButtonState {
    pub held: bool,
    pub press_frame: u64,
    pub release_frame: u64,
}
```

#### Enums

```rust
pub enum Is { Pressed, Released, Held }

pub enum Mouse { Left, Right, Middle, Back, Forward, Other(u16) }

pub enum GamepadButton {
    A, B, X, Y, LB, RB, LT, RT, Back, Start, Guide,
    LeftStick, RightStick, DPadUp, DPadDown, DPadLeft, DPadRight,
    Other(u8),
}

pub enum GamepadAxis {
    LeftX, LeftY, RightX, RightY, LeftTrigger, RightTrigger,
}
```

#### Constants

```rust
pub const MAX_GAMEPADS: usize = 4;
```

#### Methods

| Signature | Description |
|-----------|-------------|
| `fn new() -> Self` | Create new input state |
| `fn clear(&mut self)` | Reset everything to defaults |
| `fn end_frame(&mut self)` | Call at end of frame (increments `frame`, clears scroll/resize) |
| `fn process_window_event(&mut self, event: &WindowEvent, _window: &Window)` | Process a winit event |
| `fn process_gilrs_event(&mut self, event: &gilrs::Event)` | Process a gilrs gamepad event |
| `fn key(&self, kc: KeyCode, action: Is) -> bool` | Check key state |
| `fn key_combo(&self, primary: KeyCode, modifier: KeyCode, action: Is) -> bool` | Check combo (e.g. Ctrl+S) |
| `fn key_combo_n(&self, keys: &[(KeyCode, Is)]) -> bool` | Check multiple keys |
| `fn any_key(&self, action: Is) -> bool` | Any key matches? |
| `fn mouse(&self, m: Mouse, action: Is) -> bool` | Check mouse button state |
| `fn any_mouse(&self, action: Is) -> bool` | Any mouse button matches? |
| `fn gamepad_connected(&self, id: usize) -> bool` | Is gamepad connected? |
| `fn gamepad_count(&self) -> usize` | Number of connected gamepads |
| `fn gamepad_button(&self, id: usize, button: GamepadButton, action: Is) -> bool` | Check gamepad button |
| `fn any_gamepad_button(&self, id: usize, action: Is) -> bool` | Any button on gamepad? |
| `fn any_gamepad(&self, action: Is) -> bool` | Any button on any gamepad? |
| `fn gamepad_axis_raw(&self, id: usize, axis: GamepadAxis) -> f32` | Raw axis value [-1, 1] |
| `fn gamepad_axis(&self, id: usize, axis: GamepadAxis) -> f32` | Axis value with default deadzone |
| `fn gamepad_axis_deadzoned(&self, id: usize, axis: GamepadAxis, deadzone: f32) -> f32` | Axis value with custom deadzone |

**Note:** `KeyCode` is re-exported from `winit::keyboard::KeyCode`.

### ScreenInfo

```rust
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
| `fn from_handle(handle: &winit::monitor::MonitorHandle) -> Self` | Build from winit monitor handle |

### Re-exports

`optic_window` re-exports `winit` and `gilrs` crates.

---

## 4. Renderer (`optic_render`)

### GPU

```rust
pub struct GPU {
    pub ctx: RenderContext,
    pub poly_mode: PolyMode,
    pub cull_face: Cull,
    pub bg_color: RGBA,
    pub msaa: bool,
    pub msaa_samples: u32,
    pub culling: bool,
    pub fallback_shader2d: Shader,
    pub fallback_shader3d: Shader,
    pub fallback_texture: Texture2D,
    pub canvas_size: Size2D,
}
```

#### Construction

| Signature | Description |
|-----------|-------------|
| `fn new_headless() -> OpticResult<Self>` | Create a headless GPU context (pbuffer only) |
| `fn new_windowed(handle: RawWindowHandle, display_handle: RawDisplayHandle, size: Size2D) -> OpticResult<Self>` | Create GPU context from a window |

#### Info

| Signature | Description |
|-----------|-------------|
| `fn version(&self) -> &str` | GL version string |
| `fn lang_version(&self) -> &str` | GLSL version string |
| `fn name(&self) -> &str` | Renderer name |
| `fn log_backend_info(&self)` | Log GL info to stdout |
| `fn log_info(&self)` | Log all GPU info |

#### State

| Signature | Description |
|-----------|-------------|
| `fn clear(&self)` | Clear color + depth buffers |
| `fn set_msaa_samples(&mut self, samples: u32)` | Set MSAA sample count |
| `fn set_bg_color(&mut self, color: RGBA)` | Set background clear color |
| `fn set_poly_mode(&mut self, mode: PolyMode)` | Set polygon mode |
| `fn toggle_wireframe(&mut self)` | Toggle between Filled and WireFrame |
| `fn set_msaa(&mut self, enable: bool)` | Enable/disable MSAA |
| `fn toggle_msaa(&mut self)` | Toggle MSAA |
| `fn set_culling(&mut self, enable: bool)` | Enable/disable backface culling |
| `fn toggle_culling(&mut self)` | Toggle culling |
| `fn set_cull_face(&mut self, cull_face: Cull)` | Set which face to cull |
| `fn flip_cull_face(&mut self)` | Swap cull face |
| `fn set_canvas_size(&mut self, size: Size2D)` | Set canvas/render size |
| `fn set_wire_width(&mut self, width: f32)` | Wireframe line width |
| `fn set_point_size(&self, size: f32)` | Point size |

#### Fallback Assets

| Signature | Description |
|-----------|-------------|
| `fn fallback_shader3d(&self) -> Shader` | Default 3D shader |
| `fn fallback_shader2d(&self) -> Shader` | Default 2D shader |

#### GPU Resource Shipping

| Signature | Description |
|-----------|-------------|
| `fn ship_mesh3d(&self, file: &Mesh3DFile) -> Mesh3D` | Upload 3D mesh to GPU |
| `fn ship_mesh2d(&self, file: &Mesh2DFile) -> Mesh2D` | Upload 2D mesh to GPU |
| `fn ship_shader(&self, asset: &ShaderFile) -> Option<Shader>` | Compile and upload shader |
| `fn ship_texture(&self, image: &TextureFile) -> Texture2D` | Upload texture to GPU |
| `fn ship_gradient(&self, gradient: &Gradient, resolution: u32) -> Texture2D` | Bake gradient to 1D RGBA texture |
| `fn ship_canvas(&mut self, desc: &CanvasDesc) -> OpticResult<Canvas>` | Create offscreen render target |

#### Rendering

| Signature | Description |
|-----------|-------------|
| `fn set_render_target(&mut self, target: &RenderTarget) -> OpticResult<()>` | Set active render target |
| `fn clear_target(&mut self, color: Option<RGBA>, depth: bool)` | Clear current render target (if color is None, uses bg_color) |
| `fn current_render_target_size(&self) -> Size2D` | Size of current render target |
| `fn render3d(&self, mesh: &Mesh3D, camera: &Camera)` | Render a 3D mesh |
| `fn render2d(&self, mesh: &Mesh2D)` | Render a 2D mesh (orthographic) |

### RenderContext

```rust
pub struct RenderContext {
    pub display: egl::Display,
    pub context: egl::Context,
    pub surfaces: Vec<WindowSurface>,
    pub active_index: Option<usize>,
    pub gl_ver: String,
    pub glsl_ver: String,
    pub device: String,
}
```

| Signature | Description |
|-----------|-------------|
| `fn new_headless() -> OpticResult<Self>` | Create headless context |
| `fn new_windowed(handle: RawWindowHandle, display_handle: RawDisplayHandle, size: Size2D) -> OpticResult<Self>` | Create context attached to a window |
| `fn attach_window(&mut self, handle: RawWindowHandle, size: Size2D) -> OpticResult<usize>` | Attach an additional window surface |
| `fn resize_window(&mut self, index: usize, size: Size2D)` | Resize a window surface |
| `fn make_current(&self, index: usize) -> OpticResult<()>` | Make a surface current |
| `fn swap_buffers(&self, index: usize) -> OpticResult<()>` | Swap buffers for a surface |
| `fn clear(&self)` | Clear color + depth |
| `fn set_vsync(&self, enable: bool)` | Enable/disable vsync |
| `fn set_clear_color(&self, color: RGBA)` | Set clear color |

```rust
pub struct WindowSurface {
    pub surface: egl::Surface,
    pub size: Size2D,
}
```

### GL Static Methods

```rust
pub struct GL;  // unit struct, all methods are associated
```

| Signature | Description |
|-----------|-------------|
| `fn clear()` | Clear color + depth buffers |
| `fn set_clear(color: RGBA)` | Set clear color via `glClearColor` |
| `fn resize(size: Size2D)` | Set viewport |
| `fn poly_mode(mode: PolyMode)` | Set polygon render mode |
| `fn enable_msaa(enable: bool)` | Enable/disable MSAA |
| `fn enable_depth(enable: bool)` | Enable/disable depth testing |
| `fn enable_alpha(enable: bool)` | Enable/disable alpha blending (SRC_ALPHA, ONE_MINUS_SRC_ALPHA) |
| `fn enable_cull(enable: bool)` | Enable/disable face culling |
| `fn set_cull_face(face: Cull)` | Set which face to cull |
| `fn set_point_size(size: f32)` | Set point size |
| `fn set_wire_width(width: f32)` | Set line width |
| `fn bind_shader(id: u32)` | Bind shader program |
| `fn unbind_shader()` | Unbind shader |
| `fn bind_texture_at(tex_id: u32, slot: u32)` | Bind texture to slot |
| `fn unbind_texture()` | Unbind texture |
| `fn bind_vao(id: u32)` | Bind VAO |
| `fn unbind_vao()` | Unbind VAO |
| `fn bind_buffer(id: u32)` | Bind VBO |
| `fn unbind_buffer()` | Unbind VBO |
| `fn bind_ebo(id: u32)` | Bind EBO |
| `fn unbind_ebo()` | Unbind EBO |
| `fn bind_ssbo(id: u32)` | Bind SSBO |
| `fn unbind_ssbo()` | Unbind SSBO |

### Camera

```rust
pub struct Camera {
    pub transform: CamTransform,
}
```

| Signature | Description |
|-----------|-------------|
| `fn new(size: Size2D, proj: CamProj) -> Self` | Create camera with size and projection type |
| `fn match_canvas_size(canvas: &Canvas, proj: CamProj) -> Self` | Create camera matching canvas size |
| `fn pre_update(&mut self)` | Recalculate view/projection matrices |
| `fn fov(&self) -> f32` | Field of view (degrees) |
| `fn ortho_scale(&self) -> f32` | Orthographic scale |
| `fn proj(&self) -> CamProj` | Projection type |
| `fn clip(&self) -> ClipDist` | Clip distances |
| `fn set_clip(&mut self, clip: ClipDist)` | Set clip distances |
| `fn set_clip_near(&mut self, near: f32)` | Set near clip |
| `fn set_clip_far(&mut self, far: f32)` | Set far clip |
| `fn set_size(&mut self, size: Size2D)` | Set viewport size |
| `fn set_proj(&mut self, proj: CamProj)` | Set projection type |
| `fn set_fov(&mut self, fov: f32)` | Set FOV |
| `fn add_fov(&mut self, value: f32)` | Add to FOV |
| `fn set_ortho_scale(&mut self, value: f32)` | Set ortho scale |
| `fn add_ortho_scale(&mut self, value: f32)` | Add to ortho scale |
| `fn fly_forw(&mut self, speed: f32)` | Move camera forward |
| `fn fly_back(&mut self, speed: f32)` | Move camera backward |
| `fn fly_left(&mut self, speed: f32)` | Move camera left |
| `fn fly_right(&mut self, speed: f32)` | Move camera right |
| `fn fly_up(&mut self, speed: f32)` | Move camera up |
| `fn fly_down(&mut self, speed: f32)` | Move camera down |
| `fn spin_x(&mut self, speed: f32)` | Pitch (degrees) |
| `fn spin_y(&mut self, speed: f32)` | Yaw (degrees) |
| `fn spin_z(&mut self, speed: f32)` | Roll (degrees) |

### CamTransform

```rust
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
| `fn calc_matrices(&mut self)` | Recalculate view/projection matrices from position/rotation |
| `fn view_matrix(&self) -> Matrix4<f32>` | Current view matrix |
| `fn proj_matrix(&self) -> Matrix4<f32>` | Current projection matrix |

### Mesh3D

```rust
pub struct Mesh3D {
    pub visibility: bool,
    pub handle: MeshHandle,
    pub shader: Option<Shader>,
    pub transform: Transform3D,
    pub draw_mode: DrawMode,
}
```

| Signature | Description |
|-----------|-------------|
| `fn set_shader(&mut self, shader: Shader)` | Assign a shader |
| `fn remove_shader(&mut self)` | Remove shader |
| `fn get_draw_mode(&self) -> DrawMode` | Get draw mode |
| `fn set_draw_mode(&mut self, draw_mode: DrawMode)` | Set draw mode |
| `fn index_count(&self) -> u32` | Number of indices |
| `fn vertex_count(&self) -> u32` | Number of vertices |
| `fn has_indices(&self) -> bool` | Has index buffer? |
| `fn is_empty(&self) -> bool` | Zero vertices? |
| `fn is_visible(&self) -> bool` | Visibility && non-empty |
| `fn set_visibility(&mut self, enable: bool)` | Show/hide |
| `fn toggle_visibility(&mut self)` | Toggle visibility |
| `fn update(&mut self)` | Recalculate transform matrix |
| `fn delete(self)` | Delete GPU resources |
| `fn log_info(&self)` | Print mesh info |
| `fn render(&self, view: &Matrix4<f32>, proj: &Matrix4<f32>)` | Render with explicit view/proj matrices |

### Mesh2D

```rust
pub struct Mesh2D {
    pub visibility: bool,
    pub handle: MeshHandle,
    pub shader: Option<Shader>,
    pub transform: Transform2D,
    pub draw_mode: DrawMode,
}
```

Same methods as Mesh3D, plus:

| Signature | Description |
|-----------|-------------|
| `fn render(&self, proj: &Matrix4<f32>)` | Render with explicit projection matrix |

### MeshHandle

```rust
pub struct MeshHandle {
    pub layouts: Vec<(ATTRInfo, u32)>,
    pub draw_mode: DrawMode,
    pub has_indices: bool,
    pub vert_count: u32,
    pub ind_count: u32,
    pub vao_id: u32,
    pub buf_id: u32,
    pub ind_id: u32,
}
```

| Signature | Description |
|-----------|-------------|
| `fn draw(&self)` | Issue the draw call |
| `fn delete(self)` | Free GPU resources |

### Shader

```rust
pub struct Shader {
    pub workers: Workers,
    pub id: u32,
    pub is_compute: bool,
    pub tex_ids: Vec<Option<u32>>,
    pub sbo_ids: Vec<Option<u32>>,
}
```

| Signature | Description |
|-----------|-------------|
| `fn new(id: u32, is_compute: bool) -> Self` | Wrap a shader program ID |
| `fn attach_tex(&mut self, tex: &Texture2D)` | Attach texture to next available slot |
| `fn attach_sbo(&mut self, sbo: &StorageBuffer)` | Attach SSBO to next available slot |
| `fn set_tex_at_slot(&mut self, tex: &Texture2D, slot: Slot)` | Bind texture to specific slot |
| `fn set_sbo_at_slot(&mut self, sbo: &StorageBuffer, slot: Slot)` | Bind SSBO to specific slot |
| `fn delete(self)` | Delete the shader program |
| `fn bind(&self)` | Use this shader |
| `fn unbind(&self)` | Unbind shader |
| `fn compute(&self)` | Dispatch compute shader with worker group counts |
| `fn uniform_location(&self, name: &str) -> Option<u32>` | Query uniform location |
| `fn texture_binds(&self) -> Vec<(u32, u32)>` | List of (tex_id, slot) bindings |
| `fn storage_binds(&self) -> Vec<(u32, u32)>` | List of (sbo_id, slot) bindings |
| `fn bind_textures(&self)` | Bind all attached textures |
| `fn bind_storages(&self)` | Bind all attached SSBOs |
| `fn set_i32(&self, name: &str, v: i32)` | Set int uniform |
| `fn set_u32(&self, name: &str, v: u32)` | Set uint uniform |
| `fn set_f32(&self, name: &str, v: f32)` | Set float uniform |
| `fn set_vec2_f32(&self, name: &str, v: Vector2<f32>)` | Set vec2 uniform |
| `fn set_vec3_f32(&self, name: &str, v: Vector3<f32>)` | Set vec3 uniform |
| `fn set_vec4_f32(&self, name: &str, v: Vector4<f32>)` | Set vec4 uniform |
| `fn set_vec2_i32(&self, name: &str, v: Vector2<i32>)` | Set ivec2 uniform |
| `fn set_vec3_i32(&self, name: &str, v: Vector3<i32>)` | Set ivec3 uniform |
| `fn set_vec4_i32(&self, name: &str, v: Vector4<i32>)` | Set ivec4 uniform |
| `fn set_vec2_u32(&self, name: &str, v: Vector2<u32>)` | Set uvec2 uniform |
| `fn set_vec3_u32(&self, name: &str, v: Vector3<u32>)` | Set uvec3 uniform |
| `fn set_vec4_u32(&self, name: &str, v: Vector4<u32>)` | Set uvec4 uniform |
| `fn set_m2_f32(&self, name: &str, m: Matrix2<f32>)` | Set mat2 uniform |
| `fn set_m3_f32(&self, name: &str, m: Matrix3<f32>)` | Set mat3 uniform |
| `fn set_m4_f32(&self, name: &str, m: Matrix4<f32>)` | Set mat4 uniform |

### Texture2D

```rust
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
| `fn new(id: u32, size: Size2D, fmt: ImgFormat, filter: ImgFilter, wrap: ImgWrap) -> Self` | Wrap a GL texture ID |
| `fn size(&self) -> Size2D` | Texture size |
| `fn wrap(&self) -> ImgWrap` | Current wrap mode |
| `fn set_wrap(&mut self, wrap: ImgWrap)` | Set wrap mode |
| `fn filter(&self) -> ImgFilter` | Current filter mode |
| `fn set_filter(&mut self, filter: ImgFilter)` | Set filter mode |
| `fn delete(self)` | Delete the GL texture |

### StorageBuffer

```rust
pub struct StorageBuffer {
    pub id: u32,
    pub size: usize,
}
```

| Signature | Description |
|-----------|-------------|
| `fn new(size: usize) -> Self` | Create SSBO of given size |
| `fn resize(&mut self, size: usize)` | Resize buffer |
| `fn fill(&mut self, data: &[u8])` | Fill buffer with byte data |
| `fn subfill(&mut self, offset: usize, data: &[u8])` | Write data at offset |
| `fn fetch(&self) -> Vec<u8>` | Read back buffer contents |
| `fn delete(self)` | Delete buffer |

### Transform2D

```rust
pub struct Transform2D { /* impl Default */ }
```

| Signature | Description |
|-----------|-------------|
| `fn calc_matrix(&mut self)` | Recalculate transform matrix from pos/rot/scale |
| `fn pos(&self) -> Vector2<f32>` | Position |
| `fn rot(&self) -> f32` | Rotation (degrees) |
| `fn layer(&self) -> u8` | Z layer |
| `fn scale(&self) -> Vector2<f32>` | Scale |
| `fn matrix(&self) -> Matrix4<f32>` | Current 4×4 transform matrix |
| `fn move_all(&mut self, x: f32, y: f32)` | Translate |
| `fn move_x(&mut self, x: f32)` | Translate X |
| `fn move_y(&mut self, y: f32)` | Translate Y |
| `fn set_pos_all(&mut self, x: f32, y: f32)` | Set position |
| `fn set_pos_x(&mut self, x: f32)` | Set position X |
| `fn set_pos_y(&mut self, y: f32)` | Set position Y |
| `fn rotate(&mut self, rot: f32)` | Add rotation |
| `fn set_rot(&mut self, rot: f32)` | Set rotation |
| `fn set_layer(&mut self, layer: u8)` | Set Z layer |
| `fn scale_all(&mut self, x: f32, y: f32)` | Add scale |
| `fn scale_same(&mut self, xy: f32)` | Uniform add scale |
| `fn scale_x(&mut self, x: f32)` | Add scale X |
| `fn scale_y(&mut self, y: f32)` | Add scale Y |
| `fn set_scale_all(&mut self, x: f32, y: f32)` | Set scale |
| `fn set_scale_same(&mut self, xy: f32)` | Uniform set scale |
| `fn set_scale_x(&mut self, x: f32)` | Set scale X |
| `fn set_scale_y(&mut self, y: f32)` | Set scale Y |

### Transform3D

```rust
pub struct Transform3D { /* impl Default */ }
```

| Signature | Description |
|-----------|-------------|
| `fn calc_matrix(&mut self)` | Recalculate transform matrix |
| `fn pos(&self) -> Vector3<f32>` | Position |
| `fn rot(&self) -> Vector3<f32>` | Rotation (degrees) |
| `fn scale(&self) -> Vector3<f32>` | Scale |
| `fn matrix(&self) -> Matrix4<f32>` | Current 4×4 transform matrix |
| `fn move_all(&mut self, x: f32, y: f32, z: f32)` | Translate |
| `fn move_x(&mut self, x: f32)` | Translate X |
| `fn move_y(&mut self, y: f32)` | Translate Y |
| `fn move_z(&mut self, z: f32)` | Translate Z |
| `fn set_pos_all(&mut self, x: f32, y: f32, z: f32)` | Set position |
| `fn set_pos_x(&mut self, x: f32)` | Set position X |
| `fn set_pos_y(&mut self, y: f32)` | Set position Y |
| `fn set_pos_z(&mut self, z: f32)` | Set position Z |
| `fn rotate_all(&mut self, x: f32, y: f32, z: f32)` | Add rotation |
| `fn rotate_x(&mut self, x: f32)` | Add rotation X |
| `fn rotate_y(&mut self, y: f32)` | Add rotation Y |
| `fn rotate_z(&mut self, z: f32)` | Add rotation Z |
| `fn set_rot_all(&mut self, x: f32, y: f32, z: f32)` | Set rotation |
| `fn set_rot_x(&mut self, x: f32)` | Set rotation X |
| `fn set_rot_y(&mut self, y: f32)` | Set rotation Y |
| `fn set_rot_z(&mut self, z: f32)` | Set rotation Z |
| `fn scale_all(&mut self, x: f32, y: f32, z: f32)` | Add scale |
| `fn scale_same(&mut self, xyz: f32)` | Uniform add scale |
| `fn scale_x(&mut self, x: f32)` | Add scale X |
| `fn scale_y(&mut self, y: f32)` | Add scale Y |
| `fn scale_z(&mut self, z: f32)` | Add scale Z |
| `fn set_scale_all(&mut self, x: f32, y: f32, z: f32)` | Set scale |
| `fn set_scale_same(&mut self, xyz: f32)` | Uniform set scale |
| `fn set_scale_x(&mut self, x: f32)` | Set scale X |
| `fn set_scale_y(&mut self, y: f32)` | Set scale Y |
| `fn set_scale_z(&mut self, z: f32)` | Set scale Z |

### Canvas

```rust
pub struct Canvas { /* all fields pub(crate) */ }
```

| Signature | Description |
|-----------|-------------|
| `fn new(desc: &CanvasDesc) -> OpticResult<Self>` | Create offscreen framebuffer |
| `fn size(&self) -> Size2D` | Canvas size |
| `fn color_tex(&self, index: usize) -> OpticResult<&Texture2D>` | Get color attachment as texture |
| `fn depth_tex(&self) -> Option<&Texture2D>` | Get depth attachment as texture (if depth_as_texture) |
| `fn set_size(&mut self, new_size: Size2D) -> OpticResult<()>` | Resize canvas |
| `fn resolve(&self)` | Resolve MSAA to single-sample textures |
| `fn blit_to_screen(&self, window_size: Size2D)` | Copy canvas to screen |
| `fn set_renderable_area(&self, x: i32, y: i32, size: Size2D) -> OpticResult<()>` | Set scissor/viewport area |
| `fn read_pixels(&self, index: usize) -> OpticResult<Vec<u8>>` | Read pixel data from color attachment |
| `fn save_to_disk(&self, index: usize, path: &str) -> OpticResult<()>` | Save color attachment to PNG |
| `fn delete(&mut self)` | Free GPU resources |

### CanvasDesc

```rust
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

```rust
pub enum RenderTarget<'a> {
    Screen,
    Canvas(&'a Canvas),
}
```

### Slot

```rust
pub enum Slot {
    S0, S1, S2, S3, S4, S5, S6, S7,
    S8, S9, S10, S11, S12, S13, S14, S15,
}
```

| Signature | Description |
|-----------|-------------|
| `fn as_index(&self) -> usize` | Slot as index (0-15) |
| `fn total_slots() -> usize` | Always 16 |

### Workers

```rust
pub struct Workers {
    pub group_x: u32,
    pub group_y: u32,
    pub group_z: u32,
}
```

| Signature | Description |
|-----------|-------------|
| `fn empty() -> Self` | All zero |
| `fn one() -> Self` | All one |
| `fn set_groups(&mut self, x: u32, y: u32, z: u32)` | Set all groups |
| `fn groups(&self) -> (u32, u32, u32)` | Get all groups |
| `fn group_x(&self) -> u32` | Get X |
| `fn group_y(&self) -> u32` | Get Y |
| `fn group_z(&self) -> u32` | Get Z |
| `fn set_group_x(&mut self, x: u32)` | Set X |
| `fn set_group_y(&mut self, y: u32)` | Set Y |
| `fn set_group_z(&mut self, z: u32)` | Set Z |

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

### ShaderFile

```rust
pub struct ShaderFile {
    pub v_src: String,
    pub f_src: String,
    pub is_compute: bool,
}
```

| Signature | Description |
|-----------|-------------|
| `fn from_src(src: &str, typ: ShaderType) -> OpticResult<Self>` | Parse a combined GLSL file (marker-based: `@vertex`/`@fragment`/`@compute`) |
| `fn from_vert_frag(v_src: &str, f_src: &str) -> Self` | Build from separate vertex and fragment source strings |
| `fn compile(&self) -> OpticResult<Shader>` | Compile shader and return a GPU handle |
| `fn from_disk(path: &str, typ: ShaderType) -> OpticResult<Self>` | Load from disk (debug: loads + overwrites cache; release: loads cache only) |
| `fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary cache (`.oshdr`) |
| `fn default_3d() -> OpticResult<Self>` | Get built-in 3D fallback shader |
| `fn default_2d() -> OpticResult<Self>` | Get built-in 2D fallback shader |

### ShaderType

```rust
pub enum ShaderType {
    Pipeline,  // vertex + fragment
    Compute,
}
```

| Signature | Description |
|-----------|-------------|
| `fn is_compute(&self) -> bool` | Is this a compute shader? |

### Mesh3DFile

```rust
pub struct Mesh3DFile {
    pub pos_attr: Pos3DATTR,
    pub col_attr: ColATTR,
    pub uvm_attr: UVMATTR,
    pub nrm_attr: NrmATTR,
    pub ind_attr: IndATTR,
    pub cus_attrs: Vec<CustomATTR>,
}
```

| Signature | Description |
|-----------|-------------|
| `fn empty() -> Self` | Empty mesh |
| `fn from_obj_src(src: &str) -> OpticResult<Self>` | Parse Wavefront OBJ (triangles only) |
| `fn from_stl_src(data: &[u8]) -> OpticResult<Self>` | Parse STL (ASCII or binary, triangles only) |
| `fn from_disk(path: &str) -> OpticResult<Self>` | Load from disk (debug: loads source + overwrites cache; release: loads cache only) |
| `fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary cache (`.omesh`) |
| `fn cube(side: f32) -> Self` | Generate a cube |
| `fn cuboid(w: f32, h: f32, d: f32) -> Self` | Generate a cuboid |
| `fn sphere(radius: f32, stacks: u32, sectors: u32) -> Self` | Generate a UV sphere |
| `fn cylinder(radius: f32, height: f32, segments: u32, cap: bool) -> Self` | Generate a cylinder |
| `fn cone(radius: f32, height: f32, segments: u32, cap: bool) -> Self` | Generate a cone |
| `fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> Self` | Generate a torus |
| `fn plane(width: f32, depth: f32) -> Self` | Generate a flat plane (XZ, Y=0) |
| `fn attach_custom_attr(&mut self, attr: CustomATTR)` | Add a custom vertex attribute |
| `fn has_no_attr(&self) -> bool` | Check if mesh has no vertex data |
| `fn starts_with_custom(&self) -> bool` | Does the first attribute layout start with a custom attribute? |
| `fn ship(&self) -> MeshHandle` | Upload to GPU and get a handle |

### Mesh2DFile

```rust
pub struct Mesh2DFile {
    pub pos_attr: Pos2DATTR,
    pub layer: u8,
    pub aspect: f32,
    pub col_attr: ColATTR,
    pub uvm_attr: UVMATTR,
    pub ind_attr: IndATTR,
    pub cus_attrs: Vec<CustomATTR>,
}
```

| Signature | Description |
|-----------|-------------|
| `fn empty() -> Self` | Empty mesh |
| `fn set_pos_attr(&mut self, attr: Pos2DATTR)` | Set positions |
| `fn set_layer(&mut self, layer: u8)` | Set Z layer |
| `fn set_center(&mut self, center: Center)` | Recenter around a pivot |
| `fn set_col_attr(&mut self, attr: ColATTR)` | Set vertex colors |
| `fn set_uvm_attr(&mut self, attr: UVMATTR)` | Set UV coordinates |
| `fn set_ind_attr(&mut self, attr: IndATTR)` | Set indices |
| `fn quad(size: &Size2D) -> Self` | Generate a textured quad |
| `fn fullscreen_quad() -> Self` | Fullscreen quad (positions [-1,1]) |
| `fn circle(radius: f32, segments: u32) -> Self` | Generate a filled circle |
| `fn polygon(radius: f32, sides: u32) -> Self` | Generate a regular polygon |
| `fn ring(inner_radius: f32, outer_radius: f32, segments: u32) -> Self` | Generate a ring |
| `fn rect(width: f32, height: f32) -> Self` | Generate a rectangle |
| `fn attach_custom_attr(&mut self, attr: CustomATTR)` | Add custom attribute |
| `fn starts_with_custom(&self) -> bool` | Does the first layout start with a custom attribute? |
| `fn ship(&self) -> MeshHandle` | Upload to GPU and get a handle |

### TextureFile

```rust
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
| `fn pixel_count(&self) -> usize` | Total pixels |
| `fn set_wrap(&mut self, wrap: ImgWrap)` | Set wrap mode |
| `fn set_filter(&mut self, filter: ImgFilter)` | Set filter mode |
| `fn ship(&self) -> Texture2D` | Upload to GPU |
| `fn fallback() -> OpticResult<Self>` | Create checkerboard fallback texture |
| `fn from_disk(path: &str) -> OpticResult<Self>` | Load from disk (debug: loads source + overwrites cache; release: loads cache only) |
| `fn save_cached(&self, path: &str) -> OpticResult<()>` | Save to binary cache (`.otxtr`) |

### Attribute Types

```rust
pub struct Pos3DATTR { pub data: Vec<[f32; 3]>, pub info: ATTRInfo }
pub struct Pos2DATTR { pub data: Vec<[f32; 2]>, pub info: ATTRInfo }
pub struct ColATTR   { pub data: Vec<[f32; 4]>, pub info: ATTRInfo }
pub struct UVMATTR   { pub data: Vec<[f32; 2]>, pub info: ATTRInfo }
pub struct NrmATTR   { pub data: Vec<[f32; 3]>, pub info: ATTRInfo }
pub struct IndATTR   { pub data: Vec<u32>,      pub info: ATTRInfo }
pub struct CustomATTR {
    pub data: Vec<u8>,
    pub info: ATTRInfo,
}
```

All `*ATTR` structs share these methods via macro:

| Signature | Description |
|-----------|-------------|
| `fn empty() -> Self` | Empty attribute |
| `fn from(vec: Vec<T>) -> Self` | From a Vec |
| `fn from_array(array: &[T]) -> Self` | From a slice |
| `fn push(&mut self, elem: T)` | Add one element |
| `fn is_empty(&self) -> bool` | No data? |

`CustomATTR` additionally has generic methods:

| Signature | Description |
|-----------|-------------|
| `fn empty<D: DataType>(name: &str) -> Self` | Empty custom attribute with name |
| `fn from<D: DataType>(name: &str, vec: Vec<D>) -> Self` | From typed Vec |
| `fn from_array<D: DataType + Clone>(name: &str, array: &[D]) -> Self` | From typed slice |
| `fn push<D: DataType>(&mut self, elem: D)` | Push a typed element |

#### ATTRInfo

```rust
pub struct ATTRInfo {
    pub name: ATTRName,
    pub typ: ATTRType,
    pub byte_count: usize,
    pub elem_count: usize,
}
```

| Signature | Description |
|-----------|-------------|
| `fn empty() -> Self` | Zeroed info |
| `fn fmt_as_string(&self) -> String` | Formatted as `"{name}:{typ}:{byte_count}:{elem_count}"` |

#### ATTRName

```rust
pub enum ATTRName {
    Custom(String),
    Pos2D, Pos3D, Col, UVM, Nrm, Ind,
}
```

| Signature | Description |
|-----------|-------------|
| `fn as_string(&self) -> String` | Name as string |

### DataType Trait

```rust
pub trait DataType {
    const ATTR_FORMAT: ATTRType;
    const BYTE_COUNT: usize;
    const ELEM_COUNT: usize;
    fn u8ify(&self) -> Vec<u8>;
}
```

Implemented for: `i8`, `u8`, `i16`, `u16`, `i32`, `u32`, `f32`, `f64` (scalar) and `[T; 2]`, `[T; 3]`, `[T; 4]` for each.

### Center

```rust
pub enum Center {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Middle,
    Custom(f32, f32),
}
```

---

## 6. Game Loop (`optic_loop`)

### Runtime Trait

```rust
pub trait Runtime {
    fn start(&mut self, game: &mut Game);
    fn update(&mut self, game: &mut Game);
    fn end(&mut self, game: &mut Game);
}
```

Implement this trait to define your application's lifecycle.

### Game

```rust
pub struct Game {
    pub renderer: GPU,
    pub camera: Camera,
    pub events: Events,
    pub time: Time,
    pub window: Window,
}
```

| Signature | Description |
|-----------|-------------|
| `fn new<R: Runtime + 'static>(runtime: R) -> OpticResult<Game>` | Initialize everything |
| `fn run(mut self)` | Start the main event loop (blocking) |
| `fn exit(&mut self)` | Request exit on next frame |

### Time

```rust
pub struct Time {
    pub fps: f64,
    pub delta: f64,
    pub tick_count: u64,
    pub elapsed: f64,
    pub start_time: Instant,
    pub prev_time: Instant,
    pub prev_sec: Instant,
    pub local_tick: u32,
}
```

| Signature | Description |
|-----------|-------------|
| `fn new() -> Self` | Create timer |
| `fn update(&mut self)` | Advance time (called automatically by Game) |
| `fn fps(&self) -> f64` | Frames per second (smoothed) |
| `fn delta(&self) -> f64` | Time since last frame (seconds) |
| `fn elapsed(&self) -> f64` | Time since construction |
| `fn now(&self) -> f64` | Seconds since start |
| `fn now_ms(&self) -> u64` | Milliseconds since start |
| `fn now_as_ms(&self) -> u64` | Alias for `now_ms` |
| `fn now_as_ns(&self) -> u64` | Nanoseconds since start |
| `fn sleep(&self, secs: f64)` | Sleep for seconds |
| `fn sleep_ms(&self, millis: u64)` | Sleep for milliseconds |
| `fn sleep_ns(&self, nanos: u64)` | Sleep for nanoseconds |

### FrameState

```rust
pub struct FrameState<'a> {
    pub time: &'a Time,
    pub windows: &'a mut [WindowState],
    pub gpu: &'a mut GPU,
    pub camera: &'a mut Camera,
}
```

### WindowState

```rust
pub struct WindowState {
    pub window: Window,
    pub events: Events,
}
```

| Signature | Description |
|-----------|-------------|
| `fn new(el: &EventLoop<()>, title: &str, size: Size2D) -> Self` | Create window state |
| `fn close(&mut self)` | Close the window |
| `fn is_closed(&self) -> bool` | Check closed |
| `fn surface_index(&self) -> usize` | Surface index for renderer |

### GameLoop

```rust
pub struct GameLoop<F: FnMut(&mut FrameState)> { /* fields */ }
```

| Signature | Description |
|-----------|-------------|
| `fn new(el, gpu, camera, windows: Vec<WindowState>, frame_fn: F) -> OpticResult<Self>` | Create multi-window game loop |
| `fn run(mut self)` | Start the event loop |

### Standalone `run()`

```rust
pub fn run<F>(title: &str, size: Size2D, frame_fn: F)
where
    F: FnMut(&mut FrameState) + 'static;
```

Convenience function for single-window applications. Errors are logged via `log_error!` and the process exits with code 1 on failure.

---

## 7. File Utilities (`optic_file`)

All functions are free functions in the `optic_file` crate:

| Signature | Description |
|-----------|-------------|
| `fn name(path: &str) -> Option<String>` | File stem (name without extension) |
| `fn extension(path: &str) -> Option<String>` | File extension |
| `fn exists(path: &str) -> bool` | Check if path exists |
| `fn read_bytes(path: &str) -> OpticResult<Vec<u8>>` | Read file as bytes |
| `fn read_string(path: &str) -> OpticResult<String>` | Read file as UTF-8 string |
| `fn write_bytes(path: &str, data: &[u8]) -> OpticResult<()>` | Write bytes (creates directories) |
| `fn write_string(path: &str, data: &str) -> OpticResult<()>` | Write string |
| `fn cached_path(source: &str, ext: &str) -> String` | Generate cache path: `{dir}/optc/{stem}.{ext}` |
| `fn create_dir(path: &str) -> OpticResult<()>` | Create directory recursively |
