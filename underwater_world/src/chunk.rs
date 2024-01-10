use crate::marching_table;

const CHUNK_SIZE: usize = 16;
const ISO_LEVEL: f32 = 0.0;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vert {
	pos: [f32; 3],
	color: [f32; 3],
}
impl Vert {
	pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vert>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
				wgpu::VertexAttribute {
					offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
					shader_location: 1,
					format: wgpu::VertexFormat::Float32x3,
				}
            ]
        }
    }
}

// space of 16x16x16
pub struct Chunk {
	pos: [i32; 3],
	verts: Vec<Vert>,
	verts_buffer: Option<wgpu::Buffer>,
}

impl Chunk {
	pub fn new(pos: [i32; 3]) -> Self {
		Self {
			pos,
			verts: Vec::new(),
			verts_buffer: None,
		}
	}

	pub fn create_verts(
		&mut self, perlin: &noise::Perlin,
		device: &wgpu::Device,
		queue: &wgpu::Queue,
	) {
		// marching cubes
		use noise::NoiseFn;

		self.verts.clear();

		let chunk_offset = [
			self.pos[0] * CHUNK_SIZE as i32,
			self.pos[1] * CHUNK_SIZE as i32,
			self.pos[2] * CHUNK_SIZE as i32,
		];

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

					let isos = cube_corners.iter().map(|corner| {
						let corner = [
							corner[0] as f64 + chunk_offset[0] as f64,
							corner[1] as f64 + chunk_offset[1] as f64,
							corner[2] as f64 + chunk_offset[2] as f64,
						];
						perlin.get([corner[0] / 16.0, corner[1] / 16.0, corner[2] / 16.0]) as f32
					}).collect::<Vec<f32>>();

					let mut triangulation_idx = 0;
					for (i, iso) in isos.iter().enumerate() {
						if *iso < ISO_LEVEL {
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

						let iso_a = isos[corner_a_idx];
						let iso_b = isos[corner_b_idx];

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

						let vert = Vert {
							pos: [
								middle[0] + chunk_offset[0] as f32,
								middle[1] + chunk_offset[1] as f32,
								middle[2] + chunk_offset[2] as f32,
							],
							color: [0.7, color_intensity, color_intensity],
						};

						self.verts.push(vert);
					}
				}
			}
		}

		if self.verts_buffer.is_none() {
			use wgpu::util::DeviceExt;

			self.verts_buffer = Some(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex Buffer", self.pos)),
                contents: bytemuck::cast_slice(&self.verts),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            }));
		} else {
			queue.write_buffer(
				self.verts_buffer.as_ref().unwrap(),
				0,
				bytemuck::cast_slice(&self.verts),
			);
		}
	}

	pub fn buffer_slice(&self) -> wgpu::BufferSlice {
		self.verts_buffer.as_ref().unwrap().slice(..)
	}

	pub fn len(&self) -> usize {
		self.verts.len()
	}
}