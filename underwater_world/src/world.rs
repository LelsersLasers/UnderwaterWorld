use crate::{camera, chunk, sub, util};
use cgmath::InnerSpace;
use std::collections::HashMap;

const RECHECK_NEARBY_DIST: f32 = 4.0;
const RECHECK_NEARBY_ANGLE: f32 = 0.33;

pub const VIEW_DIST: i32 = 4;
const GENERATION_DIST: i32 = 5;
const KEEP_DIST: i32 = 6;
pub const MAX_Z: i32 = 2;
pub const MIN_Z: i32 = -2;

const STOP_FULL_BUILD: i32 = GENERATION_DIST * GENERATION_DIST * GENERATION_DIST;

const VIEW_FRUST_FOVY: f32 = 55.0;
const GENERATE_FRUST_FOVY: f32 = 90.0;

struct GeneratingChunk {
    chunk_pos: (i32, i32, i32),
    chunk: chunk::Chunk,
}

struct RemoveState {
    keys_left: Vec<(i32, i32, i32)>
}
impl RemoveState {
    fn new() -> Self {
        Self { keys_left: Vec::new() }
    }
}

struct GenPrio {
    dist: f32,
    z: f32,
    in_view: bool,
    in_gen: bool,
}
impl GenPrio {
    fn compare(&self, other: &GenPrio) -> std::cmp::Ordering {
        // in view -> in gen -> dist + z

        if self.in_view && !other.in_view {
            return std::cmp::Ordering::Less;
        } else if !self.in_view && other.in_view {
            return std::cmp::Ordering::Greater;
        }

        if self.in_gen && !other.in_gen {
            return std::cmp::Ordering::Less;
        } else if !self.in_gen && other.in_gen {
            return std::cmp::Ordering::Greater;
        }

        let self_sort = self.dist * self.dist + self.z;
        let other_sort = other.dist * other.dist + other.z;

        self_sort.partial_cmp(&other_sort).unwrap()
    }
}


pub struct World {
    chunks: HashMap<(i32, i32, i32), chunk::Chunk>,
    chunks_to_render: Vec<(i32, i32, i32)>,
    chunks_to_generate: Vec<((i32, i32, i32), GenPrio)>,
    generating_chunk: Option<GeneratingChunk>,
    remove_state: RemoveState,
    should_full_build: bool,

    last_sub_pos: cgmath::Vector3<f32>,
    last_sub_bearing: cgmath::Vector3<f32>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            chunks_to_render: Vec::new(),
            chunks_to_generate: Vec::new(),
            generating_chunk: None,
            remove_state: RemoveState::new(),
            should_full_build: true,
            last_sub_pos: cgmath::Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            last_sub_bearing: cgmath::Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        }
    }

    pub fn get_chunk(&self, pos: (i32, i32, i32)) -> Option<&chunk::Chunk> {
        self.chunks.get(&pos)
    }

    pub fn update(&mut self, sub: &sub::Sub, camera: &camera::Camera, sub_reset: bool, perlin: &noise::Perlin, device: &wgpu::Device) {
        self.remove_far_way(sub);

        let dist = (sub.pos() - self.last_sub_pos).magnitude();
        let angle = sub.bearing().angle(self.last_sub_bearing);

        if sub_reset || dist > RECHECK_NEARBY_DIST || angle > cgmath::Rad(RECHECK_NEARBY_ANGLE) {
            self.update_nearby(sub, camera);
            self.last_sub_pos = sub.pos();
            self.last_sub_bearing = sub.bearing();
        }

        if self.should_full_build {
            self.build_full_step(perlin, device);
            self.should_full_build = !(self.chunks_to_generate.is_empty() || self.chunks.len() >= STOP_FULL_BUILD as usize);
        } else {
            self.build_step(sub, perlin, device);
        }
    }

    fn build_full_step(&mut self, perlin: &noise::Perlin, device: &wgpu::Device) {
        if let Some((pos, _dist)) = self.chunks_to_generate.pop() {
            let mut chunk = chunk::Chunk::new(pos);
            chunk.build_full(perlin, device);
            if chunk.not_blank() {
                self.chunks_to_render.push(pos);
            }

            self.chunks.insert(pos, chunk);
        }
    }

    fn build_step(&mut self, sub: &sub::Sub, perlin: &noise::Perlin, device: &wgpu::Device) {
        if let Some(generating_chunk) = &mut self.generating_chunk {
            let finished = generating_chunk.chunk.build_partial(perlin, device);
            if finished {
                let pos = generating_chunk.chunk_pos;
                let sub_chunk = sub.chunk();
                let dist_sq = util::dist_sq(pos, sub_chunk);
                if generating_chunk.chunk.not_blank() && dist_sq <= (VIEW_DIST + 1) * (VIEW_DIST + 1) {
                    self.chunks_to_render.push(pos);
                }
                self.chunks.insert(pos, self.generating_chunk.take().unwrap().chunk);
                self.generating_chunk = None;
            }
        } else if let Some((pos, _dist)) = self.chunks_to_generate.pop() {
            let chunk = chunk::Chunk::new(pos);
            self.generating_chunk = Some(GeneratingChunk { chunk_pos: pos, chunk });
        }
    }


    pub fn update_nearby(&mut self, sub: &sub::Sub, camera: &camera::Camera) {
        self.chunks_to_render.clear();
        self.chunks_to_generate.clear();

        let sub_pos = sub.pos();
        let sub_chunk = sub.chunk();

        let view_view_proj = camera.chunk_generation_frustum_matrix(VIEW_FRUST_FOVY);
        let gen_view_proj = camera.chunk_generation_frustum_matrix(GENERATE_FRUST_FOVY);

        let max_view_dist = VIEW_DIST as f32 * chunk::CHUNK_SIZE as f32;
        let max_generation_dist = GENERATION_DIST as f32 * chunk::CHUNK_SIZE as f32;

        let start_z = (sub_chunk.2 - GENERATION_DIST).max(MIN_Z);
        let end_z =   (sub_chunk.2 + GENERATION_DIST).min(MAX_Z);

        for x in -GENERATION_DIST..GENERATION_DIST {
            let chunk_x = sub_chunk.0 + x;

            for y in -GENERATION_DIST..GENERATION_DIST {
                let chunk_y = sub_chunk.1 + y;

                for chunk_z in start_z..=end_z {
                    let chunk_center = cgmath::Vector3::new(
                        (chunk_x as f32 + 0.5) * chunk::CHUNK_SIZE as f32,
                        (chunk_y as f32 + 0.5) * chunk::CHUNK_SIZE as f32,
                        (chunk_z as f32 + 0.5) * chunk::CHUNK_SIZE as f32,
                    );
                    let dist = (sub_pos - chunk_center).magnitude();
                    if dist > max_generation_dist { continue; }

                    let corners = [
                        (0.0, 0.0, 0.0),
                        (1.0, 0.0, 0.0),
                        (0.0, 1.0, 0.0),
                        (1.0, 1.0, 0.0),
                        (0.0, 0.0, 1.0),
                        (1.0, 0.0, 1.0),
                        (0.0, 1.0, 1.0),
                        (1.0, 1.0, 1.0),
                    ];

                    let mut in_view = false;
                    let mut in_gen = false;

                    for corner in corners {
                        let chunk_corner = cgmath::Vector3::new(
                            (chunk_x as f32 + corner.0) * chunk::CHUNK_SIZE as f32,
                            (chunk_y as f32 + corner.1) * chunk::CHUNK_SIZE as f32,
                            (chunk_z as f32 + corner.2) * chunk::CHUNK_SIZE as f32,
                        );

                        if util::in_frustum(chunk_corner, gen_view_proj) {
                            in_gen = true;
                            if util::in_frustum(chunk_corner, view_view_proj) {
                                in_view = true;
                                break;
                            }
                        }
                    }

                    let chunk_pos = (chunk_x, chunk_y, chunk_z);

                    match self.get_chunk(chunk_pos) {
                        Some(chunk) => {
                            if dist < max_view_dist && chunk.not_blank() && in_view {
                                self.chunks_to_render.push(chunk_pos);
                            }
                        }
                        None => {
                            let gen_prio = GenPrio {
                                dist,
                                z: chunk_z as f32 * chunk::CHUNK_SIZE as f32,
                                in_view,
                                in_gen,
                            };
                            self.chunks_to_generate.push((chunk_pos, gen_prio));
                        }
                    }
                }
            }
        }

        self.chunks_to_generate.sort_unstable_by(|(_pos1, gen_prio1), (_pos2, gen_prio2)| {
            gen_prio2.compare(gen_prio1)
        });
    }

    fn remove_far_way(&mut self, sub: &sub::Sub) {
        if let Some(pos) = self.remove_state.keys_left.pop() {
            let sub_chunk = sub.chunk();
            let dist_sq = util::dist_sq(pos, sub_chunk);

            if dist_sq >= KEEP_DIST * KEEP_DIST {
                self.chunks.remove(&pos);
            }
        } else {
            self.remove_state.keys_left = self.chunks.keys().cloned().collect();
        }
    }

    pub fn chunks_to_render(&self) -> &[(i32, i32, i32)] { &self.chunks_to_render }

    pub fn generate_count(&self) -> usize { self.chunks_to_generate.len() }
    pub fn render_count(&self) -> usize { self.chunks_to_render.len() }
    pub fn total_count(&self) -> usize { self.chunks.len() }
}