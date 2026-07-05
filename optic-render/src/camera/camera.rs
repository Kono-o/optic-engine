use optic_core::{CamProj, ClipDist, Size2D};
use cgmath::*;

use crate::util::transform::CamTransform;

/// A 3D camera with perspective or orthographic projection.
///
/// Wraps a [`CamTransform`] that holds position, rotation, FOV, clip distances,
/// and pre-computed view/projection matrices. After mutating any transform
/// property, call [`pre_update`](Camera::pre_update) to recalculate.
///
/// # Conventions
///
/// - **Y-up** world coordinate system.
/// - **-Z forward** — the camera looks down its local -Z axis by default.
/// - **Euler angles** in degrees, applied in XYZ order.
///
/// # Movement methods
///
/// | Direction | Method | Axis |
/// |---|---|---|
    /// | Forward (in look direction) | [`fly_forward`](Camera::fly_forward) | Camera-local -Z |
/// | Backward | [`fly_back`](Camera::fly_back) | Camera-local +Z |
/// | Left (strafe) | [`fly_left`](Camera::fly_left) | Camera-local -X |
/// | Right (strafe) | [`fly_right`](Camera::fly_right) | Camera-local +X |
/// | Up | [`fly_up`](Camera::fly_up) | World +Y |
/// | Down | [`fly_down`](Camera::fly_down) | World -Y |
///
/// # Rotation methods
///
/// | Method | Effect | Common name |
/// |---|---|---|
/// | [`spin_x`](Camera::spin_x) | Pitch (look up/down) | Tilt forward/backward |
/// | [`spin_y`](Camera::spin_y) | Yaw (look left/right) | Turn head side-to-side |
/// | [`spin_z`](Camera::spin_z) | Roll (rotate view) | Tilt horizon |
///
/// # Getters and setters
///
/// | Property | Getter | Setter |
/// |---|---|---|
/// | Field of view (degrees) | [`fov`](Camera::fov) | [`set_fov`](Camera::set_fov), [`add_fov`](Camera::add_fov) |
/// | Orthographic scale | [`ortho_scale`](Camera::ortho_scale) | [`set_ortho_scale`](Camera::set_ortho_scale), [`add_ortho_scale`](Camera::add_ortho_scale) |
/// | Projection mode | [`proj`](Camera::proj) | [`set_proj`](Camera::set_proj) |
/// | Clip distances | [`clip`](Camera::clip) | [`set_clip`](Camera::set_clip), [`set_clip_near`](Camera::set_clip_near), [`set_clip_far`](Camera::set_clip_far) |
/// | Viewport size | — | [`set_size`](Camera::set_size) |
/// | View/proj matrices | — | [`pre_update`](Camera::pre_update) |
///
/// # Example
///
/// ```ignore
/// use optic_core::{CamProj, Size2D};
/// use optic_render::Camera;
///
/// let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
/// cam.fly_forward(10.0);       // move in look direction
/// cam.spin_y(-90.0);        // yaw left 90°
/// cam.pre_update();          // recalculate matrices
/// ```
pub struct Camera {
    pub transform: CamTransform,
}

impl Camera {
    /// Creates a camera sized to match the given canvas dimensions.
    pub fn match_canvas_size(canvas: &crate::handles::Canvas, proj: CamProj) -> Self {
        Self::new(canvas.size(), proj)
    }

    /// Creates a camera at `(0, 0, 5)` with 75° FOV, default clip distances,
    /// and the given projection type.
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

    /// Recalculates the view and projection matrices from the current transform state.
    ///
    /// Call this once per frame **after** all movement ([`fly_forward`](Camera::fly_forward), etc.)
    /// and rotation ([`spin_y`](Camera::spin_y), etc.) have been applied.
    /// The matrices are consumed by the rendering pipeline — without this call
    /// the camera will continue using stale matrices from the previous frame.
    pub fn pre_update(&mut self) {
        self.transform.calc_matrices();
    }

    /// Returns the vertical field of view in degrees.
    pub fn fov(&self) -> f32 { self.transform.fov }
    /// Returns the orthographic scale factor.
    pub fn ortho_scale(&self) -> f32 { self.transform.ortho_scale }
    /// Returns the current projection type.
    pub fn proj(&self) -> CamProj { self.transform.proj }
    /// Returns the near/far clip distances.
    pub fn clip(&self) -> ClipDist { self.transform.clip }

    /// Sets both near and far clip distances at once.
    pub fn set_clip(&mut self, clip: ClipDist) { self.transform.clip = clip; }
    /// Sets the near clip plane distance.
    pub fn set_clip_near(&mut self, near: f32) { self.transform.clip.near = near; }
    /// Sets the far clip plane distance.
    pub fn set_clip_far(&mut self, far: f32) { self.transform.clip.far = far; }
    /// Sets the canvas/viewport size (used for aspect ratio calculation).
    pub fn set_size(&mut self, size: Size2D) { self.transform.size = size; }
    /// Switches between perspective and orthographic projection.
    pub fn set_proj(&mut self, proj: CamProj) { self.transform.proj = proj; }

    /// Sets the vertical field of view in degrees (clamped to ≥ 0.01).
    pub fn set_fov(&mut self, fov: f32) {
        self.transform.fov = fov.max(0.01);
    }
    /// Adds `value` to the FOV (clamped to ≥ 0.01).
    pub fn add_fov(&mut self, value: f32) {
        self.transform.fov = (self.transform.fov + value).max(0.01);
    }

    /// Sets the orthographic scale factor.
    pub fn set_ortho_scale(&mut self, value: f32) { self.transform.ortho_scale = value; }
    /// Adds `value` to the orthographic scale factor.
    pub fn add_ortho_scale(&mut self, value: f32) { self.transform.ortho_scale += value; }

    /// Moves the camera forward (in the direction it faces).
    ///
    /// The forward direction is derived from the camera's current rotation
    /// (stored as `front` in [`CamTransform`]). For the default orientation this
    /// moves along world -Z.
    ///
    /// # When to use
    ///
    /// Call this each frame with `speed * delta_time` for smooth first-person
    /// movement. Call [`pre_update`](Camera::pre_update) after all movement and
    /// rotation for the frame.
    pub fn fly_forward(&mut self, speed: f32) {
        self.transform.pos += speed * self.transform.front;
    }
    /// Moves the camera backward (opposite the direction it faces).
    ///
    /// The inverse of [`fly_forward`](Camera::fly_forward). Equivalent to
    /// `fly_forward(-speed)`.
    pub fn fly_back(&mut self, speed: f32) {
        self.transform.pos -= speed * self.transform.front;
    }
    /// Moves the camera left (strafe), perpendicular to the forward direction.
    ///
    /// The strafe direction is computed as `front × world_up`, producing a
    /// vector orthogonal to both the look direction and the world Y axis. This
    /// keeps the horizon level even when pitching up or down.
    pub fn fly_left(&mut self, speed: f32) {
        self.transform.pos -= speed * self.transform.front.cross(Vector3::unit_y()).normalize();
    }
    /// Moves the camera right (strafe), perpendicular to the forward direction.
    ///
    /// The inverse of [`fly_left`](Camera::fly_left). Equivalent to
    /// `fly_left(-speed)`.
    pub fn fly_right(&mut self, speed: f32) {
        self.transform.pos += speed * self.transform.front.cross(Vector3::unit_y()).normalize();
    }
    /// Moves the camera straight up (world Y axis).
    ///
    /// Unlike [`fly_forward`](Camera::fly_forward) / [`fly_left`](Camera::fly_left),
    /// this always moves along the **world** Y axis regardless of the camera's
    /// current pitch or roll.
    pub fn fly_up(&mut self, speed: f32) { self.transform.pos.y += speed; }
    /// Moves the camera straight down (world Y axis).
    ///
    /// The inverse of [`fly_up`](Camera::fly_up). Equivalent to
    /// `fly_up(-speed)`.
    pub fn fly_down(&mut self, speed: f32) { self.transform.pos.y -= speed; }

    /// Pitches the camera up or down (rotation around the local X axis).
    ///
    /// Positive values tilt the view downward (looking toward the ground);
    /// negative values tilt upward (looking toward the sky).
    ///
    /// # When to use
    ///
    /// Combine with [`spin_y`](Camera::spin_y) for full free-look control
    /// (e.g. mouse-look in a first-person game).
    pub fn spin_x(&mut self, speed: f32) { self.transform.rot.x += speed; }
    /// Yaws the camera left or right (rotation around the local Y axis).
    ///
    /// Positive values turn right; negative values turn left. This is the
    /// primary rotation for first-person horizontal look-around.
    pub fn spin_y(&mut self, speed: f32) { self.transform.rot.y += speed; }
    /// Rolls the camera (rotation around the local Z axis).
    ///
    /// Tilts the horizon. Rarely used in first-person games; useful for
    /// cinematic cameras or flight simulators.
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
        let cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        assert!((cam.fov() - 75.0).abs() < f32::EPSILON);
        assert!((cam.transform.pos.y - 0.0).abs() < f32::EPSILON);
        assert!((cam.transform.pos.z - 5.0).abs() < f32::EPSILON);
    }

    #[test]
    fn camera_new_ortho() {
        let cam = Camera::new(Size2D::new(800, 600), CamProj::Ortho);
        assert_eq!(cam.proj(), CamProj::Ortho);
    }

    #[test]
    fn camera_set_clip() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        cam.set_clip(ClipDist::new(0.1, 500.0));
        assert!(approx_eq(cam.clip().near, 0.1));
        assert!(approx_eq(cam.clip().far, 500.0));
    }

    #[test]
    fn camera_set_clip_near_far() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        cam.set_clip_near(0.5);
        cam.set_clip_far(2000.0);
        assert!(approx_eq(cam.clip().near, 0.5));
        assert!(approx_eq(cam.clip().far, 2000.0));
    }

    #[test]
    fn camera_set_fov() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        cam.set_fov(90.0);
        assert!(approx_eq(cam.fov(), 90.0));
    }

    #[test]
    fn camera_set_fov_clamped() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        cam.set_fov(-1.0); // clamped to 0.01
        assert!(approx_eq(cam.fov(), 0.01));
    }

    #[test]
    fn camera_add_fov() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        cam.add_fov(10.0);
        assert!(approx_eq(cam.fov(), 85.0));
    }

    #[test]
    fn camera_ortho_scale() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Ortho);
        assert!((cam.ortho_scale() - 2.0).abs() < f32::EPSILON);
        cam.set_ortho_scale(5.0);
        assert!((cam.ortho_scale() - 5.0).abs() < f32::EPSILON);
        cam.add_ortho_scale(3.0);
        assert!((cam.ortho_scale() - 8.0).abs() < f32::EPSILON);
    }

    #[test]
    fn camera_set_proj() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        assert_eq!(cam.proj(), CamProj::Persp);
        cam.set_proj(CamProj::Ortho);
        assert_eq!(cam.proj(), CamProj::Ortho);
    }

    #[test]
    fn camera_set_size() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        cam.set_size(Size2D::new(800, 600));
        assert_eq!(cam.transform.size.w, 800);
        assert_eq!(cam.transform.size.h, 600);
    }

    #[test]
    fn camera_fly_movements() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        let start = cam.transform.pos;

        cam.fly_forward(1.0);
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
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        cam.spin_x(45.0);
        cam.spin_y(90.0);
        cam.spin_z(30.0);
        assert!(approx_eq(cam.transform.rot.x, 45.0));
        assert!(approx_eq(cam.transform.rot.y, 0.0));  // initial -90 + 90
        assert!(approx_eq(cam.transform.rot.z, 30.0));
    }

    #[test]
    fn camera_pre_update() {
        let mut cam = Camera::new(Size2D::new(1920, 1080), CamProj::Persp);
        let view_before = cam.transform.view_matrix;
        cam.transform.rot.x = 30.0;
        cam.pre_update();
        assert!(view_before != cam.transform.view_matrix);
    }
}
