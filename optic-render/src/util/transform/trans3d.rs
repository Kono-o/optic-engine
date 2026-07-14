use cgmath::*;

/// 3D transform (position, rotation, scale) used by Mesh3D and Text3D.
///
/// Stores world-space position, Euler rotation in degrees, and non-uniform scale,
/// maintaining a 4×4 TRS (translate × rotate × scale) matrix. The engine uses
/// `Transform3D` as the spatial component of every 3D mesh and text object — call
/// [`calc_matrix`](Transform3D::calc_matrix) after mutating to recompute the matrix
/// consumed by the rendering pipeline.
///
/// # Operations
///
/// | Category | Methods |
/// |---|---|
/// | **Position** — getter | [`pos`](Transform3D::pos) |
    /// | **Position** — absolute setter | [`set_position`](Transform3D::set_position), [`set_position_x`](Transform3D::set_position_x), [`set_position_y`](Transform3D::set_position_y), [`set_position_z`](Transform3D::set_position_z) |
    /// | **Position** — relative move | [`translate`](Transform3D::translate), [`translate_x`](Transform3D::translate_x), [`translate_y`](Transform3D::translate_y), [`translate_z`](Transform3D::translate_z) |
    /// | **Rotation** — getter | [`rotation`](Transform3D::rotation) |
    /// | **Rotation** — absolute setter | [`set_rotation`](Transform3D::set_rotation), [`set_rotation_x`](Transform3D::set_rotation_x), [`set_rotation_y`](Transform3D::set_rotation_y), [`set_rotation_z`](Transform3D::set_rotation_z) |
    /// | **Rotation** — relative add | [`rotate`](Transform3D::rotate), [`rotate_x`](Transform3D::rotate_x), [`rotate_y`](Transform3D::rotate_y), [`rotate_z`](Transform3D::rotate_z) |
    /// | **Scale** — getter | [`scale_factor`](Transform3D::scale_factor) |
    /// | **Scale** — absolute setter | [`set_scale`](Transform3D::set_scale), [`set_scale_uniform`](Transform3D::set_scale_uniform), [`set_scale_x`](Transform3D::set_scale_x), [`set_scale_y`](Transform3D::set_scale_y), [`set_scale_z`](Transform3D::set_scale_z) |
    /// | **Scale** — relative add | [`scale`](Transform3D::scale), [`scale_uniform`](Transform3D::scale_uniform), [`scale_x`](Transform3D::scale_x), [`scale_y`](Transform3D::scale_y), [`scale_z`](Transform3D::scale_z) |
/// | **Matrix** | [`matrix`](Transform3D::matrix), [`calc_matrix`](Transform3D::calc_matrix) |
///
/// # Example
///
/// ```ignore
/// use optic_render::Transform3D;
///
/// let mut t = Transform3D::default();
/// t.set_position(10.0, 0.0, 0.0);
/// t.rotate_y(90.0);
/// t.calc_matrix();
/// ```
#[derive(Clone, Debug)]
pub struct Transform3D {
    matrix: Matrix4<f32>,
    pos: Vector3<f32>,
    rot: Vector3<f32>,
    scale: Vector3<f32>,
}

impl Default for Transform3D {
    fn default() -> Self {
        Self {
            matrix: Matrix4::identity(),
            pos: Vector3::new(0.0, 0.0, 0.0),
            rot: Vector3::new(0.0, 0.0, 0.0),
            scale: Vector3::new(1.0, 1.0, 1.0),
        }
    }
}

impl Transform3D {
    fn calc_pos_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_translation(self.pos)
    }

    fn calc_rot_matrix(&self) -> Matrix4<f32> {
        let x = Matrix4::from_angle_x(Rad::from(Deg(self.rot.x)));
        let y = Matrix4::from_angle_y(Rad::from(Deg(self.rot.y)));
        let z = Matrix4::from_angle_z(Rad::from(Deg(self.rot.z)));
        x * y * z
    }

    fn calc_scale_matrix(&self) -> Matrix4<f32> {
        Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, self.scale.z)
    }

    /// Recomputes the transformation matrix from the current pos/rot/scale.
    pub fn calc_matrix(&mut self) {
        self.matrix = self.calc_pos_matrix() * self.calc_rot_matrix() * self.calc_scale_matrix();
    }

    /// Returns the position.
    pub fn pos(&self) -> Vector3<f32> { self.pos }
    /// Returns the rotation (Euler angles in degrees).
    pub fn rotation(&self) -> Vector3<f32> { self.rot }
    /// Returns the scale.
    pub fn scale_factor(&self) -> Vector3<f32> { self.scale }
    /// Returns the cached 4×4 transformation matrix.
    pub fn matrix(&self) -> Matrix4<f32> { self.matrix }

    /// Translates by `(x, y, z)`.
    pub fn translate(&mut self, x: f32, y: f32, z: f32) { self.pos += vec3(x, y, z); }
    /// Translates along the X axis.
    pub fn translate_x(&mut self, x: f32) { self.pos.x += x; }
    /// Translates along the Y axis.
    pub fn translate_y(&mut self, y: f32) { self.pos.y += y; }
    /// Translates along the Z axis.
    pub fn translate_z(&mut self, z: f32) { self.pos.z += z; }
    /// Sets the position to `(x, y, z)`.
    pub fn set_position(&mut self, x: f32, y: f32, z: f32) { self.pos = vec3(x, y, z); }
    /// Sets the X coordinate.
    pub fn set_position_x(&mut self, x: f32) { self.pos.x = x; }
    /// Sets the Y coordinate.
    pub fn set_position_y(&mut self, y: f32) { self.pos.y = y; }
    /// Sets the Z coordinate.
    pub fn set_position_z(&mut self, z: f32) { self.pos.z = z; }

    /// Rotates by `(x, y, z)` degrees.
    pub fn rotate(&mut self, x: f32, y: f32, z: f32) { self.rot += vec3(x, y, z); }
    /// Rotates around the X axis.
    pub fn rotate_x(&mut self, x: f32) { self.rot.x += x; }
    /// Rotates around the Y axis.
    pub fn rotate_y(&mut self, y: f32) { self.rot.y += y; }
    /// Rotates around the Z axis.
    pub fn rotate_z(&mut self, z: f32) { self.rot.z += z; }
    /// Sets the rotation to `(x, y, z)` degrees.
    pub fn set_rotation(&mut self, x: f32, y: f32, z: f32) { self.rot = vec3(x, y, z); }
    /// Sets the X rotation.
    pub fn set_rotation_x(&mut self, x: f32) { self.rot.x = x; }
    /// Sets the Y rotation.
    pub fn set_rotation_y(&mut self, y: f32) { self.rot.y = y; }
    /// Sets the Z rotation.
    pub fn set_rotation_z(&mut self, z: f32) { self.rot.z = z; }

    /// Adds `(x, y, z)` to the scale.
    pub fn scale(&mut self, x: f32, y: f32, z: f32) { self.scale += vec3(x, y, z); }
    /// Adds `xyz` to all three scale components.
    pub fn scale_uniform(&mut self, xyz: f32) { self.scale(xyz, xyz, xyz); }
    /// Adds `x` to the scale X component.
    pub fn scale_x(&mut self, x: f32) { self.scale.x += x; }
    /// Adds `y` to the scale Y component.
    pub fn scale_y(&mut self, y: f32) { self.scale.y += y; }
    /// Adds `z` to the scale Z component.
    pub fn scale_z(&mut self, z: f32) { self.scale.z += z; }
    /// Sets the scale to `(x, y, z)`.
    pub fn set_scale(&mut self, x: f32, y: f32, z: f32) { self.scale = vec3(x, y, z); }
    /// Sets all three scale components to `xyz`.
    pub fn set_scale_uniform(&mut self, xyz: f32) { self.set_scale(xyz, xyz, xyz); }
    /// Sets the scale X component.
    pub fn set_scale_x(&mut self, x: f32) { self.scale.x = x; }
    /// Sets the scale Y component.
    pub fn set_scale_y(&mut self, y: f32) { self.scale.y = y; }
    /// Sets the scale Z component.
    pub fn set_scale_z(&mut self, z: f32) { self.scale.z = z; }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    fn mat_approx_eq(m1: &Matrix4<f32>, m2: &Matrix4<f32>) -> bool {
        for c in 0..4 {
            for r in 0..4 {
                if !approx_eq(m1[c][r], m2[c][r]) {
                    return false;
                }
            }
        }
        true
    }

    fn is_identity(m: &Matrix4<f32>) -> bool {
        mat_approx_eq(m, &Matrix4::identity())
    }

    #[test]
    fn transform3d_default() {
        let t = Transform3D::default();
        assert_eq!(t.pos(), vec3(0.0, 0.0, 0.0));
        assert_eq!(t.rotation(), vec3(0.0, 0.0, 0.0));
        assert_eq!(t.scale_factor(), vec3(1.0, 1.0, 1.0));
        assert!(is_identity(&t.matrix()));
    }

    #[test]
    fn transform3d_set_pos() {
        let mut t = Transform3D::default();
        t.set_position(10.0, 20.0, 30.0);
        assert_eq!(t.pos(), vec3(10.0, 20.0, 30.0));
    }

    #[test]
    fn transform3d_move() {
        let mut t = Transform3D::default();
        t.translate(1.0, 2.0, 3.0);
        assert_eq!(t.pos(), vec3(1.0, 2.0, 3.0));
        t.translate_x(10.0);
        assert_eq!(t.pos().x, 11.0);
        t.translate_y(20.0);
        assert_eq!(t.pos().y, 22.0);
        t.translate_z(30.0);
        assert_eq!(t.pos().z, 33.0);
    }

    #[test]
    fn transform3d_rotate() {
        let mut t = Transform3D::default();
        t.rotate(90.0, 0.0, 0.0);
        assert_eq!(t.rotation(), vec3(90.0, 0.0, 0.0));
        t.rotate_x(45.0);
        assert!(approx_eq(t.rotation().x, 135.0));
        t.rotate_y(30.0);
        assert!(approx_eq(t.rotation().y, 30.0));
        t.rotate_z(60.0);
        assert!(approx_eq(t.rotation().z, 60.0));
    }

    #[test]
    fn transform3d_set_rotation() {
        let mut t = Transform3D::default();
        t.set_rotation(45.0, 90.0, 180.0);
        assert_eq!(t.rotation(), vec3(45.0, 90.0, 180.0));
        t.set_rotation_x(10.0);
        t.set_rotation_y(20.0);
        t.set_rotation_z(30.0);
        assert_eq!(t.rotation(), vec3(10.0, 20.0, 30.0));
    }

    #[test]
    fn transform3d_scale() {
        let mut t = Transform3D::default();
        t.set_scale(2.0, 3.0, 4.0);
        assert_eq!(t.scale_factor(), vec3(2.0, 3.0, 4.0));
    }

    #[test]
    fn transform3d_scale_operations() {
        let mut t = Transform3D::default();
        t.scale(1.0, 2.0, 3.0);
        assert_eq!(t.scale_factor(), vec3(2.0, 3.0, 4.0));
        t.scale_uniform(5.0);
        assert_eq!(t.scale_factor(), vec3(7.0, 8.0, 9.0));
        t.scale_x(1.0);
        t.scale_y(1.0);
        t.scale_z(1.0);
        assert_eq!(t.scale_factor(), vec3(8.0, 9.0, 10.0));
    }

    #[test]
    fn transform3d_set_scale_individual() {
        let mut t = Transform3D::default();
        t.set_scale_x(5.0);
        t.set_scale_y(6.0);
        t.set_scale_z(7.0);
        assert_eq!(t.scale_factor(), vec3(5.0, 6.0, 7.0));
    }

    #[test]
    fn transform3d_calc_matrix() {
        let mut t = Transform3D::default();
        // identity -> identity
        t.calc_matrix();
        assert!(is_identity(&t.matrix()));

        // translation
        t.set_position(1.0, 2.0, 3.0);
        t.calc_matrix();
        let m = t.matrix();
        assert!(approx_eq(m[3][0], 1.0));
        assert!(approx_eq(m[3][1], 2.0));
        assert!(approx_eq(m[3][2], 3.0));
    }

    #[test]
    fn transform3d_matrix_combines() {
        let mut t = Transform3D::default();
        t.set_position(10.0, 0.0, 0.0);
        t.set_scale(2.0, 1.0, 1.0);
        t.calc_matrix();
        let m = t.matrix();
        // translation x is 10, scale x is 2
        assert!(approx_eq(m[0][0], 2.0));
        assert!(approx_eq(m[3][0], 10.0));
    }

    #[test]
    fn transform3d_set_scale_uniform() {
        let mut t = Transform3D::default();
        t.set_scale_uniform(3.0);
        assert_eq!(t.scale_factor(), vec3(3.0, 3.0, 3.0));
    }
}
