// use crate::consts;
use cgmath::SquareMatrix;

const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 100.0;
const FOVY: f32 = 45.0;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);


pub struct Camera {
    pub eye: cgmath::Point3<f32>,
    pub target: cgmath::Point3<f32>,
    pub up: cgmath::Vector3<f32>,
    aspect: f32,

    uniform: CameraUniform,
}
impl Camera {
    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        Self {
            eye: cgmath::Point3::new(0.0, 0.0, 0.0),
            target: cgmath::Point3::new(1.0, 0.0, 0.0),
            up: cgmath::Vector3::unit_z(),
            aspect: config.width as f32 / config.height as f32,

            uniform: CameraUniform::new(),
        }
    }

    pub fn uniform(&self) -> &CameraUniform {
        &self.uniform
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(FOVY), self.aspect, Z_NEAR, Z_FAR);
        proj * view
    }

    pub fn update_uniform(&mut self) {
        self.uniform.view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.build_view_projection_matrix()).into();
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