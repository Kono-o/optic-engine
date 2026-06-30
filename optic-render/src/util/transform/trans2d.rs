use cgmath::*;

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

    pub fn calc_matrix(&mut self) {
        self.matrix = self.calc_pos_matrix() * self.calc_rot_matrix() * self.calc_scale_matrix();
    }

    pub fn aspect(&self) -> f32 { self.aspect }
    pub fn set_aspect(&mut self, aspect: f32) { self.aspect = aspect; }
    pub fn pos(&self) -> Vector2<f32> { self.pos }
    pub fn rot(&self) -> f32 { self.rot }
    pub fn layer(&self) -> u8 { self.layer }
    pub fn scale(&self) -> Vector2<f32> { self.scale }
    pub fn matrix(&self) -> Matrix4<f32> { self.matrix }

    pub fn move_all(&mut self, x: f32, y: f32) { self.pos += vec2(x, y); }
    pub fn move_x(&mut self, x: f32) { self.pos.x += x; }
    pub fn move_y(&mut self, y: f32) { self.pos.y += y; }
    pub fn set_pos_all(&mut self, x: f32, y: f32) { self.pos = vec2(x, y); }
    pub fn set_pos_x(&mut self, x: f32) { self.pos.x = x; }
    pub fn set_pos_y(&mut self, y: f32) { self.pos.y = y; }

    pub fn rotate(&mut self, rot: f32) { self.rot += rot; }
    pub fn set_rot(&mut self, rot: f32) { self.rot = rot; }
    pub fn set_layer(&mut self, layer: u8) { self.layer = layer; }

    pub fn scale_all(&mut self, x: f32, y: f32) { self.scale += vec2(x, y); }
    pub fn scale_same(&mut self, xy: f32) { self.scale_all(xy, xy); }
    pub fn scale_x(&mut self, x: f32) { self.scale.x += x; }
    pub fn scale_y(&mut self, y: f32) { self.scale.y += y; }
    pub fn set_scale_all(&mut self, x: f32, y: f32) { self.scale = vec2(x, y); }
    pub fn set_scale_same(&mut self, xy: f32) { self.set_scale_all(xy, xy); }
    pub fn set_scale_x(&mut self, x: f32) { self.scale.x = x; }
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
        assert_eq!(t.rot(), 0.0);
        assert_eq!(t.scale(), vec2(1.0, 1.0));
    }

    #[test]
    fn transform2d_set_pos() {
        let mut t = Transform2D::default();
        t.set_pos_all(100.0, 200.0);
        assert_eq!(t.pos(), vec2(100.0, 200.0));
        t.set_pos_x(50.0);
        t.set_pos_y(150.0);
        assert_eq!(t.pos(), vec2(50.0, 150.0));
    }

    #[test]
    fn transform2d_move() {
        let mut t = Transform2D::default();
        t.move_all(10.0, 20.0);
        assert_eq!(t.pos(), vec2(10.0, 20.0));
        t.move_x(5.0);
        t.move_y(3.0);
        assert_eq!(t.pos(), vec2(15.0, 23.0));
    }

    #[test]
    fn transform2d_rotate() {
        let mut t = Transform2D::default();
        t.rotate(90.0);
        assert!(approx_eq(t.rot(), 90.0));
        t.set_rot(45.0);
        assert!(approx_eq(t.rot(), 45.0));
    }

    #[test]
    fn transform2d_scale() {
        let mut t = Transform2D::default();
        t.set_scale_all(2.0, 3.0);
        assert_eq!(t.scale(), vec2(2.0, 3.0));
        t.set_scale_x(5.0);
        t.set_scale_y(6.0);
        assert_eq!(t.scale(), vec2(5.0, 6.0));
    }

    #[test]
    fn transform2d_scale_operations() {
        let mut t = Transform2D::default();
        t.scale_all(1.0, 2.0);
        assert_eq!(t.scale(), vec2(2.0, 3.0));
        t.scale_same(3.0);
        assert_eq!(t.scale(), vec2(5.0, 6.0));
        t.scale_x(1.0);
        t.scale_y(1.0);
        assert_eq!(t.scale(), vec2(6.0, 7.0));
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
        t.set_pos_all(0.5, 0.0);
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
