use noise::NoiseFn;

use crate::chunk;


pub fn perlin_3d_octaves(perlin: &noise::Perlin, point: [f64; 3], octaves: u32) -> f64 {
    let mut total = 0.0;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        total += perlin.get([point[0] * frequency, point[1] * frequency, point[2] * frequency]) * amplitude;

        max_value += amplitude;

        amplitude *= 0.5;
        frequency *= 2.0;
    }

    total / max_value
}

pub fn iso_at(perlin: &noise::Perlin, x: f64, y: f64, z: f64) -> f32 {
    let corner = [x, y, z];
    let p = perlin_3d_octaves(perlin, corner, chunk::PERLIN_OCTAVES) as f32;
    p + (z as f32 * chunk::CHUNK_SIZE as f32 / chunk::MAX_HEIGHT)
}