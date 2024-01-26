use cgmath::InnerSpace;

use crate::{chunk, sub};
use std::collections::HashMap;

const RECHECK_NEARBY_DIST: f32 = 4.0;
const RECHECK_REMOVE_DIST: i32 = 8; // chunks

pub const VIEW_DIST: i32 = 4;
const GENERATION_DIST: i32 = 5;
const KEEP_DIST: i32 = 8;
const MAX_Z: i32 = 2;
const MIN_Z: i32 = -2;

struct GeneratingChunk {
    chunk_pos: (i32, i32, i32),
    chunk: chunk::Chunk,
}


pub struct World {
    chunks: HashMap<(i32, i32, i32), chunk::Chunk>,
    chunks_to_render: Vec<(i32, i32, i32)>,
    chunks_to_generate: Vec<(i32, i32, i32)>,
    generating_chunk: Option<GeneratingChunk>,
    last_sub_nearby_pos: cgmath::Vector3<f32>,
    last_sub_remove_pos: cgmath::Vector3<f32>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            chunks_to_render: Vec::new(),
            chunks_to_generate: Vec::new(),
            generating_chunk: None,
            last_sub_nearby_pos: cgmath::Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            last_sub_remove_pos: cgmath::Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY),
        }
    }

    pub fn get_chunk(&self, pos: (i32, i32, i32)) -> Option<&chunk::Chunk> {
        self.chunks.get(&pos)
    }

    pub fn update(&mut self, sub: &sub::Sub, perlin: &noise::Perlin, device: &wgpu::Device) {
        let nearby_dist = (sub.pos() - self.last_sub_nearby_pos).magnitude();
        if nearby_dist > RECHECK_NEARBY_DIST {
            self.update_nearby(sub);
            self.last_sub_nearby_pos = sub.pos();
            println!("update_nearby");
        }

        let remove_dist = (sub.pos() - self.last_sub_remove_pos).magnitude();
        if remove_dist > RECHECK_REMOVE_DIST as f32 * chunk::CHUNK_SIZE as f32 {
            println!("Intial {}", self.chunks.len());
            self.remove_far_way(sub);
            self.last_sub_remove_pos = sub.pos();
            println!("After {}", self.chunks.len());
        }

        // println!("chunks_to_generate: {}", self.chunks_to_generate.len());

        if let Some(generating_chunk) = &mut self.generating_chunk {
            let finished = generating_chunk.chunk.build_partial(perlin, device);
            if finished {
                let pos = generating_chunk.chunk_pos;
                if generating_chunk.chunk.not_blank() {
                    self.chunks_to_render.push(pos);
                }
                self.chunks.insert(pos, self.generating_chunk.take().unwrap().chunk);
                self.generating_chunk = None;
            }
        } else if let Some(pos) = self.chunks_to_generate.pop() {
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
                            self.chunks_to_generate.push(chunk_pos);
                        }
                    }
                }
            }
        }

        self.chunks_to_generate.sort_unstable_by_key(|(x, y, z)| {
            let dx = x - sub_chunk.0;
            let dy = y - sub_chunk.1;
            let dz = z - sub_chunk.2;
            -(dx * dx + dy * dy + dz * dz)
        });
    }

    fn remove_far_way(&mut self, sub: &sub::Sub) {
        let sub_chunk = sub.chunk();
        self.chunks.retain(|pos, _| {
            let dx = pos.0 - sub_chunk.0;
            let dy = pos.1 - sub_chunk.1;
            let dz = pos.2 - sub_chunk.2;
            dx * dx + dy * dy + dz * dz <= KEEP_DIST * KEEP_DIST
        });
    }

    pub fn chunks_to_render(&self) -> &[(i32, i32, i32)] {
        &self.chunks_to_render
    }

    pub fn generate_count(&self) -> usize {
        self.chunks_to_generate.len()
    }
}