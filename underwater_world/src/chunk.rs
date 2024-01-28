use crate::{marching_table, draw, perlin_util, util};
use std::collections::HashMap;
use wgpu::util::DeviceExt;

pub const CHUNK_SIZE: usize = 16;
pub const PERLIN_OCTAVES: u32 = 3;
pub const ISO_LEVEL: f32 = -0.1;
pub const MAX_HEIGHT: f32 = (CHUNK_SIZE * 2) as f32;

const X_GENERATION_STEP_ISO: i32 = 9;
const X_GENERATION_STEP_MESH: i32 = 3;

const ISO_LEN: usize = CHUNK_SIZE + 1;

enum BuildState {
    Done,
    Iso,
    Mesh,
}
impl BuildState {
    fn new() -> Self { Self::Iso }
}

struct Build {
    chunk_offset: [i32; 3],
    verts: Vec<draw::VertColor>,
    num_verts: usize,
    tris: HashMap<(usize, usize, usize), Vec<util::Tri>>,
    isos: Vec<f32>,
    x: i32,
}
impl Build {
    fn new(chunk_offset: [i32; 3]) -> Self {
        Self {
            chunk_offset,
            verts: Vec::new(),
            num_verts: 0,
            tris: HashMap::new(),
            isos: Vec::with_capacity(ISO_LEN * ISO_LEN * ISO_LEN),
            x: 0,
        }
    }
    
    fn start_mesh(&mut self) {
        self.x = 0;
    }

    fn finish(&mut self) {
        self.isos.clear();
        self.isos.shrink_to(0);

        self.verts.clear();
        self.verts.shrink_to(0);

        self.x = -1;
    }
}

pub struct Chunk {
	// num_verts: usize,
    // tris: HashMap<(usize, usize, usize), Vec<util::Tri>>,
	verts_buffer: Option<wgpu::Buffer>,
    build: Build,
    build_state: BuildState,
}

impl Chunk {
	pub fn new(pos: (i32, i32, i32)) -> Self {
        let chunk_offset = [
			pos.0 * CHUNK_SIZE as i32,
			pos.1 * CHUNK_SIZE as i32,
			pos.2 * CHUNK_SIZE as i32,
		];

        Self {
            verts_buffer: None,
            build: Build::new(chunk_offset),
            build_state: BuildState::new(),
        }
    }

    fn build_iso(&mut self, perlin: &noise::Perlin) -> bool {
        for _ in 0..X_GENERATION_STEP_ISO {
            for y in 0..(CHUNK_SIZE + 1) {
                for z in 0..(CHUNK_SIZE + 1) {
                    let iso = perlin_util::iso_at(
                        perlin,
                        (self.build.x + self.build.chunk_offset[0]) as f64 / CHUNK_SIZE as f64,
                        (y as i32     + self.build.chunk_offset[1]) as f64 / CHUNK_SIZE as f64,
                        (z as i32     + self.build.chunk_offset[2]) as f64 / CHUNK_SIZE as f64,
                    );
                    self.build.isos.push(iso);
                }
            }
    
            self.build.x += 1;

            if self.build.x == ISO_LEN as i32 { break; }
        }

        self.build.x == ISO_LEN as i32
    }

    fn early_blank_check(&self) -> bool {
        self.build.isos.iter().all(|iso| *iso > ISO_LEVEL)
    }

    fn build_mesh(&mut self) -> bool {
        for _ in 0..X_GENERATION_STEP_MESH {
            let x = self.build.x as usize;
            let chunk_offset = self.build.chunk_offset;

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
                        let iso = self.build.isos[iso_idx];
                        if iso < ISO_LEVEL {
                            triangulation_idx |= 1 << i;
                        }
                    }

                    let indices = marching_table::TRIANGULATION[triangulation_idx];


                    let mut pos_tris = Vec::with_capacity(indices.len() / 3);
                    let mut current_tri = Vec::with_capacity(3);


                    for (i, tri_index) in indices.iter().enumerate() {
                        if *tri_index == -1 {
                            let key = (x, y, z);
                            self.build.tris.insert(key, pos_tris);
                            break;
                        }

                        let edge_corners_indices = marching_table::EDGE_VERTEX_INDICES[*tri_index as usize];
                        let corner_a_idx = edge_corners_indices[0];
                        let corner_b_idx = edge_corners_indices[1];

                        let corner_a = cube_corners[corner_a_idx];
                        let corner_b = cube_corners[corner_b_idx];

                        let iso_idx_a = corner_to_iso_idx(corner_a);
                        let iso_idx_b = corner_to_iso_idx(corner_b);

                        let iso_a = self.build.isos[iso_idx_a];
                        let iso_b = self.build.isos[iso_idx_b];

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
                        self.build.verts.push(vert);


                        current_tri.push(vert.pos);
                        if i % 3 == 2 {
                            let tri = util::Tri::new([current_tri[0], current_tri[1], current_tri[2]]);
                            pos_tris.push(tri);
                            current_tri.clear();
                        }
                    }
                }
            }


            self.build.x += 1;

            if self.build.x == CHUNK_SIZE as i32 { break; }
        }

        self.build.num_verts = self.build.verts.len();

        self.build.x == CHUNK_SIZE as i32
    }

    pub fn build_full(&mut self, perlin: &noise::Perlin, device: &wgpu::Device) {
        while !self.build_partial(perlin, device) {}
    }

    pub fn build_partial(&mut self, perlin: &noise::Perlin, device: &wgpu::Device) -> bool {
        match self.build_state {
            BuildState::Done => true,
            BuildState::Iso => {
                let finished = self.build_iso(perlin);
                if finished {
                    let blank_check = self.early_blank_check();
                    if blank_check {
                        self.build_state = BuildState::Done;
                        self.build.finish();
                    } else {
                        self.build_state = BuildState::Mesh;
                        self.build.start_mesh();
                    }
                }
                false
            },
            BuildState::Mesh => {
                let finished = self.build_mesh();
                if finished {
                    if self.build.num_verts > 0 {
                        let verts_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some(&format!("{:?} Chunk Vertex Buffer", self.build.chunk_offset)),
                            contents: bytemuck::cast_slice(&self.build.verts),
                            usage: wgpu::BufferUsages::VERTEX,
                        });
                        self.verts_buffer = Some(verts_buffer);
                    }

                    self.build_state = BuildState::Done;
                    self.build.finish();
                }
                finished
            },
        }
    }

    pub fn tris_at(&self, pos: (usize, usize, usize)) -> &[util::Tri] {
        match self.build.tris.get(&pos) {
            Some(tris) => tris,
            None => &[],
        }
    }

    pub fn not_blank(&self) -> bool { self.verts_buffer.is_some() }
    // only call if self is not blank
    pub fn verts_buffer_slice(&self) -> wgpu::BufferSlice { self.verts_buffer.as_ref().unwrap().slice(..) }
	pub fn num_verts(&self) -> usize { self.build.num_verts }
}

fn corner_to_iso_idx(corner: [usize; 3]) -> usize {
    corner[0] * (CHUNK_SIZE + 1) * (CHUNK_SIZE + 1) + corner[1] * (CHUNK_SIZE + 1) + corner[2]
}