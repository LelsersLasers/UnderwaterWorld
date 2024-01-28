use crate::{chunk, sub, util};
use cgmath::InnerSpace;
use std::collections::HashMap;

const RECHECK_NEARBY_DIST: f32 = 4.0;

pub const VIEW_DIST: i32 = 4;
const GENERATION_DIST: i32 = 5;
const KEEP_DIST: i32 = 6;
const MAX_Z: i32 = 2;
const MIN_Z: i32 = -2;

const STOP_FULL_BUILD: i32 = GENERATION_DIST * GENERATION_DIST * GENERATION_DIST;

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


pub struct World {
    chunks: HashMap<(i32, i32, i32), chunk::Chunk>,
    chunks_to_render: Vec<(i32, i32, i32)>,
    chunks_to_generate: Vec<((i32, i32, i32), f32)>,
    generating_chunk: Option<GeneratingChunk>,
    last_sub_pos: cgmath::Vector3<f32>,
    remove_state: RemoveState,
    should_full_build: bool,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            chunks_to_render: Vec::new(),
            chunks_to_generate: Vec::new(),
            generating_chunk: None,
            last_sub_pos: cgmath::Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            remove_state: RemoveState::new(),
            should_full_build: true,
        }
    }

    pub fn get_chunk(&self, pos: (i32, i32, i32)) -> Option<&chunk::Chunk> {
        self.chunks.get(&pos)
    }

    pub fn update(&mut self, sub: &sub::Sub, perlin: &noise::Perlin, device: &wgpu::Device) {
        self.remove_far_way(sub);

        let dist = (sub.pos() - self.last_sub_pos).magnitude();
        if dist > RECHECK_NEARBY_DIST {
            self.update_nearby(sub);
            self.last_sub_pos = sub.pos();
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


    pub fn update_nearby(&mut self, sub: &sub::Sub) {
        self.chunks_to_render.clear();
        self.chunks_to_generate.clear();

        let sub_pos = sub.pos();
        let sub_chunk = sub.chunk();

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

                    let chunk_pos = (chunk_x, chunk_y, chunk_z);

                    match self.get_chunk(chunk_pos) {
                        Some(chunk) => {
                            if dist < max_view_dist && chunk.not_blank() {
                                self.chunks_to_render.push(chunk_pos);
                            }
                        }
                        None => {
                            let sort = dist * dist + chunk_z as f32 * chunk::CHUNK_SIZE as f32;
                            self.chunks_to_generate.push((chunk_pos, sort));
                        }
                    }
                }
            }
        }

        self.chunks_to_generate.sort_unstable_by(|(_pos1, dist1), (_pos2, dist2)| {
            dist2.partial_cmp(dist1).unwrap()
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