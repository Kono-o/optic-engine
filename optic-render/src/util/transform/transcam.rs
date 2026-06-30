use optic_core::{CamProj, ClipDist, Size2D};
use cgmath::*;

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

impl CamTransform {
    pub fn calc_matrices(&mut self) {
        let aspect = self.size.aspect_ratio();

        // View matrix
        let pitch = Rad::from(Deg(self.rot.x));
        let yaw = Rad::from(Deg(self.rot.y));
        self.front = vec3(
            yaw.cos() * pitch.cos(),
            pitch.sin(),
            yaw.sin() * pitch.cos(),
        )
        .normalize();

        let target = self.pos + self.front;
        let view = Matrix4::<f32>::look_at_rh(Point3::from_vec(self.pos), Point3::from_vec(target), Vector3::unit_y());
        self.view_matrix = view;

        // Projection matrices
        self.persp_matrix = perspective(Rad::from(Deg(self.fov)), aspect, self.clip.near, self.clip.far);
        self.ortho_matrix = ortho(
            -self.ortho_scale * aspect,
            self.ortho_scale * aspect,
            -self.ortho_scale,
            self.ortho_scale,
            self.clip.near,
            self.clip.far,
        );
    }

    pub fn view_matrix(&self) -> Matrix4<f32> {
        self.view_matrix
    }

    pub fn proj_matrix(&self) -> Matrix4<f32> {
        match self.proj {
            CamProj::Persp => self.persp_matrix,
            CamProj::Ortho => self.ortho_matrix,
        }
    }
}
