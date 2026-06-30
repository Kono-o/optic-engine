use optic_core::{CamProj, ClipDist, Size2D};
use cgmath::*;

use crate::util::transform::CamTransform;

pub struct Camera {
    pub transform: CamTransform,
}

impl Camera {
    pub fn new(size: Size2D, proj: CamProj) -> Self {
        let fov = 75.0;
        let clip = ClipDist::default();
        let pos = vec3(0.0, 0.0, 5.0);
        let rot = vec3(0.0, -90.0, 0.0);

        let pos_inverse = Matrix4::from_translation(vec3(-pos.x, -pos.y, -pos.z));
        let rot_inverse = Matrix4::<f32>::from_angle_x(Rad::from(Deg(-rot.x)))
            * Matrix4::<f32>::from_angle_y(Rad::from(Deg(-rot.y)))
            * Matrix4::<f32>::from_angle_z(Rad::from(Deg(-rot.z)));

        let view_matrix = pos_inverse * rot_inverse;

        let mut transform = CamTransform {
            pos,
            rot,
            fov,
            clip,
            size,
            proj,
            view_matrix,
            ortho_scale: 2.0,
            front: vec3(0.0, 0.0, -1.0),
            persp_matrix: Matrix4::identity(),
            ortho_matrix: Matrix4::identity(),
        };
        transform.calc_matrices();

        Camera { transform }
    }

    pub fn pre_update(&mut self) {
        self.transform.calc_matrices();
    }

    pub fn fov(&self) -> f32 { self.transform.fov }
    pub fn ortho_scale(&self) -> f32 { self.transform.ortho_scale }
    pub fn proj(&self) -> CamProj { self.transform.proj }
    pub fn clip(&self) -> ClipDist { self.transform.clip }

    pub fn set_clip(&mut self, clip: ClipDist) { self.transform.clip = clip; }
    pub fn set_clip_near(&mut self, near: f32) { self.transform.clip.near = near; }
    pub fn set_clip_far(&mut self, far: f32) { self.transform.clip.far = far; }
    pub fn set_size(&mut self, size: Size2D) { self.transform.size = size; }
    pub fn set_proj(&mut self, proj: CamProj) { self.transform.proj = proj; }

    pub fn set_fov(&mut self, fov: f32) {
        self.transform.fov = fov.max(0.01);
    }
    pub fn add_fov(&mut self, value: f32) {
        self.transform.fov = (self.transform.fov + value).max(0.01);
    }

    pub fn set_ortho_scale(&mut self, value: f32) { self.transform.ortho_scale = value; }
    pub fn add_ortho_scale(&mut self, value: f32) { self.transform.ortho_scale += value; }

    pub fn fly_forw(&mut self, speed: f32) {
        self.transform.pos += speed * self.transform.front;
    }
    pub fn fly_back(&mut self, speed: f32) {
        self.transform.pos -= speed * self.transform.front;
    }
    pub fn fly_left(&mut self, speed: f32) {
        self.transform.pos -= speed * self.transform.front.cross(Vector3::unit_y()).normalize();
    }
    pub fn fly_right(&mut self, speed: f32) {
        self.transform.pos += speed * self.transform.front.cross(Vector3::unit_y()).normalize();
    }
    pub fn fly_up(&mut self, speed: f32) { self.transform.pos.y += speed; }
    pub fn fly_down(&mut self, speed: f32) { self.transform.pos.y -= speed; }

    pub fn spin_x(&mut self, speed: f32) { self.transform.rot.x += speed; }
    pub fn spin_y(&mut self, speed: f32) { self.transform.rot.y += speed; }
    pub fn spin_z(&mut self, speed: f32) { self.transform.rot.z += speed; }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn camera_new_persp() {
        let cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        assert!((cam.fov() - 75.0).abs() < f32::EPSILON);
        assert!((cam.transform.pos.y - 0.0).abs() < f32::EPSILON);
        assert!((cam.transform.pos.z - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn camera_new_ortho() {
        let cam = Camera::new(Size2D::from(800, 600), CamProj::Ortho);
        assert_eq!(cam.proj(), CamProj::Ortho);
    }

    #[test]
    fn camera_set_clip() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        cam.set_clip(ClipDist::from(0.1, 500.0));
        assert!(approx_eq(cam.clip().near, 0.1));
        assert!(approx_eq(cam.clip().far, 500.0));
    }

    #[test]
    fn camera_set_clip_near_far() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        cam.set_clip_near(0.5);
        cam.set_clip_far(2000.0);
        assert!(approx_eq(cam.clip().near, 0.5));
        assert!(approx_eq(cam.clip().far, 2000.0));
    }

    #[test]
    fn camera_set_fov() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        cam.set_fov(90.0);
        assert!(approx_eq(cam.fov(), 90.0));
    }

    #[test]
    fn camera_set_fov_clamped() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        cam.set_fov(-1.0); // clamped to 0.01
        assert!(approx_eq(cam.fov(), 0.01));
    }

    #[test]
    fn camera_add_fov() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        cam.add_fov(10.0);
        assert!(approx_eq(cam.fov(), 85.0));
    }

    #[test]
    fn camera_ortho_scale() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Ortho);
        assert!((cam.ortho_scale() - 2.0).abs() < f32::EPSILON);
        cam.set_ortho_scale(5.0);
        assert!((cam.ortho_scale() - 5.0).abs() < f32::EPSILON);
        cam.add_ortho_scale(3.0);
        assert!((cam.ortho_scale() - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn camera_set_proj() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        assert_eq!(cam.proj(), CamProj::Persp);
        cam.set_proj(CamProj::Ortho);
        assert_eq!(cam.proj(), CamProj::Ortho);
    }

    #[test]
    fn camera_set_size() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        cam.set_size(Size2D::from(800, 600));
        assert_eq!(cam.transform.size.w, 800);
        assert_eq!(cam.transform.size.h, 600);
    }

    #[test]
    fn camera_fly_movements() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        let start = cam.transform.pos;

        cam.fly_forw(1.0);
        // front should be (0, 0, -1) more or less
        let diff = cam.transform.pos - start;
        assert!(approx_eq(diff.z, -1.0));
        assert!(approx_eq(diff.x, 0.0));

        cam.fly_up(1.0);
        assert!(approx_eq(cam.transform.pos.y, 1.0));

        cam.fly_down(0.5);
        assert!(approx_eq(cam.transform.pos.y, 0.5));
    }

    #[test]
    fn camera_spin() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        cam.spin_x(45.0);
        cam.spin_y(90.0);
        cam.spin_z(30.0);
        assert!(approx_eq(cam.transform.rot.x, 45.0));
        assert!(approx_eq(cam.transform.rot.y, 0.0));  // initial -90 + 90
        assert!(approx_eq(cam.transform.rot.z, 30.0));
    }

    #[test]
    fn camera_pre_update() {
        let mut cam = Camera::new(Size2D::from(1920, 1080), CamProj::Persp);
        let view_before = cam.transform.view_matrix;
        cam.transform.rot.x = 30.0;
        cam.pre_update();
        assert!(view_before != cam.transform.view_matrix);
    }
}
