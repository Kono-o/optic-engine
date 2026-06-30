use optic_core::{CamProj, Size2D};
use optic_render::Camera;

pub struct Scene {
    pub camera: Camera,
}

impl Scene {
    pub fn new(size: Size2D, proj: CamProj) -> Self {
        Self { camera: Camera::new(size, proj) }
    }
}
