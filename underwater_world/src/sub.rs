use crate::{camera, chunk, draw, sub_obj, util};
use wgpu::util::DeviceExt;
use cgmath::{EuclideanSpace, One, Rotation, Rotation3};
use noise::NoiseFn;
use std::collections::HashMap;

const MIN_SPEED: f32 = 0.5;
const MAX_SPEED: f32 = 5.0;
const MIDDLE_SPEED: f32 = 4.0;
const ACCELERATION: f32 = 5.0;

const EXTRA_PROP_ROT: f32 = 2.0;

const MAX_TURN_SPEED: f32 = std::f32::consts::PI / 6.0;
const MAX_DIVE_SPEED: f32 = std::f32::consts::PI / 3.0;
const TURN_ACCELERATION: f32 = std::f32::consts::PI;
const TURN_DECAY: f32 = 3.0;

const TARGET_DOWN: f32 = 0.6;
const HORIZONTAL_OFFSET: f32 = 7.0;
const VERTICAL_OFFSET: f32 = 6.0;

const LIGHT_DOWN_OFFSET: f32 = 0.25;

const PROP_START_X: f32 = -120.0;
const SUB_MODEL_SCALE: f32 = 2.5;
const PERLIN_FACTOR: f32 = 2.0;

const CAMERA_FOLLOW_SPEED: f32 = 10.0;
const START_Y_OFFSET: f32 = 0.5 * chunk::CHUNK_SIZE as f32;
const START_Z_OFFSET: f32 = 0.75 * chunk::CHUNK_SIZE as f32;

const MAX_Z: f32 = chunk::CHUNK_SIZE as f32 * 2.0;
const MIN_Z: f32 = chunk::CHUNK_SIZE as f32 * -1.5;

struct Keys {
	w_down: bool,
	s_down: bool,
	a_down: bool,
	d_down: bool,
	q_down: bool,
	e_down: bool,
	space_down: bool,
	control_down: bool,
    r_down: bool,
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
            r_down: false,
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
                    winit::event::VirtualKeyCode::R | winit::event::VirtualKeyCode::Return => {
                        self.r_down = pressed;
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
	pos: cgmath::Vector3<f32>,

    up: cgmath::Vector3<f32>,
    forward: cgmath::Vector3<f32>,
    right: cgmath::Vector3<f32>,

	overall_rotation: cgmath::Quaternion<f32>,

	yaw: f32,
	pitch: f32,
    roll: f32,

	yaw_speed: f32,
	pitch_speed: f32,
	roll_speed: f32,

    prop_rot: f32,

	speed: f32,

	keys: Keys,

	verts_buffer: wgpu::Buffer,
    prop_verts_buffer: wgpu::Buffer,

	inst_buffer: wgpu::Buffer,
    prop_inst_buffer: wgpu::Buffer,

    num_verts: usize,
    num_prop_verts: usize,

    color_mix: f32,
}

impl Sub {
	pub fn new(device: &wgpu::Device, perlin: &noise::Perlin) -> Self {
        //--------------------------------------------------------------------//
        let mut mats = HashMap::new();

        for line in sub_obj::sub_mat().lines() {
            let mut split = line.split_whitespace();

            let first = split.next();
            match first {
                Some(key) => {
                    let r = split.next().unwrap().parse::<f32>().unwrap();
                    let g = split.next().unwrap().parse::<f32>().unwrap();
                    let b = split.next().unwrap().parse::<f32>().unwrap();

                    mats.insert(key, [r, g, b]);
                }
                _ => continue,
            }
        }
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let mut vert_poses = Vec::new();

		let mut verts = Vec::new();
        let mut prop_verts = Vec::new();

        let mut active_color = [0.0, 0.0, 0.0];

        let mut highest_v: f32 = 0.0;

        for line in sub_obj::sub_obj().lines() {
            let mut split = line.split_whitespace();
            let first = split.next();
            match first {
                Some("v") => {
                    let x: f32 = split.next().unwrap().parse().unwrap();
                    let y: f32 = split.next().unwrap().parse().unwrap();
                    let z: f32 = split.next().unwrap().parse().unwrap();
                    vert_poses.push([-z, x, y]);

                    highest_v = highest_v.max(x.abs()).max(y.abs()).max(z.abs());
                }
                Some("f") => {
                    // 0 (i) (i + 1)  [for i in 1..(n - 2)]
                    let remaining = split.collect::<Vec<&str>>();
                    let n = remaining.len();
                    for i in 1..(n - 1) {
                        let i0 = remaining[0].split('/').next().unwrap().parse::<usize>().unwrap() - 1;
                        let i1 = remaining[i].split('/').next().unwrap().parse::<usize>().unwrap() - 1;
                        let i2 = remaining[i + 1].split('/').next().unwrap().parse::<usize>().unwrap() - 1;

                        let v0 = vert_poses[i0];
                        let v1 = vert_poses[i1];
                        let v2 = vert_poses[i2];

                        if v0[0] < PROP_START_X && v1[0] < PROP_START_X && v2[0] < PROP_START_X {
                            prop_verts.push(draw::VertColor::new(v0, active_color));
                            prop_verts.push(draw::VertColor::new(v1, active_color));
                            prop_verts.push(draw::VertColor::new(v2, active_color));
                        } else {
                            verts.push(draw::VertColor::new(v0, active_color));
                            verts.push(draw::VertColor::new(v1, active_color));
                            verts.push(draw::VertColor::new(v2, active_color));
                        }
                    }
                }
                Some("usemtl") => {
                    let mut mat = split.next().unwrap();
                    if mat == "default" {
                        mat = "Mat.2";
                    }
                    let active_color_rgb = *mats.get(mat).unwrap_or(&active_color);
                    active_color = util::to_srgb_decimal(active_color_rgb);
                }
                _ => {}
            }
        }

        verts.iter_mut().for_each(|v| {
            v.pos[0] *= SUB_MODEL_SCALE / highest_v;
            v.pos[1] *= SUB_MODEL_SCALE / highest_v;
            v.pos[2] *= SUB_MODEL_SCALE / highest_v;

            let p = perlin.get([v.pos[0] as f64, v.pos[1] as f64, v.pos[2] as f64]) as f32;
            v.color[0] += p / PERLIN_FACTOR;
            v.color[1] += p / PERLIN_FACTOR;
            v.color[2] += p / PERLIN_FACTOR;
        });

        prop_verts.iter_mut().for_each(|v| {
            v.pos[0] *= SUB_MODEL_SCALE / highest_v;
            v.pos[1] *= SUB_MODEL_SCALE / highest_v;
            v.pos[2] *= SUB_MODEL_SCALE / highest_v;

            let p = perlin.get([v.pos[0] as f64, v.pos[1] as f64, v.pos[2] as f64]) as f32;
            v.color[0] += p / PERLIN_FACTOR;
            v.color[1] += p / PERLIN_FACTOR;
            v.color[2] += p / PERLIN_FACTOR;
        });
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
		let verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sub Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let prop_verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sub Propeller Vertex Buffer"),
            contents: bytemuck::cast_slice(&prop_verts),
            usage: wgpu::BufferUsages::VERTEX,
        });

		let inst = draw::Instance::identity();
		let inst_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&[inst]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );

        let prop_inst = draw::Instance::identity();
        let prop_inst_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Propellers Instance Buffer"),
                contents: bytemuck::cast_slice(&[prop_inst]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }
        );
        //--------------------------------------------------------------------//

		Self {
			pos: cgmath::Vector3::new(0.0, START_Y_OFFSET, START_Z_OFFSET),

            up: cgmath::Vector3::unit_z(),
            forward: cgmath::Vector3::unit_x(),
            right: cgmath::Vector3::unit_y(),

			overall_rotation: cgmath::Quaternion::one(),

			yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,

            yaw_speed: 0.0,
            pitch_speed: 0.0,
            roll_speed: 0.0,

            prop_rot: 0.0,
			
			speed: MIDDLE_SPEED,

			keys: Keys::new(),

			verts_buffer,
            prop_verts_buffer,

			inst_buffer,
            prop_inst_buffer,

            num_verts: verts.len(),
            num_prop_verts: prop_verts.len(),

            color_mix: util::create_mix_ratio(MIN_Z, MAX_Z, START_Z_OFFSET),
		}
	}

    fn decay_turn_rates(&mut self, delta: f32) {
        let min_turn_decay = TURN_DECAY * MAX_TURN_SPEED * delta;

        if self.keys.w_down || self.keys.s_down {
			self.pitch_speed = self.pitch_speed.clamp(-MAX_DIVE_SPEED, MAX_DIVE_SPEED);
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

	pub fn update(&mut self, queue: &wgpu::Queue, delta: f32) -> bool {
		if self.keys.w_down { self.pitch_speed -= TURN_ACCELERATION * delta; }
		if self.keys.s_down { self.pitch_speed += TURN_ACCELERATION * delta; }
		if self.keys.a_down { self.yaw_speed   += TURN_ACCELERATION * delta; }
		if self.keys.d_down { self.yaw_speed   -= TURN_ACCELERATION * delta; }
		if self.keys.q_down { self.roll_speed  -= TURN_ACCELERATION * delta; }
		if self.keys.e_down { self.roll_speed  += TURN_ACCELERATION * delta; }
		if self.keys.space_down { self.speed   += ACCELERATION * delta; }
		if self.keys.control_down { self.speed -= ACCELERATION * delta; }

        if self.keys.r_down {
            self.speed = MIDDLE_SPEED;
            self.pos.z = START_Z_OFFSET;

            self.pitch = 0.0;
            self.yaw = 0.0;
            self.roll = 0.0;

            self.pitch_speed = 0.0;
            self.yaw_speed = 0.0;
            self.roll_speed = 0.0;

            self.overall_rotation = cgmath::Quaternion::one();

            self.forward = cgmath::Vector3::unit_x();
            self.up = cgmath::Vector3::unit_z();
            self.right = cgmath::Vector3::unit_y();
        } else {
            self.decay_turn_rates(delta);

            self.speed = self.speed.clamp(MIN_SPEED, MAX_SPEED);

            let angle_change_mod = (self.speed / MIDDLE_SPEED).clamp(0.0, 1.0);

            let pitch_change = self.pitch_speed * delta * angle_change_mod;
            self.pitch += pitch_change;

            let yaw_change = self.yaw_speed * delta * angle_change_mod;
            self.yaw += yaw_change;

            let roll_change = self.roll_speed * delta * angle_change_mod;
            self.roll += roll_change;

            self.prop_rot += self.speed * EXTRA_PROP_ROT * delta;


            let pitch_change_quat = cgmath::Quaternion::from_axis_angle(self.right, cgmath::Rad(pitch_change));
            let yaw_change_quat = cgmath::Quaternion::from_axis_angle(self.up, cgmath::Rad(yaw_change));
            let roll_change_quat = cgmath::Quaternion::from_axis_angle(self.forward, cgmath::Rad(roll_change));

            let overall_change_quat = yaw_change_quat * pitch_change_quat * roll_change_quat;
            self.overall_rotation = overall_change_quat * self.overall_rotation;

            self.forward = overall_change_quat.rotate_vector(self.forward);
            self.up = overall_change_quat.rotate_vector(self.up);
            self.right = overall_change_quat.rotate_vector(self.right);
            
            self.pos += self.forward * self.speed * delta;
            self.pos.z = self.pos.z.clamp(MIN_Z, MAX_Z);
        }

		let inst_mat = cgmath::Matrix4::from_translation(self.pos) * cgmath::Matrix4::from(self.overall_rotation);
		let inst = draw::Instance::new(inst_mat);
		queue.write_buffer(&self.inst_buffer, 0, bytemuck::cast_slice(&[inst]));

        let prop_inst_mat = inst_mat * cgmath::Matrix4::from_angle_x(cgmath::Rad(self.prop_rot));
        let prop_inst = draw::Instance::new(prop_inst_mat);
        queue.write_buffer(&self.prop_inst_buffer, 0, bytemuck::cast_slice(&[prop_inst]));

        self.color_mix = util::create_mix_ratio(MIN_Z, MAX_Z, self.pos.z);

        self.keys.r_down
	}

	pub fn update_camera(&self, camera: &mut camera::Camera, delta: f32) {
		let eye_goal = self.pos - self.forward * HORIZONTAL_OFFSET + self.up * VERTICAL_OFFSET;
		let eye_diff = eye_goal - camera.eye.to_vec();
		let eye_move = eye_diff * delta * CAMERA_FOLLOW_SPEED;
		camera.eye += eye_move;
        // camera.eye = eye_goal;

        let target_goal = eye_goal + self.forward - self.up * TARGET_DOWN;
        let target_diff = target_goal - camera.target.to_vec();
        let target_move = target_diff * delta * CAMERA_FOLLOW_SPEED;
        camera.target += target_move;
        // camera.target = target_goal;

		let up_diff = self.up - camera.up;
		let up_move = up_diff * delta * CAMERA_FOLLOW_SPEED;
		camera.up += up_move;
        // camera.up = self.up;
        
		camera.set_sub_pos(self.pos.into());

        let light_forward = self.forward - self.up * LIGHT_DOWN_OFFSET;
        camera.set_sub_dir(light_forward.into());

        camera.update_uniform();
	}

	pub fn process_events(&mut self, event: &winit::event::WindowEvent) -> bool {
		self.keys.process_events(event)
    }

    pub fn chunk(&self) -> (i32, i32, i32) {
         (
            (self.pos.x / chunk::CHUNK_SIZE as f32).floor() as i32,
            (self.pos.y / chunk::CHUNK_SIZE as f32).floor() as i32,
            (self.pos.z / chunk::CHUNK_SIZE as f32).floor() as i32,
        )
    }

    pub fn pos(&self) -> cgmath::Vector3<f32> { self.pos }
    pub fn bearing(&self) -> cgmath::Vector3<f32> { self.forward }

    pub fn t(&self) -> f32 { self.color_mix }

    pub fn verts_buffer_slice(&self) -> wgpu::BufferSlice { self.verts_buffer.slice(..) }
    pub fn prop_verts_buffer_slice(&self) -> wgpu::BufferSlice { self.prop_verts_buffer.slice(..) }

	pub fn inst_buffer_slice(&self) -> wgpu::BufferSlice { self.inst_buffer.slice(..) }
    pub fn prop_inst_buffer_slice(&self) -> wgpu::BufferSlice { self.prop_inst_buffer.slice(..) }

    pub fn num_verts(&self) -> usize { self.num_verts }
    pub fn num_prop_verts(&self) -> usize { self.num_prop_verts }
}