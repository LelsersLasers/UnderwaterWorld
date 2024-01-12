use crate::{camera, draw};
use wgpu::util::DeviceExt;
use cgmath::{Rotation, Rotation3};

const MIN_SPEED: f32 = 0.5;
const MAX_SPEED: f32 = 7.5;
const ACCELERATION: f32 = 5.0;

const MAX_TURN_SPEED: f32 = std::f32::consts::PI / 3.0;
const TURN_ACCELERATION: f32 = std::f32::consts::PI;
const TURN_DECAY: f32 = 3.0;

const STACK_COUNT: usize = 8;
const SECTOR_COUNT: usize = 8;
const RADIUS: f32 = 2.0;
const LENGTH: f32 = 4.0;
const COLOR: [f32; 3] = [
	0.07843137254,
	0.07843137254,
	0.09019607843,
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

	yaw: f32,
	yaw_speed: f32,

	pitch: f32,
	pitch_speed: f32,

	roll: f32,
	roll_speed: f32,

	speed: f32,

	keys: Keys,

	num_verts: usize,
	verts_buffer: wgpu::Buffer,
	// instance_buffer: wgpu::Buffer,
}

impl Sub {
	pub fn new(device: &wgpu::Device) -> Self {

		// for(int i = 0; i <= stackCount; ++i)
		// {
		// 	stackAngle = PI / 2 - i * stackStep;        // starting from pi/2 to -pi/2
		// 	xy = radius * cosf(stackAngle);             // r * cos(u)
		// 	z = radius * sinf(stackAngle);              // r * sin(u)

		// 	// add (sectorCount+1) vertices per stack
		// 	// first and last vertices have same position and normal, but different tex coords
		// 	for(int j = 0; j <= sectorCount; ++j)
		// 	{
		// 		sectorAngle = j * sectorStep;           // starting from 0 to 2pi

		// 		// vertex position (x, y, z)
		// 		x = xy * cosf(sectorAngle);             // r * cos(u) * cos(v)
		// 		y = xy * sinf(sectorAngle);             // r * cos(u) * sin(v)
		// 		vertices.push_back(x);
		// 		vertices.push_back(y);
		// 		vertices.push_back(z);
		// }

		// for(int i = 0; i < stackCount; ++i)
		// {
		// 	k1 = i * (sectorCount + 1);     // beginning of current stack
		// 	k2 = k1 + sectorCount + 1;      // beginning of next stack

		// 	for(int j = 0; j < sectorCount; ++j, ++k1, ++k2)
		// 	{
		// 		// 2 triangles per sector excluding first and last stacks
		// 		// k1 => k2 => k1+1
		// 		if(i != 0)
		// 		{
		// 			indices.push_back(k1);
		// 			indices.push_back(k2);
		// 			indices.push_back(k1 + 1);
		// 		}

		// 		// k1+1 => k2 => k2+1
		// 		if(i != (stackCount-1))
		// 		{
		// 			indices.push_back(k1 + 1);
		// 			indices.push_back(k2);
		// 			indices.push_back(k2 + 1);
		// 		}

		let mut verts = Vec::new();

		// capsule: 2 semicircles and a cylinder
		let sector_step = std::f32::consts::PI * 2.0 / SECTOR_COUNT as f32;
		let stack_step = (std::f32::consts::PI / 2.0) / STACK_COUNT as f32;

		let forward_x = LENGTH / 2.0;
		let back_x = -LENGTH / 2.0;


		// forward semicircle
		let mut semicircle_verts = Vec::new();

		for i in 0..=STACK_COUNT {
			let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
			let xy = RADIUS * stack_angle.cos();
			let z = RADIUS * stack_angle.sin();

			let color_z = stack_angle.sin();

			for j in 0..=SECTOR_COUNT {
				let sector_angle = j as f32 * sector_step;

				let x = xy * sector_angle.cos();
				let y = xy * sector_angle.sin();

				let color_x = sector_angle.cos();
				let color_y = sector_angle.sin();

				let color = [
					color_x,
					color_y,
					color_z,
				];

				semicircle_verts.push(draw::Vert::new([z + forward_x, y, x], color));
			}
		}
		for i in 0..STACK_COUNT {
			let k1 = i * (SECTOR_COUNT + 1);
			let k2 = k1 + SECTOR_COUNT + 1;

			for j in 0..SECTOR_COUNT {
				let k1 = k1 + j;
				let k2 = k2 + j;

				verts.push(semicircle_verts[k1]);
				verts.push(semicircle_verts[k2]);
				verts.push(semicircle_verts[k1 + 1]);

				verts.push(semicircle_verts[k1 + 1]);
				verts.push(semicircle_verts[k2]);
				verts.push(semicircle_verts[k2 + 1]);
			}
		}

		// middle cylinder
		for i in 0..SECTOR_COUNT {
			let sector_angle_left = i as f32 * sector_step;
			let sector_angle_right = (i + 1) as f32 * sector_step;

			let top_left = [
				forward_x,
				RADIUS * sector_angle_left.cos(),
				RADIUS * sector_angle_left.sin(),
			];
			let top_right = [
				forward_x,
				RADIUS * sector_angle_right.cos(),
				RADIUS * sector_angle_right.sin(),
			];
			let bottom_left = [
				back_x,
				RADIUS * sector_angle_left.cos(),
				RADIUS * sector_angle_left.sin(),
			];
			let bottom_right = [
				back_x,
				RADIUS * sector_angle_right.cos(),
				RADIUS * sector_angle_right.sin(),
			];

			verts.push(draw::Vert::new(top_left, COLOR));
			verts.push(draw::Vert::new(bottom_left, COLOR));
			verts.push(draw::Vert::new(top_right, COLOR));

			verts.push(draw::Vert::new(bottom_left, COLOR));
			verts.push(draw::Vert::new(bottom_right, COLOR));
			verts.push(draw::Vert::new(top_right, COLOR));
		}
		// back semicircle
		let mut semicircle_verts = Vec::new();

		for i in 0..=STACK_COUNT {
			let stack_angle = std::f32::consts::PI / 2.0 - i as f32 * stack_step;
			let xy = RADIUS * stack_angle.cos();
			let z = RADIUS * stack_angle.sin();

			let color_z = stack_angle.sin();

			for j in 0..=SECTOR_COUNT {
				let sector_angle = j as f32 * sector_step;

				let x = xy * sector_angle.cos();
				let y = xy * sector_angle.sin();

				let color_x = sector_angle.cos();
				let color_y = sector_angle.sin();

				let color = [
					color_x,
					color_y,
					color_z,
				];

				semicircle_verts.push(draw::Vert::new([-(z + forward_x), y, x], color));
			}
		}
		for i in 0..STACK_COUNT {
			let k1 = i * (SECTOR_COUNT + 1);
			let k2 = k1 + SECTOR_COUNT + 1;

			for j in 0..SECTOR_COUNT {
				let k1 = k1 + j;
				let k2 = k2 + j;

				verts.push(semicircle_verts[k1]);
				verts.push(semicircle_verts[k2]);
				verts.push(semicircle_verts[k1 + 1]);

				verts.push(semicircle_verts[k1 + 1]);
				verts.push(semicircle_verts[k2]);
				verts.push(semicircle_verts[k2 + 1]);
			}
		}


		let verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sub Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });


		Self {
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

			num_verts: verts.len(),
			verts_buffer,
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

impl draw::VertBuffer for Sub {
	fn buffer_slice(&self) -> wgpu::BufferSlice {
		self.verts_buffer.slice(..)
	}
	
	fn num_verts(&self) -> usize {
		self.num_verts
	}
}