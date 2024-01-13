use crate::{camera, draw, sub_obj};
use wgpu::util::DeviceExt;
use cgmath::{EuclideanSpace, One, Rotation, Rotation3};

const MIN_SPEED: f32 = 0.5;
const MAX_SPEED: f32 = 7.5;
const ACCELERATION: f32 = 5.0;

const MAX_TURN_SPEED: f32 = std::f32::consts::PI / 3.0;
const TURN_ACCELERATION: f32 = std::f32::consts::PI;
const TURN_DECAY: f32 = 3.0;

const TARGET_DOWN: f32 = 0.6;
const HORIZONTAL_OFFSET: f32 = 7.0;
const VERTICAL_OFFSET: f32 = 6.0;

const SUB_MODEL_SCALE: f32 = 8.0;

const CAMERA_FOLLOW_SPEED: f32 = 20.0;

const COLOR: [f32; 3] = [
	0.15625,
	0.15625,
	0.1953125,
];

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

	overall_rotation: cgmath::Quaternion<f32>,

	yaw: f32,
	yaw_speed: f32,

	pitch: f32,
	pitch_speed: f32,

	roll: f32,
	roll_speed: f32,

	speed: f32,

	keys: Keys,

	verts_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
	inst_buffer: wgpu::Buffer,

    num_indices: usize,
}

impl Sub {
	pub fn new(device: &wgpu::Device) -> Self {
		let mut verts = Vec::new();
        let mut indices = Vec::new();

        let mut highest_v: f32 = 0.0;

        for line in sub_obj::SUB_OBJ.lines() {
            let mut split = line.split_whitespace();
            let first = split.next();
            match first {
                Some("v") => {
                    let x: f32 = split.next().unwrap().parse().unwrap();
                    let y: f32 = split.next().unwrap().parse().unwrap();
                    let z: f32 = split.next().unwrap().parse().unwrap();
                    verts.push(draw::Vert::new([x, y, z], COLOR));

                    highest_v = highest_v.max(x.abs()).max(y.abs()).max(z.abs());
                }
                Some("f") => {
                    for _ in 0..3 {
                        let i = split.next().unwrap().split("//").next().unwrap().parse::<u32>().unwrap() - 1;
                        indices.push(i);
                    }
                }
                _ => {}
            }
        }

        verts.iter_mut().for_each(|v| {
            v.pos[0] *= SUB_MODEL_SCALE / highest_v / 2.0;
            v.pos[1] *= SUB_MODEL_SCALE / highest_v;
            v.pos[2] *= SUB_MODEL_SCALE / highest_v;
        });
        //--------------------------------------------------------------------//

		let verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sub Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });
        
        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sub Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

		let inst = draw::sub::Instance::identity();
		let inst_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&[inst]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

		Self {
			pos: cgmath::Point3::new(0.0, 0.0, 0.0),
			target: cgmath::Point3::new(1.0, 0.0, 0.0),
            up: cgmath::Vector3::unit_z(),
            forward: cgmath::Vector3::unit_x(),
            right: cgmath::Vector3::unit_y(),

			overall_rotation: cgmath::Quaternion::one(),

			yaw: 0.0,
			yaw_speed: 0.0,

			pitch: 0.0,
			pitch_speed: 0.0,

			roll: 0.0,
			roll_speed: 0.0,
			
			speed: 4.0,

			keys: Keys::new(),

			verts_buffer,
            index_buffer,
			inst_buffer,

            num_indices: indices.len(),
		}
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

	pub fn update(&mut self, queue: &wgpu::Queue, delta: f32) {
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


        let pitch_change_quat = cgmath::Quaternion::from_axis_angle(self.right, cgmath::Rad(pitch_change));
        let yaw_change_quat = cgmath::Quaternion::from_axis_angle(self.up, cgmath::Rad(yaw_change));
		let roll_change_quat = cgmath::Quaternion::from_axis_angle(self.forward, cgmath::Rad(roll_change));

		let overall_change_quat = yaw_change_quat * pitch_change_quat * roll_change_quat;
		self.overall_rotation = overall_change_quat * self.overall_rotation;

		self.forward = overall_change_quat.rotate_vector(self.forward);
		self.up = overall_change_quat.rotate_vector(self.up);
		self.right = overall_change_quat.rotate_vector(self.right);
        
        self.pos += self.forward * self.speed * delta;
        self.target = self.pos + self.forward * self.speed;

		let inst_mat = cgmath::Matrix4::from_translation(self.pos.to_vec()) * cgmath::Matrix4::from(self.overall_rotation);
		let inst = draw::sub::Instance::new(inst_mat);
		queue.write_buffer(&self.inst_buffer, 0, bytemuck::cast_slice(&[inst]));
	}

	pub fn update_camera(&self, camera: &mut camera::Camera, delta: f32) {
		let eye_goal = self.pos - self.forward * HORIZONTAL_OFFSET + self.up * VERTICAL_OFFSET;
		let eye_diff = eye_goal - camera.eye;
		let eye_move = eye_diff * delta * CAMERA_FOLLOW_SPEED;
		camera.eye += eye_move;
        // camera.eye = eye_goal;

		// let target_goal = self.pos + self.forward * TARGET_LEAD;
		// let target_diff = target_goal - camera.target;
		// let target_move = target_diff * delta * CAMERA_FOLLOW_SPEED;
		// camera.target += target_move;
        // camera.target = target_goal;
        camera.target = camera.eye + self.forward - self.up * TARGET_DOWN;

		let up_diff = self.up - camera.up;
		let up_move = up_diff * delta * CAMERA_FOLLOW_SPEED;
		camera.up += up_move;
        // camera.up = self.up;
		
		camera.update_uniform();
	}

	pub fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
		self.keys.process_events(event)
    }

    pub fn vert_buffer_slice(&self) -> wgpu::BufferSlice { self.verts_buffer.slice(..) }
    pub fn index_buffer_slice(&self) -> wgpu::BufferSlice { self.index_buffer.slice(..) }
	pub fn inst_buffer_slice(&self) -> wgpu::BufferSlice { self.inst_buffer.slice(..) }
    pub fn num_indices(&self) -> usize { self.num_indices }
}