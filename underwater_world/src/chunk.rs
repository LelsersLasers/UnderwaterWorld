use crate::{marching_table, draw, perlin_util};
use wgpu::util::DeviceExt;

pub const CHUNK_SIZE: usize = 16;
const PERLIN_OCTAVES: u32 = 3;
const ISO_LEVEL: f32 = -0.1;
const MAX_HEIGHT: f32 = (CHUNK_SIZE * 3) as f32;


// space of 16x16x16
pub struct Chunk {
	num_verts: usize,
	verts_buffer: Option<wgpu::Buffer>,
}

impl Chunk {
	pub fn new(
        pos: (i32, i32, i32),
        perlin: &noise::Perlin,
		device: &wgpu::Device,
    ) -> Self {
        let chunk_offset = [
			pos.0 * CHUNK_SIZE as i32,
			pos.1 * CHUNK_SIZE as i32,
			pos.2 * CHUNK_SIZE as i32,
		];

		// flat vec
		let isos = (0..CHUNK_SIZE + 1).flat_map(|x| {
			(0..CHUNK_SIZE + 1).flat_map(move |y| {
				(0..CHUNK_SIZE + 1).map(move |z| {
                    let h = z as i32 + chunk_offset[2];
					let corner = [
						(x as i32 + chunk_offset[0]) as f64 / CHUNK_SIZE as f64,
						(y as i32 + chunk_offset[1]) as f64 / CHUNK_SIZE as f64,
						h as f64 / CHUNK_SIZE as f64,
					];
                    let p = perlin_util::perlin_3d_octaves(perlin, corner, PERLIN_OCTAVES) as f32;
                    p + (h as f32 / MAX_HEIGHT)
				})
			})
		}).collect::<Vec<f32>>();
		let corner_to_iso_idx = |corner: [usize; 3]| {
			corner[0] * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1) + corner[1] * (CHUNK_SIZE + 1) + corner[2]
		};

        let mut verts = Vec::new();

		for x in 0..CHUNK_SIZE {
			for y in 0..CHUNK_SIZE {
				for z in 0..CHUNK_SIZE {

					let cube_corners = [
						[x, y, z],
						[x + 1, y, z],
						[x + 1, y, z + 1],
						[x, y, z + 1],
						[x, y + 1, z],
						[x + 1, y + 1, z],
						[x + 1, y + 1, z + 1],
						[x, y + 1, z + 1],
					];

					let mut triangulation_idx = 0;
					for (i, cube_corner) in cube_corners.iter().enumerate() {
						let iso_idx = corner_to_iso_idx(*cube_corner);
						let iso = isos[iso_idx];
						if iso < ISO_LEVEL {
							triangulation_idx |= 1 << i;
						}
					}

					let indices = marching_table::TRIANGULATION[triangulation_idx];

					for tri_index in indices.iter() {
						if *tri_index == -1 {
							break;
						}

						let edge_corners_indices = marching_table::EDGE_VERTEX_INDICES[*tri_index as usize];
						let corner_a_idx = edge_corners_indices[0];
						let corner_b_idx = edge_corners_indices[1];

						let corner_a = cube_corners[corner_a_idx];
						let corner_b = cube_corners[corner_b_idx];

						let iso_idx_a = corner_to_iso_idx(corner_a);
						let iso_idx_b = corner_to_iso_idx(corner_b);

						let iso_a = isos[iso_idx_a];
						let iso_b = isos[iso_idx_b];

						// interpolate using dist from iso
						let t = (ISO_LEVEL - iso_a) / (iso_b - iso_a);
						let corner_diff = [
							corner_b[0] as f32 - corner_a[0] as f32,
							corner_b[1] as f32 - corner_a[1] as f32,
							corner_b[2] as f32 - corner_a[2] as f32,
						];
						let middle = [
							corner_a[0] as f32 + t * corner_diff[0],
							corner_a[1] as f32 + t * corner_diff[1],
							corner_a[2] as f32 + t * corner_diff[2],
						];

						let color_intensity = *tri_index as f32 / 16.0;

						let vert = draw::VertColor::new(
							[
								middle[0] + chunk_offset[0] as f32,
								middle[1] + chunk_offset[1] as f32,
								middle[2] + chunk_offset[2] as f32,
							],
							[0.7, color_intensity, color_intensity],
						);
						verts.push(vert);
					}
				}
			}
		}

        let num_verts = verts.len();

        if num_verts > 0 {
            let verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", pos)),
                contents: bytemuck::cast_slice(&verts),
                usage: wgpu::BufferUsages::VERTEX,
            });
            Self {
                num_verts,
                verts_buffer: Some(verts_buffer),
            }
        } else {
            Self {
                num_verts,
                verts_buffer: None,
            }
        }
	}

    pub fn not_blank(&self) -> bool { self.verts_buffer.is_some() }
    // only call if self is not blank
    pub fn vert_buffer_slice(&self) -> wgpu::BufferSlice { self.verts_buffer.as_ref().unwrap().slice(..) }
	pub fn num_verts(&self) -> usize { self.num_verts }
}

// fn smoothstep(x: f32, edge0: f32, edge1: f32) -> f32 {
//     let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
//     t * t * (3.0 - 2.0 * t)
// }