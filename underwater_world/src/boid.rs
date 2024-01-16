use crate::{chunk, texture, world};
use cgmath::{InnerSpace, Zero};
use rand::prelude::*;

const MIN_SPEED: f32 = 2.0;
const MAX_SPEED: f32 = 5.0;

const PERCEPTION_RADIUS: f32 = 2.5;
const AVOIDANCE_RADIUS: f32 = 1.0;

const MAX_STEER_FORCE: f32 = 3.0;

const NUM_BOIDS: usize = 50;

#[derive(Copy, Clone)]
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
        }
    }

    fn update(&mut self, delta: f32) {
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

        self.velocity += acceleration * delta;
        let target_speed = self.velocity.magnitude().min(MAX_SPEED).max(MIN_SPEED);
        self.velocity = self.velocity.normalize_to(target_speed);

        self.position += self.velocity * delta;
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


struct TextureInfo {
    diffuse_bind_group: wgpu::BindGroup,
    diffuse_texture: texture::Texture,
}

pub struct BoidManager {
    boids: Vec<Boid>,
    texture_infos: Vec<TextureInfo>,
}
impl BoidManager {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_bind_group_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut boids = Vec::new();
        let mut texture_infos = Vec::new();

        let diffuse_bytes_red = include_bytes!("red.jpg");
        let diffuse_bytes_green = include_bytes!("green.png");

        for species in &ALL_SPECIES {
            for _ in 0..NUM_BOIDS {
                let position_range = chunk::CHUNK_SIZE as f32 * world::VIEW_DIST as f32;
    
                let position = cgmath::Vector3::new(
                    rng.gen_range(-position_range..position_range),
                    rng.gen_range(-position_range..position_range),
                    rng.gen_range(-position_range..position_range),
                );
    
                let velocity = cgmath::Vector3::new(
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                    rng.gen_range(-1.0..1.0),
                ).normalize_to(rng.gen_range(MIN_SPEED..MAX_SPEED));
                boids.push(Boid::new(position, velocity, *species));
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

            texture_infos.push(TextureInfo {
                diffuse_bind_group,
                diffuse_texture,
            });
        }

        Self { boids, texture_infos }
    }

    pub fn update(&mut self, delta: f32) {
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

                if distance < PERCEPTION_RADIUS {
                    self.boids[i].num_flockmates += 1;
                    
                    let boid_j_vel = self.boids[j].velocity;
                    self.boids[i].sum_flock_heading += boid_j_vel;

                    let boid_j_pos = self.boids[j].position;
                    self.boids[i].sum_flock_center += boid_j_pos;

                    if distance < AVOIDANCE_RADIUS {
                        self.boids[i].sum_flock_separation -= offset;
                    }
                }
            }
        }

        for boid in self.boids.iter_mut() {
            boid.update(delta);
        }
    }
}
