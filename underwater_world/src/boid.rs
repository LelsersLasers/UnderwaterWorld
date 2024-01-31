use std::collections::HashMap;

use crate::{boid_obj, chunk, draw, perlin_util, sub, texture, util, world};
use cgmath::{InnerSpace, Zero, num_traits::Pow};
use rand::prelude::*;
use wgpu::util::DeviceExt;

const MIN_SPEED: f32 = 3.0;
const MAX_SPEED: f32 = 6.0;
const MIDDLE_SPEED: f32 = (MIN_SPEED + MAX_SPEED) / 2.0;

const PERCEPTION_RADIUS: f32 = 5.0;
const AVOIDANCE_RADIUS: f32 = 2.0;

const WALL_RANGE: i32 = 3;
const WALL_FORCE_MULT: f32 = 1.0;
const WALL_FORCE_DECAY: f32 = 100.0;
const RAY_DIRECTION_COUNT: usize = 20;

const MAX_STEER_FORCE: f32 = 4.0;

const DOWN_STEER_MULT: f32 = -0.1;

// Note: this is the number of boids per species
const NUM_BOIDS: usize = 100;

const WRAP_STRENGTH: f32 = 1.975;
const ISO_PADDING: f32 = 0.075;
const NEW_Z_STEP: f32 = 2.0;
const POS_RANGE_XY: f32 = 46.0;
const POS_RANGE_Z: f32 = 16.0;

const SPAT_PART_SIZE: f32 = PERCEPTION_RADIUS;

const FISH_SCALE: f32 = 0.75;



#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Species {
    Red = 0,
    Green = 1,
    Blue = 2,
}
pub const ALL_SPECIES: [Species; 3] = [Species::Red, Species::Green, Species::Blue];
const SPECIES_COUNT: usize = ALL_SPECIES.len();
const SPECIES_TEXTURE_PATHS: [&str; SPECIES_COUNT] = [
    "red.jpg",
    "green.png",
    "blue.jpg",
];

struct Boid {
    pos: cgmath::Vector3<f32>,
    vel: cgmath::Vector3<f32>,
    wall_accel: cgmath::Vector3<f32>,

    sum_flock_heading: cgmath::Vector3<f32>,    // alignment
    sum_flock_center: cgmath::Vector3<f32>,     // cohesion
    sum_flock_separation: cgmath::Vector3<f32>, // separation

    spat_part_key: (i32, i32, i32),
    spat_part_key_start: (i32, i32, i32),
    spat_part_key_end: (i32, i32, i32),

    num_flockmates: usize,

    species: Species,

    rot_mat: cgmath::Matrix4<f32>,
    inst: draw::InstanceTime,
    time: f32,
}

impl Boid {
    fn new(position: cgmath::Vector3<f32>, velocity: cgmath::Vector3<f32>, species: Species, time: f32) -> Self {
        let rot_mat = vel_to_rot_mat(velocity);
        let spat_part_key = pos_to_spat_part_key(position);
        Self {
            pos: position,
            vel: velocity,
            wall_accel: cgmath::Vector3::zero(),

            sum_flock_heading: cgmath::Vector3::zero(),
            sum_flock_center: cgmath::Vector3::zero(),
            sum_flock_separation: cgmath::Vector3::zero(),

            spat_part_key,
            spat_part_key_start: spat_part_key,
            spat_part_key_end: spat_part_key,

            num_flockmates: 0,

            species,

            rot_mat,
            inst: pos_rot_mat_to_inst(position, rot_mat, time),
            time,
        }
    }

    fn wrap(&mut self, sub: &sub::Sub, perlin: &noise::Perlin) -> cgmath::Vector3<f32> {
        let mut accel = cgmath::Vector3::zero();

        let sub_pos = sub.pos();

        let sub_offset = sub_pos - self.pos;
        if sub_offset.z < -POS_RANGE_Z {
            let sub_force = self.steer_towards(-cgmath::Vector3::unit_z());
            accel += sub_force;
        }

        let sub_offset_xy = sub_offset.truncate();
        let sub_distance_xy = sub_offset_xy.magnitude();
        if sub_distance_xy > POS_RANGE_XY {
            let new_x = sub_offset_xy.x * WRAP_STRENGTH + self.pos.x;
            let new_y = sub_offset_xy.y * WRAP_STRENGTH + self.pos.y;

            let mut new_z = self.pos.z;
            let mut new_z_in_wall = true;
            while new_z_in_wall {
                let iso = perlin_util::iso_at(
                    perlin,
                    new_x as f64 / chunk::CHUNK_SIZE as f64,
                    new_y as f64 / chunk::CHUNK_SIZE as f64,
                    new_z as f64 / chunk::CHUNK_SIZE as f64,
                );
                if iso > chunk::ISO_LEVEL + ISO_PADDING {
                    new_z_in_wall = false;
                } else {
                    new_z += NEW_Z_STEP;
                }
            }

            let new_pos = cgmath::Vector3::new(new_x, new_y, new_z);
            self.pos = new_pos;
        }

        accel
    }

    fn update(&mut self, perlin: &noise::Perlin, sub: &sub::Sub, world: &world::World, avoidance_rays: &[cgmath::Vector3<f32>], delta: f32) {
        let mut accel = cgmath::Vector3::zero();

        if self.num_flockmates > 0 {
            let center_offset = self.sum_flock_center / self.num_flockmates as f32 - self.pos;

            let separation_force = self.steer_towards(self.sum_flock_separation);
            let alignment_force = self.steer_towards(self.sum_flock_heading);
            let cohesion_force = self.steer_towards(center_offset);

            accel += separation_force;
            accel += alignment_force;
            accel += cohesion_force;
        }

        let wrap_force = self.wrap(sub, perlin);
        accel += wrap_force;

        let down_force = self.steer_towards(cgmath::Vector3::unit_z()) * DOWN_STEER_MULT;
        accel += down_force;

        let mut all_tris = Vec::new();

        let world_start_x = ((self.pos.x - WALL_RANGE as f32) / chunk::CHUNK_SIZE as f32).floor() as i32;
        let world_start_y = ((self.pos.y - WALL_RANGE as f32) / chunk::CHUNK_SIZE as f32).floor() as i32;
        let world_start_z = ((self.pos.z - WALL_RANGE as f32) / chunk::CHUNK_SIZE as f32).floor() as i32;

        let world_end_x = ((self.pos.x + WALL_RANGE as f32) / chunk::CHUNK_SIZE as f32).floor() as i32;
        let world_end_y = ((self.pos.y + WALL_RANGE as f32) / chunk::CHUNK_SIZE as f32).floor() as i32;
        let world_end_z = ((self.pos.z + WALL_RANGE as f32) / chunk::CHUNK_SIZE as f32).floor() as i32;

        for a in world_start_x..=world_end_x {
            let local_x = self.pos.x - a as f32 * chunk::CHUNK_SIZE as f32;
            let local_percent_x = local_x / chunk::CHUNK_SIZE as f32;

            for b in world_start_y..=world_end_y {
                let local_y = self.pos.y - b as f32 * chunk::CHUNK_SIZE as f32;
                let local_percent_y = local_y / chunk::CHUNK_SIZE as f32;

                for c in world_start_z..=world_end_z {
                    let chunk_pos = (a, b, c);

                    let chunk = match world.get_chunk(chunk_pos) {
                        Some(chunk) => chunk,
                        None => continue,
                    };

                    let local_z = self.pos.z - c as f32 * chunk::CHUNK_SIZE as f32;
                    let local_percent_z = local_z / chunk::CHUNK_SIZE as f32;

                    let local_pos_percent = (local_percent_x, local_percent_y, local_percent_z);
                    let tris = chunk.tris_around(local_pos_percent, WALL_RANGE);

                    all_tris.extend(tris);
                }   
            }
        }

        let v_norm = util::safe_normalize(self.vel);

        let lower_ray = util::safe_normalize(cgmath::Vector3::new(1.0, 0.0, -0.5));
        let lower_ray = (self.rot_mat * lower_ray.extend(1.0)).truncate();

        let vs = [lower_ray, v_norm];

        let heading_for_collision = all_tris.iter().any(|tri| {
            vs.iter().any(|v| {
                let t = tri.intersects(self.pos, *v, WALL_RANGE as f32);
                match t {
                    Some(t) => t < WALL_RANGE as f32,
                    None => false,
                }
            })
        });

        let old_wall_accel = self.wall_accel;
        if heading_for_collision {
            'ray: for ray in avoidance_rays.iter() {
                let ray = (self.rot_mat * ray.extend(1.0)).truncate();

                let safe_dir = all_tris.iter().all(|tri| {
                    let t = tri.intersects(self.pos, ray, WALL_RANGE as f32);
                    // match t {
                    //     Some(t) => t > WALL_RANGE as f32,
                    //     None => true,
                    // }
                    t.is_none()
                });

                if safe_dir {
                    let force = self.steer_towards(ray) * WALL_FORCE_MULT;
                    // TODO: does this need a `* delta`
                    self.wall_accel += force;
                    break 'ray;
                }
            }
        }

        let wall_decay = WALL_FORCE_DECAY * delta;
        if util::vec3_eq(old_wall_accel, self.wall_accel) {
            if self.wall_accel.magnitude() < wall_decay {
                self.wall_accel = cgmath::Vector3::zero();
            } else {
                self.wall_accel -= util::safe_normalize(self.wall_accel) * wall_decay;
            }
        }

        accel += self.wall_accel;        
        self.vel += accel * delta;
        let target_speed = self.vel.magnitude().clamp(MIN_SPEED, MAX_SPEED);
        self.vel = util::safe_normalize_to(self.vel, target_speed);

        self.pos += self.vel * delta;

        let wiggle = target_speed / MIDDLE_SPEED;
        self.time += delta * wiggle;

        self.rot_mat = vel_to_rot_mat(self.vel);
        self.inst = pos_rot_mat_to_inst(self.pos, self.rot_mat, self.time);

        self.spat_part_key = pos_to_spat_part_key(self.pos);
        let spat_part_size_vec = cgmath::Vector3::new(SPAT_PART_SIZE, SPAT_PART_SIZE, SPAT_PART_SIZE);
        self.spat_part_key_start = pos_to_spat_part_key(self.pos - spat_part_size_vec);
        self.spat_part_key_end = pos_to_spat_part_key(self.pos + spat_part_size_vec);
    }

    fn steer_towards(&self, target: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        let v = util::safe_normalize_to(target, MAX_SPEED) - self.vel;
        let v_mag = v.magnitude().min(MAX_STEER_FORCE);
        util::safe_normalize_to(v, v_mag)
    }
}

fn vel_to_rot_mat(vel: cgmath::Vector3<f32>) -> cgmath::Matrix4<f32> {
    let xy_rot_quat = cgmath::Quaternion::from_arc(
        cgmath::Vector3::unit_x(),
        util::safe_normalize(cgmath::Vector3::new(vel.x, vel.y, 0.0)),
        None,
    );
    let z_rot_quat = cgmath::Quaternion::from_arc(
        util::safe_normalize(cgmath::Vector3::new(vel.x, vel.y, 0.0)),
        util::safe_normalize(cgmath::Vector3::new(vel.x, vel.y, vel.z)),
        None,
    );
    cgmath::Matrix4::from(z_rot_quat) * cgmath::Matrix4::from(xy_rot_quat)
}

fn pos_rot_mat_to_inst(pos: cgmath::Vector3<f32>, rot_mat: cgmath::Matrix4<f32>, time: f32) -> draw::InstanceTime {
    let mat = cgmath::Matrix4::from_translation(pos) * rot_mat;
    draw::InstanceTime::new(mat, time)
}

fn pos_to_spat_part_key(pos:cgmath::Vector3<f32>) -> (i32, i32, i32) {
    let x = (pos.x / SPAT_PART_SIZE).floor() as i32;
    let y = (pos.y / SPAT_PART_SIZE).floor() as i32;
    let z = (pos.z / SPAT_PART_SIZE).floor() as i32;
    (x, y, z)
}

fn random_pos(rng: &mut ThreadRng, perlin: &noise::Perlin, sub: &sub::Sub) -> cgmath::Vector3<f32> {
    let sub_pos = sub.pos();

    loop {
        let x_range = (sub_pos.x - POS_RANGE_XY)..(sub_pos.x + POS_RANGE_XY);
        let y_range = (sub_pos.y - POS_RANGE_XY)..(sub_pos.y + POS_RANGE_XY);
        let z_range = (sub_pos.z - POS_RANGE_XY)..(sub_pos.z + POS_RANGE_Z);
        let pos = cgmath::Vector3::new(
            rng.gen_range(x_range),
            rng.gen_range(y_range),
            rng.gen_range(z_range),
        );

        let iso = perlin_util::iso_at(
            perlin,
            pos.x as f64 / chunk::CHUNK_SIZE as f64,
            pos.y as f64 / chunk::CHUNK_SIZE as f64,
            pos.z as f64 / chunk::CHUNK_SIZE as f64,
        );
        if iso > chunk::ISO_LEVEL + ISO_PADDING {
            return pos;
        }
    }
}


struct PerSpecies {
    diffuse_bind_group: wgpu::BindGroup,

    insts: Vec<draw::InstanceTime>,

    verts_buffer: wgpu::Buffer,
    inst_buffer: wgpu::Buffer,

    num_verts: usize,
}

pub struct BoidManager {
    boids: Vec<Boid>,
    spat_part: HashMap<(i32, i32, i32), Vec<usize>>,
    per_species: Vec<PerSpecies>,
    avoidance_rays: Vec<cgmath::Vector3<f32>>,
}
impl BoidManager {
    pub fn new(
        sub: &sub::Sub,
        perlin: &noise::Perlin,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut boids = Vec::with_capacity(NUM_BOIDS * SPECIES_COUNT);
        let mut spat_part: HashMap<(i32, i32, i32), Vec<usize>> = HashMap::new();
        let mut per_species = Vec::new();

        let diffuse_bytes_red = include_bytes!("red.jpg");
        let diffuse_bytes_green = include_bytes!("green.png");
        let diffuse_bytes_blue = include_bytes!("blue.jpg");

        let mut boid_i = 0;

        for species in &ALL_SPECIES {
            let mut insts = Vec::with_capacity(NUM_BOIDS);
            for _ in 0..NUM_BOIDS {
                let position = random_pos(&mut rng, perlin, sub);
                let velocity = util::safe_normalize_to(cgmath::Vector3::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                ), rng.gen_range(MIN_SPEED..MAX_SPEED));

                let time = rng.gen_range(0.0..std::f32::consts::PI * 2.0);
                let boid = Boid::new(position, velocity, *species, time);

                let spat_part_key = boid.spat_part_key;
                match spat_part.get_mut(&spat_part_key) {
                    Some(boid_is) => boid_is.push(boid_i),
                    None => { spat_part.insert(spat_part_key, vec![boid_i]); },
                };

                insts.push(boid.inst);
                boids.push(boid);
                boid_i += 1;
            }

            let diffuse_texture = match species {
                Species::Red   => texture::Texture::from_bytes(device, queue, diffuse_bytes_red,   SPECIES_TEXTURE_PATHS[*species as usize]).unwrap(),
                Species::Green => texture::Texture::from_bytes(device, queue, diffuse_bytes_green, SPECIES_TEXTURE_PATHS[*species as usize]).unwrap(),
                Species::Blue  => texture::Texture::from_bytes(device, queue, diffuse_bytes_blue,  SPECIES_TEXTURE_PATHS[*species as usize]).unwrap(),
            };

            let diffuse_bind_group = device.create_bind_group(
                &wgpu::BindGroupDescriptor {
                    layout: texture_bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&diffuse_texture.view),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::Sampler(&diffuse_texture.sampler),
                        }
                    ],
                    label: Some("diffuse_bind_group"),
                }
            );

            let mut vert_poses = Vec::new();
            let mut vert_txs = Vec::new();
            let mut verts = Vec::new();

            let mut highest_v: f32 = 0.0;

            let obj = match species {
                Species::Red   => boid_obj::red_obj(),
                Species::Green => boid_obj::green_obj(),
                Species::Blue  => boid_obj::blue_obj(),
            };

            for line in obj.lines() {
                let mut split = line.split_whitespace();
                let first = split.next();
                match first {
                    Some("v") => {
                        let x: f32 = split.next().unwrap().parse().unwrap();
                        let y: f32 = split.next().unwrap().parse().unwrap();
                        let z: f32 = split.next().unwrap().parse().unwrap();

                        let pos = match species {
                            Species::Red   => [z, x, y],
                            Species::Green => [z, x, -y],
                            Species::Blue  => [-x, y, z],
                        };
                        vert_poses.push(pos);

                        highest_v = highest_v.max(x.abs()).max(y.abs()).max(z.abs());
                    }
                    Some("vt") => {
                        let x: f32 = split.next().unwrap().parse().unwrap();
                        let y: f32 = split.next().unwrap().parse().unwrap();
                        vert_txs.push([x, y]);
                    }
                    Some("f") => {
                        // 0 (i) (i + 1)  [for i in 1..(n - 2)]
                        let remaining = split.collect::<Vec<&str>>();
                        let n = remaining.len();
                        for i in 1..(n - 1) {
                            let mut i0 = remaining[0].split('/');
                            let mut i1 = remaining[i].split('/');
                            let mut i2 = remaining[i + 1].split('/');

                            let v0 = i0.next().unwrap().parse::<usize>().unwrap() - 1;
                            let v1 = i1.next().unwrap().parse::<usize>().unwrap() - 1;
                            let v2 = i2.next().unwrap().parse::<usize>().unwrap() - 1;

                            let vt0 = i0.next().unwrap().parse::<usize>().unwrap() - 1;
                            let vt1 = i1.next().unwrap().parse::<usize>().unwrap() - 1;
                            let vt2 = i2.next().unwrap().parse::<usize>().unwrap() - 1;

                            verts.push(draw::VertTex::new(vert_poses[v0], vert_txs[vt0]));
                            verts.push(draw::VertTex::new(vert_poses[v1], vert_txs[vt1]));
                            verts.push(draw::VertTex::new(vert_poses[v2], vert_txs[vt2]));
                        }
                    }
                    _ => {}
                }
            }

            verts.iter_mut().for_each(|v| {
                v.pos[0] *= FISH_SCALE / highest_v;
                v.pos[1] *= FISH_SCALE / highest_v;
                v.pos[2] *= FISH_SCALE / highest_v;
            });

            let verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", species)),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });

            let inst_buffer = device.create_buffer_init(
                &wgpu::util::BufferInitDescriptor {
                    label: Some("Instance Buffer"),
                    contents: bytemuck::cast_slice(&insts),
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                }
            );

            per_species.push(PerSpecies {
                diffuse_bind_group,

                insts,

                verts_buffer,
                inst_buffer,

                num_verts: verts.len(),
            });
        }

        let mut avoidance_rays = Vec::with_capacity(RAY_DIRECTION_COUNT);
        let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
        let angle_increment = std::f32::consts::PI * 2.0 / golden_ratio;

        for i in 0..RAY_DIRECTION_COUNT {
            let t = (i as f32) / RAY_DIRECTION_COUNT as f32;
            let inclination = (1.0 - 2.0 * t).acos();
            let azimuth = angle_increment * i as f32;

            let x = inclination.sin() * azimuth.cos();
            let y = inclination.sin() * azimuth.sin();
            let z = inclination.cos();
            
            let v = cgmath::Vector3::new(z, y, x);
            let v_norm = util::safe_normalize(v);
            avoidance_rays.push(v_norm);
        }


        // let center = cgmath::Vector3::new(0.5, 0.5, 0.5);
        // let corners = [
        //     cgmath::Vector3::new(0.0, 0.0, 0.0),
        //     cgmath::Vector3::new(1.0, 0.0, 0.0),
        //     cgmath::Vector3::new(0.0, 1.0, 0.0),
        //     cgmath::Vector3::new(1.0, 1.0, 0.0),
        //     cgmath::Vector3::new(0.0, 0.0, 1.0),
        //     cgmath::Vector3::new(1.0, 0.0, 1.0),
        //     cgmath::Vector3::new(0.0, 1.0, 1.0),
        //     cgmath::Vector3::new(1.0, 1.0, 1.0),
        // ];
        // let corner_vectors = corners.iter().map(|corner| {
        //     let offset = corner - center;
        //     util::safe_normalize(offset)
        // }).collect::<Vec<_>>();
        // let face_vectors = [
        //     cgmath::Vector3::new(0.0, 0.0, -1.0),
        //     cgmath::Vector3::new(0.0, 0.0, 1.0),
        //     cgmath::Vector3::new(0.0, -1.0, 0.0),
        //     cgmath::Vector3::new(0.0, 1.0, 0.0),
        //     cgmath::Vector3::new(-1.0, 0.0, 0.0),
        //     cgmath::Vector3::new(1.0, 0.0, 0.0),
        // ];
        // let mut avoidance_rays = corner_vectors.clone();
        // avoidance_rays.extend(face_vectors);

        avoidance_rays.sort_unstable_by(|ray1, ray2| {
            let angle1 = cgmath::Vector3::unit_x().angle(*ray1);
            let angle2 = cgmath::Vector3::unit_x().angle(*ray2);
            angle1.partial_cmp(&angle2).unwrap()
        });

        Self { boids, spat_part, per_species, avoidance_rays }
    }

    fn boids_near(&self, boid_i: usize) -> Vec<usize> {
        let boid = &self.boids[boid_i];
        let mut boids_near = Vec::new();

        let start = boid.spat_part_key_start;
        let end = boid.spat_part_key_end;

        for x in start.0..=end.0 {
            for y in start.1..=end.1 {
                for z in start.2..=end.2 {
                    let key = (x, y, z);
                    if let Some(boid_is) = self.spat_part.get(&key) {
                        boids_near.extend(boid_is);
                    }
                }
            }
        }

        boids_near
    }

    pub fn update(&mut self, queue: &wgpu::Queue, perlin: &noise::Perlin, sub: &sub::Sub, world: &world::World, delta: f32) {
        for i in 0..self.boids.len() {
            self.boids[i].num_flockmates = 0;
            self.boids[i].sum_flock_heading = cgmath::Vector3::zero();
            self.boids[i].sum_flock_center = cgmath::Vector3::zero();
            self.boids[i].sum_flock_separation = cgmath::Vector3::zero();

            let nearby = self.boids_near(i);
            for j in nearby {
                if i == j {
                    continue;
                }

                let offset = self.boids[j].pos - self.boids[i].pos;
                let distance = offset.magnitude();

                let i_species = self.boids[i].species;
                let j_species = self.boids[j].species;

                if distance < PERCEPTION_RADIUS  {
                    if i_species == j_species {
                        self.boids[i].num_flockmates += 1;
                        
                        let boid_j_vel = self.boids[j].vel;
                        self.boids[i].sum_flock_heading += boid_j_vel;
    
                        let boid_j_pos = self.boids[j].pos;
                        self.boids[i].sum_flock_center += boid_j_pos;
                    }
    
                    if i_species != j_species || distance < AVOIDANCE_RADIUS {
                        self.boids[i].sum_flock_separation -= offset / distance.pow(2);
                    }
                }
            }

            let offset = sub.pos() - self.boids[i].pos;
            let distance = offset.magnitude();

            if distance < PERCEPTION_RADIUS {
                self.boids[i].sum_flock_separation -= offset / distance.pow(2);
            }
        }

        self.spat_part.clear();

        for (boid_i, boid) in self.boids.iter_mut().enumerate() {
            boid.update(perlin, sub, world, &self.avoidance_rays, delta);

            let spat_part_key = boid.spat_part_key;
            match self.spat_part.get_mut(&spat_part_key) {
                Some(boid_is) => boid_is.push(boid_i),
                None => { self.spat_part.insert(spat_part_key, vec![boid_i]); },
            };

            let species_i = boid_i % NUM_BOIDS;
            self.per_species[boid.species as usize].insts[species_i] = boid.inst;
        }

        for per_species in self.per_species.iter() {
            queue.write_buffer(&per_species.inst_buffer, 0, bytemuck::cast_slice(&per_species.insts));
        }
    }

    pub fn verts_buffer_slice(&self, species: Species) -> wgpu::BufferSlice { self.per_species[species as usize].verts_buffer.slice(..) }
    pub fn inst_buffer_slice(&self, species: Species) -> wgpu::BufferSlice { self.per_species[species as usize].inst_buffer.slice(..) }
    pub fn diffuse_bind_group(&self, species: Species) -> &wgpu::BindGroup { &self.per_species[species as usize].diffuse_bind_group }
    pub fn num_verts(&self, species: Species) -> usize { self.per_species[species as usize].num_verts }
    pub fn num_inst(&self, _species: Species) -> usize { NUM_BOIDS }
}
