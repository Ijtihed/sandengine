use glam::{Mat4, Vec3};

pub struct OrbitCamera {
    pub target: Vec3,
    pub distance: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub aspect: f32,
    pub fov: f32,
    pub near: f32,
    pub far: f32,
}

impl OrbitCamera {
    pub fn new(target: Vec3, distance: f32) -> Self {
        Self {
            target,
            distance,
            yaw: std::f32::consts::FRAC_PI_4,
            pitch: 0.5,
            aspect: 16.0 / 9.0,
            fov: std::f32::consts::FRAC_PI_4,
            near: 0.1,
            far: 1000.0,
        }
    }

    pub fn eye(&self) -> Vec3 {
        self.target
            + self.distance
                * Vec3::new(
                    self.pitch.cos() * self.yaw.sin(),
                    self.pitch.sin(),
                    self.pitch.cos() * self.yaw.cos(),
                )
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye(), self.target, Vec3::Y)
    }

    pub fn proj_matrix(&self) -> Mat4 {
        Mat4::perspective_rh(self.fov, self.aspect, self.near, self.far)
    }

    pub fn vp_matrix(&self) -> Mat4 {
        self.proj_matrix() * self.view_matrix()
    }

    pub fn orbit(&mut self, dx: f32, dy: f32) {
        self.yaw += dx * 0.005;
        self.pitch = (self.pitch + dy * 0.005).clamp(-1.5, 1.5);
    }

    pub fn zoom(&mut self, delta: f32) {
        if delta > 0.0 {
            self.distance *= 0.9;
        } else if delta < 0.0 {
            self.distance *= 1.1;
        }
        self.distance = self.distance.clamp(10.0, 500.0);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        let view = self.view_matrix();
        let right = Vec3::new(view.col(0).x, view.col(0).y, view.col(0).z);
        let up = Vec3::new(view.col(1).x, view.col(1).y, view.col(1).z);
        self.target += right * dx * 0.05 + up * -dy * 0.05;
    }

    pub fn screen_ray(&self, ndc_x: f32, ndc_y: f32) -> (Vec3, Vec3) {
        let inv_vp = self.vp_matrix().inverse();
        let near_pt = inv_vp.project_point3(Vec3::new(ndc_x, ndc_y, 0.0));
        let far_pt = inv_vp.project_point3(Vec3::new(ndc_x, ndc_y, 1.0));
        let dir = (far_pt - near_pt).normalize();
        (near_pt, dir)
    }
}
