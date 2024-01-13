#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vert {
	pub pos: [f32; 3],
	color: [f32; 3],
}
impl Vert {
	pub fn new(pos: [f32; 3], color: [f32; 3]) -> Self {
		Self { pos, color }
	}

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

pub mod sub {
	use cgmath::SquareMatrix;


	#[repr(C)]
	#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
	pub struct Instance {
		model: [[f32; 4]; 4],
	}
	impl Instance {
		pub fn identity() -> Self {
			Self {
				model: cgmath::Matrix4::identity().into(),
			}
		}
		pub fn new(model: cgmath::Matrix4<f32>) -> Self {
			Self {
				model: model.into(),
			}
		}
		pub fn desc() -> wgpu::VertexBufferLayout<'static> {
			wgpu::VertexBufferLayout {
				array_stride: std::mem::size_of::<Instance>() as wgpu::BufferAddress,
				step_mode: wgpu::VertexStepMode::Instance,
				attributes: &[
					wgpu::VertexAttribute {
						offset: 0,
						shader_location: 5,
						format: wgpu::VertexFormat::Float32x4,
					},
					wgpu::VertexAttribute {
						offset: std::mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
						shader_location: 6,
						format: wgpu::VertexFormat::Float32x4,
					},
					wgpu::VertexAttribute {
						offset: std::mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
						shader_location: 7,
						format: wgpu::VertexFormat::Float32x4,
					},
					wgpu::VertexAttribute {
						offset: std::mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
						shader_location: 8,
						format: wgpu::VertexFormat::Float32x4,
					},
				],
			}
		}
	}
}





// //----------------------------------------------------------------------------//
// #[repr(C)]
// #[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct Vertex {
//     pub position: [f32; 3],
//     pub color: [f32; 3],
// }

// impl Vertex {
//     pub fn desc() -> wgpu::VertexBufferLayout<'static> {
//         wgpu::VertexBufferLayout {
//             array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
//             step_mode: wgpu::VertexStepMode::Vertex,
//             attributes: &[
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     shader_location: 0,
//                     format: wgpu::VertexFormat::Float32x3,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
//                     shader_location: 1,
//                     format: wgpu::VertexFormat::Float32x3,
//                 }
//             ]
//         }
//     }
// }
// //----------------------------------------------------------------------------//


// //----------------------------------------------------------------------------//
// pub struct Instance {
//     pub position: cgmath::Vector3<f32>,
//     pub rotation: cgmath::Quaternion<f32>,
// }
// impl Instance {
//     pub fn to_raw(&self) -> InstanceRaw {
//         InstanceRaw {
//             model: (cgmath::Matrix4::from_translation(self.position) * cgmath::Matrix4::from(self.rotation)).into(),
//         }
//     }
// }

// #[repr(C)]
// #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
// pub struct InstanceRaw {
//     model: [[f32; 4]; 4],
// }
// impl InstanceRaw {
//     pub fn desc() -> wgpu::VertexBufferLayout<'static> {
//         use std::mem;
//         wgpu::VertexBufferLayout {
//             array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
//             // We need to switch from using a step mode of Vertex to Instance
//             // This means that our shaders will only change to use the next
//             // instance when the shader starts processing a new instance
//             step_mode: wgpu::VertexStepMode::Instance,
//             attributes: &[
//                 // A mat4 takes up 4 vertex slots as it is technically 4 vec4s. We need to define a slot
//                 // for each vec4. We'll have to reassemble the mat4 in the shader.
//                 wgpu::VertexAttribute {
//                     offset: 0,
//                     // While our vertex shader only uses locations 0, and 1 now, in later tutorials, we'll
//                     // be using 2, 3, and 4, for Vertex. We'll start at slot 5, not conflict with them later
//                     shader_location: 5,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 4]>() as wgpu::BufferAddress,
//                     shader_location: 6,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 8]>() as wgpu::BufferAddress,
//                     shader_location: 7,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//                 wgpu::VertexAttribute {
//                     offset: mem::size_of::<[f32; 12]>() as wgpu::BufferAddress,
//                     shader_location: 8,
//                     format: wgpu::VertexFormat::Float32x4,
//                 },
//             ],
//         }
//     }
// }
// //----------------------------------------------------------------------------//


// //----------------------------------------------------------------------------//
// pub const VERTICES: &[Vertex] = &[
//     Vertex { position: [-0.0868241, 0.49240386, 0.0], color: [0.9, 0.1, 0.1], }, // A
//     Vertex { position: [-0.49513406, 0.06958647, 0.0], color: [0.9, 0.9, 0.1], }, // B
//     Vertex { position: [-0.21918549, -0.44939706, 0.0], color: [0.9, 0.1, 0.9], }, // C
//     Vertex { position: [0.35966998, -0.3473291, 0.0], color: [0.1, 0.1, 0.9], }, // D
//     Vertex { position: [0.44147372, 0.2347359, 0.0], color: [0.1, 0.9, 0.1], }, // E
// ];

// pub const INDICES: &[u16] = &[
//     0, 1, 4,
//     1, 2, 4,
//     2, 3, 4,
// ];

// pub const NUM_INSTANCES_PER_ROW: u32 = 10;
// pub const INSTANCE_DISPLACEMENT: cgmath::Vector3<f32> = cgmath::Vector3::new(NUM_INSTANCES_PER_ROW as f32 * 0.5, 0.0, NUM_INSTANCES_PER_ROW as f32 * 0.5);
