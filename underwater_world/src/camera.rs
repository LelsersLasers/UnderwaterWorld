// use crate::consts;
use cgmath::SquareMatrix;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


pub struct Camera {
    eye: cgmath::Point3<f32>,
    target: cgmath::Point3<f32>,
    up: cgmath::Vector3<f32>,
    aspect: f32,
    fovy: f32,
    znear: f32,
    zfar: f32,

    uniform: CameraUniform,
}
impl Camera {
    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            eye: (1., 1., 1.).into(),
            target: (0., 0., 0.).into(),
            up: cgmath::Vector3::unit_z(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.,
            znear: 0.01,
            zfar: 100.,

            uniform: CameraUniform::new(),
        }
    }

    pub fn uniform(&self) -> &CameraUniform {
        &self.uniform
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }

    pub fn update_uniform(&mut self) {
        self.uniform.view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.build_view_projection_matrix()).into();
    }

    pub fn set_eye(&mut self, eye: cgmath::Point3<f32>) {
        self.eye = eye;
    }

    pub fn set_target(&mut self, target: cgmath::Point3<f32>) {
        self.target = target;
    }

    pub fn set_up(&mut self, up: cgmath::Vector3<f32>) {
        self.up = up;
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}