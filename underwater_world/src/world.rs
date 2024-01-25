use crate::{chunk, sub};
use std::collections::HashMap;

pub const VIEW_DIST: i32 = 4;
pub const MAX_Z: i32 = 2;
pub const MIN_Z: i32 = -2;


pub struct World {
    chunks: HashMap<(i32, i32, i32), chunk::Chunk>,
    chunks_to_render: Vec<(i32, i32, i32)>,
    chunks_to_generate: Vec<(i32, i32, i32)>,
    last_sub_chunk: (i32, i32, i32),
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            chunks_to_render: Vec::new(),
            chunks_to_generate: Vec::new(),
            last_sub_chunk: (i32::MAX, i32::MAX, i32::MAX),
        }
    }

    pub fn get_chunk(&self, pos: (i32, i32, i32)) -> Option<&chunk::Chunk> {
        self.chunks.get(&pos)
    }

    pub fn update(&mut self, sub: &sub::Sub, perlin: &noise::Perlin, device: &wgpu::Device) {
        let sub_chunk = sub.chunk();
        if sub_chunk != self.last_sub_chunk {
            self.update_nearby(sub_chunk);
        }
        // self.update_nearby(sub_chunk);

        // println!("chunks_to_generate: {}", self.chunks_to_generate.len());

        if let Some(pos) = self.chunks_to_generate.pop() {
            let chunk = chunk::Chunk::new(pos, perlin, device);
            if chunk.not_blank() {
                self.chunks_to_render.push(pos);
            }
            self.chunks.insert(pos, chunk);
        }
    }


    pub fn update_nearby(&mut self, sub_chunk: (i32, i32, i32)) {
        self.last_sub_chunk = sub_chunk;

        self.chunks_to_render.clear();
        self.chunks_to_generate.clear();

        for x in -VIEW_DIST..VIEW_DIST {
            for y in -VIEW_DIST..VIEW_DIST {
                for z in -VIEW_DIST..VIEW_DIST {
                    let chunk_z = sub_chunk.2 + z;
                    if !(MIN_Z..=MAX_Z).contains(&chunk_z) { continue; }

                    let dist = ((x.pow(2) + y.pow(2) + z.pow(2)) as f32).sqrt();
                    if dist > VIEW_DIST as f32 { continue; }

                    let chunk_pos = (
                        sub_chunk.0 + x,
                        sub_chunk.1 + y,
                        chunk_z,
                    );

                    match self.get_chunk(chunk_pos) {
                        Some(chunk) => {
                            if chunk.not_blank() {
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

    pub fn chunks_to_render(&self) -> &[(i32, i32, i32)] {
        &self.chunks_to_render
    }

    pub fn generate_count(&self) -> usize {
        self.chunks_to_generate.len()
    }
}