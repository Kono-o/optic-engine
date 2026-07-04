# optic-color

Colour types and conversions for the Optic engine.

Provides `RGBA`, `RGB`, `HSV`, `HSL` with conversions between each other,
sRGB/linear gamma transfer, hex parsing, lighten/darken/saturate, and
colour gradients.

## Key types

| Type | Description |
|---|---|
| [`RGBA`] | Red-green-blue-alpha (0–1 f32) |
| [`RGB`] | RGB without alpha |
| [`HSV`] | Hue-saturation-value |
| [`HSL`] | Hue-saturation-lightness |
| [`Gradient`] | Multi-stop colour ramp |

```rust
use optic_color::RGBA;

let red = RGBA::new(1.0, 0.0, 0.0, 1.0);
let hex = RGBA::from_hex_u32(0xFF8800FF);
let lit = red.lighten(0.2);
```

[`RGBA`]: https://docs.rs/optic-color/latest/optic_color/struct.RGBA.html
[`RGB`]: https://docs.rs/optic-color/latest/optic_color/struct.RGB.html
[`HSV`]: https://docs.rs/optic-color/latest/optic_color/struct.HSV.html
[`HSL`]: https://docs.rs/optic-color/latest/optic_color/struct.HSL.html
[`Gradient`]: https://docs.rs/optic-color/latest/optic_color/gradient/struct.Gradient.html
