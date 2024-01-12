use crate::camera;

const MIN_SPEED: f32 = 0.5;
const MAX_SPEED: f32 = 7.5;
const ACCELERATION: f32 = 5.0;

const MAX_TURN_SPEED: f32 = std::f32::consts::PI / 3.0;
const TURN_ACCELERATION: f32 = std::f32::consts::PI;
const TURN_DECAY: f32 = 3.0;


struct Keys {
	w_down: bool,
	s_down: bool,
	a_down: bool,
	d_down: bool,
	q_down: bool,
	e_down: bool,
	space_down: bool,
	control_down: bool,
}
impl Keys {
	fn new() -> Self {
		Self {
			w_down: false,
			s_down: false,
			a_down: false,
			d_down: false,
			q_down: false,
			e_down: false,
			space_down: false,
			control_down: false,
		}
	}

	fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
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
                        self.w_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::S | winit::event::VirtualKeyCode::Down => {
                        self.s_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::A | winit::event::VirtualKeyCode::Left => {
                        self.a_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::D | winit::event::VirtualKeyCode::Right => {
                        self.d_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::Q | winit::event::VirtualKeyCode::PageUp => {
                        self.q_down = pressed;
                        true
                    }
                    winit::event::VirtualKeyCode::E | winit::event::VirtualKeyCode::PageDown => {
                        self.e_down = pressed;
                        true
                    }
					winit::event::VirtualKeyCode::Space => {
						self.space_down = pressed;
						true
					}
					winit::event::VirtualKeyCode::LControl | winit::event::VirtualKeyCode::RControl => {
						self.control_down = pressed;
						true
					}
                    _ => false,
                }
            }
            _ => false,
        }
	}
}

pub struct Sub {
	pos: cgmath::Point3<f32>,
	target: cgmath::Point3<f32>,

    up: cgmath::Vector3<f32>,
    forward: cgmath::Vector3<f32>,
    right: cgmath::Vector3<f32>,

	yaw: f32,
	yaw_speed: f32,

	pitch: f32,
	pitch_speed: f32,

	roll: f32,
	roll_speed: f32,

	speed: f32,

	keys: Keys,
}

impl Sub {
	pub fn new() -> Self {
		let mut sub = Self {
			pos: cgmath::Point3::new(0.0, 0.0, 0.0),
			target: cgmath::Point3::new(1.0, 0.0, 0.0),
            up: cgmath::Vector3::unit_z(),
            forward: cgmath::Vector3::unit_x(),
            right: cgmath::Vector3::unit_y(),

			yaw: 0.0,
			yaw_speed: 0.0,

			pitch: 0.0,
			pitch_speed: 0.0,

			roll: 0.0,
			roll_speed: 0.0,
			
			speed: 4.0,

			keys: Keys::new(),
		};
		sub.update(0.0);
		sub
	}

    fn decay_turn_rates(&mut self, delta: f32) {
        let min_turn_decay = TURN_DECAY * MAX_TURN_SPEED * delta;

        if self.keys.w_down || self.keys.s_down {
			self.pitch_speed = self.pitch_speed.clamp(-MAX_TURN_SPEED, MAX_TURN_SPEED);
		} else if self.pitch_speed.abs() < min_turn_decay {
			self.pitch_speed = 0.0;
		} else {
			self.pitch_speed -= min_turn_decay * self.pitch_speed.signum();
		}

        if self.keys.a_down || self.keys.d_down {
			self.yaw_speed = self.yaw_speed.clamp(-MAX_TURN_SPEED, MAX_TURN_SPEED);
		} else if self.yaw_speed.abs() < min_turn_decay {
			self.yaw_speed = 0.0;
		} else {
			self.yaw_speed -= min_turn_decay * self.yaw_speed.signum();
		}

		if self.keys.q_down || self.keys.e_down {
			self.roll_speed = self.roll_speed.clamp(-MAX_TURN_SPEED, MAX_TURN_SPEED);
		} else if self.roll_speed.abs() < min_turn_decay {
			self.roll_speed = 0.0;
		} else {
			self.roll_speed -= min_turn_decay * self.roll_speed.signum();
		}
    }

	pub fn update(&mut self, delta: f32) {
		if self.keys.w_down { self.pitch_speed -= TURN_ACCELERATION * delta; }
		if self.keys.s_down { self.pitch_speed += TURN_ACCELERATION * delta; }
		if self.keys.a_down { self.yaw_speed   += TURN_ACCELERATION * delta; }
		if self.keys.d_down { self.yaw_speed   -= TURN_ACCELERATION * delta; }
		if self.keys.q_down { self.roll_speed  -= TURN_ACCELERATION * delta; }
		if self.keys.e_down { self.roll_speed  += TURN_ACCELERATION * delta; }
		if self.keys.space_down { self.speed   += ACCELERATION * delta; }
		if self.keys.control_down { self.speed -= ACCELERATION * delta; }

		self.decay_turn_rates(delta);

        let pitch_change = self.pitch_speed * delta;
		self.pitch += pitch_change;

        let yaw_change = self.yaw_speed * delta;
		self.yaw += yaw_change;

		let roll_change = self.roll_speed * delta;
		self.roll += roll_change;

		self.speed = self.speed.clamp(MIN_SPEED, MAX_SPEED);


        use cgmath::{Rotation, Rotation3};
        let pitch_change_quat = cgmath::Quaternion::from_axis_angle(self.right, cgmath::Rad(pitch_change));
        let yaw_change_quat = cgmath::Quaternion::from_axis_angle(self.up, cgmath::Rad(yaw_change));
		let roll_change_quat = cgmath::Quaternion::from_axis_angle(self.forward, cgmath::Rad(roll_change));

		let overall_change_quat = yaw_change_quat * pitch_change_quat * roll_change_quat;

		self.forward = overall_change_quat.rotate_vector(self.forward);
		self.up = overall_change_quat.rotate_vector(self.up);
		self.right = overall_change_quat.rotate_vector(self.right);
        
        // self.forward = yaw_change_quat.rotate_vector(pitch_change_quat.rotate_vector(self.forward));
        // self.up = yaw_change_quat.rotate_vector(pitch_change_quat.rotate_vector(self.up));
        // self.right = yaw_change_quat.rotate_vector(pitch_change_quat.rotate_vector(self.right));

        self.pos += self.forward * self.speed * delta;
        self.target = self.pos + self.forward * self.speed;
	}

	pub fn update_camera(&self, camera: &mut camera::Camera) {
		camera.set_eye(self.pos);
		camera.set_target(self.target);
        camera.set_up(self.up);
		
		camera.update_uniform();
	}

	pub fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
		self.keys.process_events(event)
    }

}