use cgmath::*;

/// 2D affine transform (position, rotation, scale) used by Mesh2D and Text2D.
///
/// Stores position in normalised screen coordinates, rotation in degrees around the Z
/// axis, non-uniform scale, draw-order layer, and aspect ratio. The engine uses
/// `Transform2D` as the spatial component of every 2D mesh and text object — call
/// [`calc_matrix`](Transform2D::calc_matrix) after mutating to recompute the 4×4 matrix
/// consumed by the rendering pipeline.
///
/// # Coordinate system
///
/// - **Position** — normalized space: `(0, 0)` is bottom-left, `(1, 1)` is
///   top-right.
/// - **Rotation** — degrees around the Z axis (counter-clockwise).
/// - **Layer** — draw order: higher values are rendered on top.
///
/// # Operations
///
/// | Category | Methods |
/// |---|---|
/// | **Position** — getter | [`pos`](Transform2D::pos) |
    /// | **Position** — absolute setter | [`set_position`](Transform2D::set_position), [`set_position_x`](Transform2D::set_position_x), [`set_position_y`](Transform2D::set_position_y) |
    /// | **Position** — relative move | [`translate`](Transform2D::translate), [`translate_x`](Transform2D::translate_x), [`translate_y`](Transform2D::translate_y) |
    /// | **Rotation** — getter/setter | [`rotation`](Transform2D::rotation), [`set_rotation`](Transform2D::set_rotation), [`rotate`](Transform2D::rotate) |
    /// | **Scale** — getter | [`scale_factor`](Transform2D::scale_factor) |
    /// | **Scale** — absolute setter | [`set_scale`](Transform2D::set_scale), [`set_scale_uniform`](Transform2D::set_scale_uniform), [`set_scale_x`](Transform2D::set_scale_x), [`set_scale_y`](Transform2D::set_scale_y) |
    /// | **Scale** — relative add | [`scale`](Transform2D::scale), [`scale_uniform`](Transform2D::scale_uniform), [`scale_x`](Transform2D::scale_x), [`scale_y`](Transform2D::scale_y) |
/// | **Layer** | [`layer`](Transform2D::layer), [`set_layer`](Transform2D::set_layer) |
/// | **Aspect** | [`aspect`](Transform2D::aspect), [`set_aspect`](Transform2D::set_aspect) |
/// | **Matrix** | [`matrix`](Transform2D::matrix), [`calc_matrix`](Transform2D::calc_matrix) |
#[derive(Clone, Debug)]
pub struct Transform2D {
    matrix: Matrix4<f32>,
    pos: Vector2<f32>,
    rot: f32,
    layer: u8,
    aspect: f32,
    scale: Vector2<f32>,
}

impl Default for Transform2D {
    fn default() -> Self {
        Self {
            matrix: Matrix4::identity(),
            pos: Vector2::new(0.0, 0.0),
            rot: 0.0,
            layer: 0,
            aspect: 1.0,
            scale: Vector2::new(1.0, 1.0),
        }
    }
}

impl Transform2D {
    fn calc_pos_matrix(&self) -> Matrix4<f32> {
        let p = self.pos * 2.0;
        let v = vec3((p.x - 1.0) * self.aspect, p.y - 1.0, 0.0);
        Matrix4::from_translation(v)
    }

    fn calc_rot_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_angle_z(Rad::from(Deg(self.rot)))
    }

    fn calc_scale_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0)
    }

    /// Recomputes the transformation matrix from the current pos/rot/scale.
    pub fn calc_matrix(&mut self) {
        self.matrix = self.calc_pos_matrix() * self.calc_rot_matrix() * self.calc_scale_matrix();
    }

    /// Returns the aspect ratio (width / height).
    pub fn aspect(&self) -> f32 { self.aspect }
    /// Sets the aspect ratio.
    pub fn set_aspect(&mut self, aspect: f32) { self.aspect = aspect; }
    /// Returns the position in normalized coordinates.
    pub fn pos(&self) -> Vector2<f32> { self.pos }
    /// Returns the rotation in degrees.
    pub fn rotation(&self) -> f32 { self.rot }
    /// Returns the layer (draw order).
    pub fn layer(&self) -> u8 { self.layer }
    /// Returns the scale factor.
    pub fn scale_factor(&self) -> Vector2<f32> { self.scale }
    /// Returns the cached 4×4 transformation matrix.
    pub fn matrix(&self) -> Matrix4<f32> { self.matrix }

    /// Translates by `(x, y)` in normalized coordinates.
    pub fn translate(&mut self, x: f32, y: f32) { self.pos += vec2(x, y); }
    /// Translates along the X axis.
    pub fn translate_x(&mut self, x: f32) { self.pos.x += x; }
    /// Translates along the Y axis.
    pub fn translate_y(&mut self, y: f32) { self.pos.y += y; }
    /// Sets the position to `(x, y)`.
    pub fn set_position(&mut self, x: f32, y: f32) { self.pos = vec2(x, y); }
    /// Sets the X coordinate.
    pub fn set_position_x(&mut self, x: f32) { self.pos.x = x; }
    /// Sets the Y coordinate.
    pub fn set_position_y(&mut self, y: f32) { self.pos.y = y; }

    /// Adds `rot` degrees to the current rotation.
    pub fn rotate(&mut self, rot: f32) { self.rot += rot; }
    /// Sets the rotation to `rot` degrees.
    pub fn set_rotation(&mut self, rot: f32) { self.rot = rot; }
    /// Sets the layer.
    pub fn set_layer(&mut self, layer: u8) { self.layer = layer; }

    /// Adds `(x, y)` to the current scale.
    pub fn scale(&mut self, x: f32, y: f32) { self.scale += vec2(x, y); }
    /// Adds `xy` to both scale components.
    pub fn scale_uniform(&mut self, xy: f32) { self.scale(xy, xy); }
    /// Adds `x` to the scale X component.
    pub fn scale_x(&mut self, x: f32) { self.scale.x += x; }
    /// Adds `y` to the scale Y component.
    pub fn scale_y(&mut self, y: f32) { self.scale.y += y; }
    /// Sets the scale to `(x, y)`.
    pub fn set_scale(&mut self, x: f32, y: f32) { self.scale = vec2(x, y); }
    /// Sets both scale components to `xy`.
    pub fn set_scale_uniform(&mut self, xy: f32) { self.set_scale(xy, xy); }
    /// Sets the scale X component.
    pub fn set_scale_x(&mut self, x: f32) { self.scale.x = x; }
    /// Sets the scale Y component.
    pub fn set_scale_y(&mut self, y: f32) { self.scale.y = y; }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn transform2d_default() {
        let t = Transform2D::default();
        assert_eq!(t.pos(), vec2(0.0, 0.0));
        assert_eq!(t.rotation(), 0.0);
        assert_eq!(t.scale_factor(), vec2(1.0, 1.0));
    }

    #[test]
    fn transform2d_set_pos() {
        let mut t = Transform2D::default();
        t.set_position(100.0, 200.0);
        assert_eq!(t.pos(), vec2(100.0, 200.0));
        t.set_position_x(50.0);
        t.set_position_y(150.0);
        assert_eq!(t.pos(), vec2(50.0, 150.0));
    }

    #[test]
    fn transform2d_move() {
        let mut t = Transform2D::default();
        t.translate(10.0, 20.0);
        assert_eq!(t.pos(), vec2(10.0, 20.0));
        t.translate_x(5.0);
        t.translate_y(3.0);
        assert_eq!(t.pos(), vec2(15.0, 23.0));
    }

    #[test]
    fn transform2d_rotate() {
        let mut t = Transform2D::default();
        t.rotate(90.0);
        assert!(approx_eq(t.rotation(), 90.0));
        t.set_rotation(45.0);
        assert!(approx_eq(t.rotation(), 45.0));
    }

    #[test]
    fn transform2d_scale() {
        let mut t = Transform2D::default();
        t.set_scale(2.0, 3.0);
        assert_eq!(t.scale_factor(), vec2(2.0, 3.0));
        t.set_scale_x(5.0);
        t.set_scale_y(6.0);
        assert_eq!(t.scale_factor(), vec2(5.0, 6.0));
    }

    #[test]
    fn transform2d_scale_operations() {
        let mut t = Transform2D::default();
        t.scale(1.0, 2.0);
        assert_eq!(t.scale_factor(), vec2(2.0, 3.0));
        t.scale_uniform(3.0);
        assert_eq!(t.scale_factor(), vec2(5.0, 6.0));
        t.scale_x(1.0);
        t.scale_y(1.0);
        assert_eq!(t.scale_factor(), vec2(6.0, 7.0));
    }

    #[test]
    fn transform2d_layer() {
        let mut t = Transform2D::default();
        assert_eq!(t.layer(), 0);
        t.set_layer(5);
        assert_eq!(t.layer(), 5);
    }

    #[test]
    fn transform2d_matrix() {
        let mut t = Transform2D::default();
        t.calc_matrix();
        let m = t.matrix();
        assert!(approx_eq(m[0][0], 1.0));
        assert!(approx_eq(m[1][1], 1.0));
        assert!(approx_eq(m[2][2], 1.0));
    }

    #[test]
    fn transform2d_matrix_translation() {
        let mut t = Transform2D::default();
        t.set_position(0.5, 0.0);
        t.calc_matrix();
        let m = t.matrix();
        assert!(approx_eq(m[3][0], (0.5 * 2.0 - 1.0) * 1.0));
        assert!(approx_eq(m[3][1], 0.0 * 2.0 - 1.0));
    }

    #[test]
    fn transform2d_aspect() {
        let mut t = Transform2D::default();
        assert!(approx_eq(t.aspect(), 1.0));
        t.set_aspect(1.5);
        assert!(approx_eq(t.aspect(), 1.5));
    }
}
