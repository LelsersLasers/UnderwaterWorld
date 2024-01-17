use crate::{boid_obj, chunk, draw, sub, texture, world};
use cgmath::{InnerSpace, Zero, EuclideanSpace, num_traits::Pow};
use rand::prelude::*;
use wgpu::util::DeviceExt;

const MIN_SPEED: f32 = 3.0;
const MAX_SPEED: f32 = 7.0;

const PERCEPTION_RADIUS: f32 = 8.0;
const AVOIDANCE_RADIUS: f32 = 2.0;

const MAX_STEER_FORCE: f32 = 10.0;

const NUM_BOIDS: usize = 400;

const FISH_SCALE: f32 = 1.5;

const POS_RANGE: f32 = chunk::CHUNK_SIZE as f32 * world::VIEW_DIST as f32;
const POS_RANGE_Z: f32 = chunk::CHUNK_SIZE as f32;


#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Species {
    Red = 0,
    Green = 1,
}
pub const ALL_SPECIES: [Species; 2] = [Species::Red, Species::Green];
const SPECIES_COUNT: usize = ALL_SPECIES.len();
const SPECIES_TEXTURE_PATHS: [&str; SPECIES_COUNT] = [
    "red.jpg",
    "green.png",
];

struct Boid {
    position: cgmath::Vector3<f32>,
    velocity: cgmath::Vector3<f32>,

    sum_flock_heading: cgmath::Vector3<f32>,    // alignment
    sum_flock_center: cgmath::Vector3<f32>,     // cohesion
    sum_flock_separation: cgmath::Vector3<f32>, // separation

    num_flockmates: usize,

    species: Species,

    inst: draw::Instance,
}

impl Boid {
    fn new(position: cgmath::Vector3<f32>, velocity: cgmath::Vector3<f32>, species: Species) -> Self {
        Self {
            position,
            velocity,

            sum_flock_heading: cgmath::Vector3::zero(),
            sum_flock_center: cgmath::Vector3::zero(),
            sum_flock_separation: cgmath::Vector3::zero(),

            num_flockmates: 0,

            species,

            inst: pos_vel_to_inst(position, velocity),
        }
    }

    fn update(&mut self, sub: &sub::Sub, delta: f32) {
        let mut acceleration = cgmath::Vector3::zero();

        if self.num_flockmates > 0 {
            let center_offset = self.sum_flock_center / self.num_flockmates as f32 - self.position;

            let separation_force = self.steer_towards(self.sum_flock_separation);
            let alignment_force = self.steer_towards(self.sum_flock_heading);
            let cohesion_force = self.steer_towards(center_offset);

            acceleration += separation_force;
            acceleration += alignment_force;
            acceleration += cohesion_force;
        }

        let sub_offset = sub.pos().to_vec() - self.position;
        let sub_distance = sub_offset.magnitude();
        if sub_distance > POS_RANGE || sub_offset.z < -POS_RANGE_Z {
            let sub_force = self.steer_towards(sub_offset);
            acceleration += sub_force;
        }

        self.velocity += acceleration * delta;
        let target_speed = self.velocity.magnitude().min(MAX_SPEED).max(MIN_SPEED);
        self.velocity = self.velocity.normalize_to(target_speed);

        self.position += self.velocity * delta;

        self.inst = pos_vel_to_inst(self.position, self.velocity);
    }

    fn steer_towards(&self, target: cgmath::Vector3<f32>) -> cgmath::Vector3<f32> {
        let v = target.normalize() * MAX_SPEED - self.velocity;
        if v.magnitude() > MAX_STEER_FORCE {
            v.normalize_to(MAX_STEER_FORCE)
        } else {
            v
        }
    }
}

fn pos_vel_to_inst(pos: cgmath::Vector3<f32>, vel: cgmath::Vector3<f32>) -> draw::Instance {
    let xy_rot_quat = cgmath::Quaternion::from_arc(
        cgmath::Vector3::unit_x(),
        cgmath::Vector3::new(vel.x, vel.y, 0.0),
        None,
    );
    let z_rot_quat = cgmath::Quaternion::from_arc(
        cgmath::Vector3::new(vel.x, vel.y, 0.0),
        cgmath::Vector3::new(vel.x, vel.y, vel.z),
        None,
    );
    let mat = cgmath::Matrix4::from_translation(pos) * cgmath::Matrix4::from(z_rot_quat) * cgmath::Matrix4::from(xy_rot_quat);
    draw::Instance::new(mat)
}

fn random_pos(rng: &mut ThreadRng) -> cgmath::Vector3<f32> {
    cgmath::Vector3::new(
        rng.gen_range(-POS_RANGE..POS_RANGE),
        rng.gen_range(-POS_RANGE..POS_RANGE),
        rng.gen_range(-POS_RANGE..POS_RANGE_Z),
    )
}


struct PerSpecies {
    diffuse_bind_group: wgpu::BindGroup,
    // diffuse_texture: texture::Texture,

    verts_buffer: wgpu::Buffer,
    // ind_buffer: wgpu::Buffer,
    inst_buffer: wgpu::Buffer,

    // num_inds: u32,
    num_verts: usize,
}

pub struct BoidManager {
    boids: Vec<Boid>,
    per_species: Vec<PerSpecies>,
    rng: ThreadRng,
}
impl BoidManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut boids = Vec::with_capacity(NUM_BOIDS * SPECIES_COUNT);
        let mut per_species = Vec::new();

        let diffuse_bytes_red = include_bytes!("red.jpg");
        let diffuse_bytes_green = include_bytes!("green.png");

        for species in &ALL_SPECIES {
            let mut insts = Vec::with_capacity(NUM_BOIDS);
            for _ in 0..NUM_BOIDS {
                let position = random_pos(&mut rng);
                let velocity = cgmath::Vector3::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                ).normalize_to(rng.gen_range(MIN_SPEED..MAX_SPEED));

                let boid = Boid::new(position, velocity, *species);

                insts.push(boid.inst);
                boids.push(boid);
            }

            let diffuse_texture = match species {
                Species::Red   => texture::Texture::from_bytes(device, queue, diffuse_bytes_red,   SPECIES_TEXTURE_PATHS[*species as usize]).unwrap(),
                Species::Green => texture::Texture::from_bytes(device, queue, diffuse_bytes_green, SPECIES_TEXTURE_PATHS[*species as usize]).unwrap(),
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
            // let mut inds = Vec::new();

            let mut highest_v: f32 = 0.0;

            let obj = match species {
                Species::Red   => boid_obj::RED_OBJ,
                Species::Green => boid_obj::GREEN_OBJ,
            };

            for line in obj.lines() {
                let mut split = line.split_whitespace();
                let first = split.next();
                match first {
                    Some("v") => {
                        let x: f32 = split.next().unwrap().parse().unwrap();
                        let y: f32 = split.next().unwrap().parse().unwrap();
                        let z: f32 = split.next().unwrap().parse().unwrap();

                        let z_sign = if *species == Species::Green { -1.0 } else { 1.0 };
                        vert_poses.push([z, x, z_sign * y]);

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
                // diffuse_texture,

                verts_buffer,
                inst_buffer,

                num_verts: verts.len(),
            });
        }

        Self { boids, per_species, rng }
    }

    pub fn update(&mut self, queue: &wgpu::Queue, sub: &sub::Sub, delta: f32) {
        for boid in self.boids.iter_mut() {
            boid.num_flockmates = 0;
            boid.sum_flock_heading = cgmath::Vector3::zero();
            boid.sum_flock_center = cgmath::Vector3::zero();
            boid.sum_flock_separation = cgmath::Vector3::zero();
        }

        for i in 0..self.boids.len() {
            for j in 0..self.boids.len() {
                if i == j {
                    continue;
                }

                let offset = self.boids[j].position - self.boids[i].position;
                let distance = offset.magnitude();

                let i_species = self.boids[i].species;
                let j_species = self.boids[j].species;

                if distance < PERCEPTION_RADIUS  {
                    if i_species == j_species {
                        self.boids[i].num_flockmates += 1;
                        
                        let boid_j_vel = self.boids[j].velocity;
                        self.boids[i].sum_flock_heading += boid_j_vel;
    
                        let boid_j_pos = self.boids[j].position;
                        self.boids[i].sum_flock_center += boid_j_pos;
                    }
    
                    if distance < AVOIDANCE_RADIUS {
                        self.boids[i].sum_flock_separation -= offset / distance.pow(2);
                    }
                }
            }

            let offset = sub.pos().to_vec() - self.boids[i].position;
            let distance = offset.magnitude();

            if distance < PERCEPTION_RADIUS {
                self.boids[i].sum_flock_separation -= offset / distance.pow(2);
            }
        }

        let mut insts = [ Vec::with_capacity(NUM_BOIDS), Vec::with_capacity(NUM_BOIDS) ];
        for boid in self.boids.iter_mut() {
            boid.update(sub, delta);
            insts[boid.species as usize].push(boid.inst);
        }

        for (i, species) in ALL_SPECIES.iter().enumerate() {
            queue.write_buffer(&self.per_species[*species as usize].inst_buffer, 0, bytemuck::cast_slice(&insts[i]));
        }
    }

    pub fn verts_buffer_slice(&self, species: Species) -> wgpu::BufferSlice { self.per_species[species as usize].verts_buffer.slice(..) }
    pub fn inst_buffer_slice(&self, species: Species) -> wgpu::BufferSlice { self.per_species[species as usize].inst_buffer.slice(..) }
    pub fn diffuse_bind_group(&self, species: Species) -> &wgpu::BindGroup { &self.per_species[species as usize].diffuse_bind_group }
    pub fn num_verts(&self, species: Species) -> usize { self.per_species[species as usize].num_verts }
    pub fn num_inst(&self, _species: Species) -> usize { NUM_BOIDS }
}
