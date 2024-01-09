// use crate::consts;

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

    lat: f64,
    lon: f64,
    radius: f64,

    turn_speed: f64,
    zoom_speed: f64,

    up_down: bool,
    down_down: bool,
    left_down: bool,
    right_down: bool,
    forward_down: bool,
    backward_down: bool,
}
impl Camera {
    pub fn new(config: &wgpu::SurfaceConfiguration) -> Self {
        let mut camera = Camera {
            eye: (1., 1., 1.).into(),
            target: (0., 0., 0.).into(),
            up: cgmath::Vector3::unit_z(),
            aspect: config.width as f32 / config.height as f32,
            fovy: 45.,
            znear: 0.01,
            zfar: 100.,

            uniform: CameraUniform::new(),

            lat: 0.35,
            lon: 0.35,
            radius: 10.0,
            
            turn_speed: std::f64::consts::PI / 3.,
            zoom_speed: 5.,
            
            up_down: false,
            down_down: false,
            left_down: false,
            right_down: false,
            forward_down: false,
            backward_down: false,
        };
        camera.update_eye();
        camera.update_uniform();

        camera
    }

    pub fn uniform(&self) -> &CameraUniform {
        &self.uniform
    }

    fn build_view_projection_matrix(&self) -> cgmath::Matrix4<f32> {
        let view = cgmath::Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = cgmath::perspective(cgmath::Deg(self.fovy), self.aspect, self.znear, self.zfar);
        proj * view
    }

    fn update_uniform(&mut self) {
        self.uniform.view_proj =
            (OPENGL_TO_WGPU_MATRIX * self.build_view_projection_matrix()).into();
    }

    pub fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
        match event {
            winit::event::WindowEvent::KeyboardInput {
                input:
                winit::event::KeyboardInput {
                        state,
                        virtual_keycode: Some(keycode),
                        ..
                    },
                ..
            } => {
                let pressed = *state == winit::event::ElementState::Pressed;
                match keycode {
                    winit::event::VirtualKeyCode::W | winit::event::VirtualKeyCode::Up => {
                        self.up_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::S | winit::event::VirtualKeyCode::Down => {
                        self.down_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                        self.left_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                        self.right_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::Q | winit::event::VirtualKeyCode::PageUp => {
                        self.forward_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::E | winit::event::VirtualKeyCode::PageDown => {
                        self.backward_down = pressed;
                        true
                    }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    pub fn update(&mut self, delta: f64) {
        if self.up_down {
            self.lat += self.turn_speed * delta;
        }
        if self.down_down {
            self.lat -= self.turn_speed * delta;
        }
        if self.left_down {
            self.lon -= self.turn_speed * delta;
        }
        if self.right_down {
            self.lon += self.turn_speed * delta;
        }
        if self.forward_down {
            self.radius -= self.zoom_speed * delta;
        }
        if self.backward_down {
            self.radius += self.zoom_speed * delta;
        }
        self.clamp_pos();
        self.update_eye();
        self.update_uniform();
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
        if self.radius < 5. {
            self.radius = 5.;
        }
    }

    fn update_eye(&mut self) {
        self.eye = cgmath::Point3::new(
            (self.radius * self.lat.cos() * self.lon.cos()) as f32,
            (self.radius * self.lat.cos() * self.lon.sin()) as f32,
            (self.radius * self.lat.sin()) as f32,
        );
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}
impl CameraUniform {
    fn new() -> Self {
        use cgmath::SquareMatrix;
        Self {
            view_proj: cgmath::Matrix4::identity().into(),
        }
    }
}