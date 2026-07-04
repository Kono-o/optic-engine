# optic-render

OpenGL 4.6 renderer for the Optic engine — EGL context, shaders, meshes,
textures, cameras, and canvas.

The largest Optic crate. Manages GPU resources through typed handles
([`Mesh3D`], [`Mesh2D`], [`Shader`], [`Texture2D`], [`Canvas`], [`InstanceBuffer`])
and provides CPU-side asset types ([`Mesh3DFile`], [`TextureFile`], [`ShaderFile`])
that can be loaded from disk and shipped to the GPU.

```rust
use optic_render::{Context, Camera, Transform3D};
use optic_core::{Size2D, CamProj};

let ctx = Context::new_headless()?;
let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
```

[`Mesh3D`]: https://docs.rs/optic-render/latest/optic_render/handles/mesh/struct.Mesh3D.html
[`Mesh2D`]: https://docs.rs/optic-render/latest/optic_render/handles/mesh/struct.Mesh2D.html
[`Shader`]: https://docs.rs/optic-render/latest/optic_render/handles/shader/struct.Shader.html
[`Texture2D`]: https://docs.rs/optic-render/latest/optic_render/handles/texture/struct.Texture2D.html
[`Canvas`]: https://docs.rs/optic-render/latest/optic_render/handles/canvas/struct.Canvas.html
[`InstanceBuffer`]: https://docs.rs/optic-render/latest/optic_render/handles/instance/struct.InstanceBuffer.html
[`Mesh3DFile`]: https://docs.rs/optic-render/latest/optic_render/asset/msh/struct.Mesh3DFile.html
[`TextureFile`]: https://docs.rs/optic-render/latest/optic_render/asset/img/struct.TextureFile.html
[`ShaderFile`]: https://docs.rs/optic-render/latest/optic_render/asset/shdr/struct.ShaderFile.html
[`Context`]: https://docs.rs/optic-render/latest/optic_render/context/struct.Context.html
[`Camera`]: https://docs.rs/optic-render/latest/optic_render/camera/camera/struct.Camera.html
[`Transform3D`]: https://docs.rs/optic-render/latest/optic_render/util/transform/trans3d/struct.Transform3D.html
