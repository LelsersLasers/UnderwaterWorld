use crate::{chunk, world};
use cgmath::SquareMatrix;

const Z_NEAR_MAIN: f32 = 2.0;
const Z_NEAR_FISH: f32 = 0.1;
const Z_FAR: f32 = chunk::CHUNK_SIZE as f32 * (world::VIEW_DIST + 1) as f32;
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

    pub fn update_uniform(&mut self) {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj_main = cgmath::perspective(cgmath::Deg(FOVY), self.aspect, Z_NEAR_MAIN, Z_FAR);
        let proj_fish = cgmath::perspective(cgmath::Deg(FOVY), self.aspect, Z_NEAR_FISH, Z_FAR);

        self.uniform.view_proj_main = (OPENGL_TO_WGPU_MATRIX * (proj_main * view)).into();
        self.uniform.view_proj_fish = (OPENGL_TO_WGPU_MATRIX * (proj_fish * view)).into();
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj_main: [[f32; 4]; 4],
    view_proj_fish: [[f32; 4]; 4],
}
impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj_main: cgmath::Matrix4::identity().into(),
            view_proj_fish: cgmath::Matrix4::identity().into(),
        }
    }
}
