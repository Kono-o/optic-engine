# optic-core

Shared types, geometry, errors, and utilities for the Optic engine.

Reused by every other Optic crate. Defines the common vocabulary: coordinates,
sizes, clip distances, draw modes, image formats, ANSI console colours,
logging macros, and error types.

## Key types

| Type | Description |
|---|---|
| [`Size2D`] / [`Size3D`] | Integer width/height/depth |
| [`Coord2D`] / [`CoordOffset`] | Floating-point coordinates and deltas |
| [`ClipDist`] | Near/far clip planes |
| [`CamProj`] | Perspective vs orthographic |
| [`OpticError`] / [`OpticResult`] | Error type with string messages |
| [`ANSI`] | Terminal colour wrappers |
| [`DrawMode`], [`ImgFormat`], [`PolyMode`], [`Cull`], [`ImgFilter`], [`ImgWrap`] | Rendering enums |
| [`Log`] | Console logging macros |

```rust
use optic_core::{Size2D, Coord2D, OpticResult};

let size = Size2D::from(1920, 1080);
let pos = Coord2D::from(100.5, 200.3);
```

[`Size2D`]: https://docs.rs/optic-core/latest/optic_core/geometry/struct.Size2D.html
[`Size3D`]: https://docs.rs/optic-core/latest/optic_core/geometry/struct.Size3D.html
[`Coord2D`]: https://docs.rs/optic-core/latest/optic_core/coord/struct.Coord2D.html
[`CoordOffset`]: https://docs.rs/optic-core/latest/optic_core/coord/struct.CoordOffset.html
[`ClipDist`]: https://docs.rs/optic-core/latest/optic_core/geometry/struct.ClipDist.html
[`CamProj`]: https://docs.rs/optic-core/latest/optic_core/geometry/enum.CamProj.html
[`OpticError`]: https://docs.rs/optic-core/latest/optic_core/error/struct.OpticError.html
[`OpticResult`]: https://docs.rs/optic-core/latest/optic_core/error/type.OpticResult.html
[`ANSI`]: https://docs.rs/optic-core/latest/optic_core/ansi/struct.ANSI.html
