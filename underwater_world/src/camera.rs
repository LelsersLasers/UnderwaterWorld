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
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera {
    pub fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }
}
//----------------------------------------------------------------------------//

//----------------------------------------------------------------------------//
#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj(&mut self, camera: &Camera) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}
//----------------------------------------------------------------------------//


//----------------------------------------------------------------------------//
pub struct CameraController {
    speed: f64,
    zoom_speed: f64,

    lat: f64,
    lon: f64,
    radius: f64,

    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
}

impl CameraController {
    pub fn new() -> Self {
        Self {
            speed: 0.75,
            zoom_speed: 3.0,
            lat: 0.,
            lon: 0.,
            radius: 10.,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
            is_up_pressed: false,
            is_down_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                input: winit::event::KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == winit::event::ElementState::Pressed;
                match keycode {
                    winit::event::VirtualKeyCode::W | winit::event::VirtualKeyCode::Up => {
                        self.is_forward_pressed = is_pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                        self.is_left_pressed = is_pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::S | winit::event::VirtualKeyCode::Down => {
                        self.is_backward_pressed = is_pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                        self.is_right_pressed = is_pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::Q | winit::event::VirtualKeyCode::PageUp => {
                        self.is_up_pressed = is_pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::E | winit::event::VirtualKeyCode::PageDown => {
                        self.is_down_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update(&mut self, camera: &mut Camera, delta: f64) {
        if self.is_forward_pressed {
            self.lat += self.speed * delta;
        }
        if self.is_backward_pressed {
            self.lat -= self.speed * delta;
        }
        if self.is_left_pressed {
            self.lon -= self.speed * delta;
        }
        if self.is_right_pressed {
            self.lon += self.speed * delta;
        }
        if self.is_up_pressed {
            self.radius -= self.zoom_speed * delta;
        }
        if self.is_down_pressed {
            self.radius += self.zoom_speed * delta;
        }
        self.clamp_pos();
        self.update_eye(camera);
    }
    fn clamp_pos(&mut self) {
        if self.lat > std::f64::consts::PI / 2. {
            self.lat = std::f64::consts::PI / 2. - 0.0001;
        } else if self.lat < -std::f64::consts::PI / 2. {
            self.lat = -std::f64::consts::PI / 2. + 0.0001;
        }
        if self.lon > std::f64::consts::PI {
            self.lon -= 2. * std::f64::consts::PI;
        } else if self.lon < -std::f64::consts::PI {
            self.lon += 2. * std::f64::consts::PI;
        }
        if self.radius < 1. {
            self.radius = 1.;
        }
    }
    pub fn update_eye(&self, camera: &mut Camera) {
        camera.eye = cgmath::Point3::new(
            (self.radius * self.lat.cos() * self.lon.cos()) as f32,
            (self.radius * self.lat.cos() * self.lon.sin()) as f32,
            (self.radius * self.lat.sin()) as f32,
        );
    }
}
//----------------------------------------------------------------------------//
