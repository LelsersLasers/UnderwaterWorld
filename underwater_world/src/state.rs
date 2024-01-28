use crate::{boid, camera, consts, draw, texture, timer, sub, world};
use wgpu::util::DeviceExt;

const TEXT_SIZE: f32 = 20.0 / 600.0;
const TEXT_SPACING: f32 = 10.0 / 600.0;
const FPSES_TO_KEEP: f32 = 2.0; // seconds

pub struct State<'a> {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    brush: wgpu_text::TextBrush<wgpu_text::glyph_brush::ab_glyph::FontRef<'a>>,

    terrain_render_pipeline: wgpu::RenderPipeline,
    sub_render_pipeline: wgpu::RenderPipeline,
    fish_render_pipeline: wgpu::RenderPipeline,

    depth_texture: texture::Texture,

    camera: camera::Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    fps_counter: timer::FpsCounter,
    fpses: Vec<f32>,

    perlin: noise::Perlin,

    sub: sub::Sub,

    world: world::World,

    boid_manager: boid::BoidManager,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: winit::window::Window,
}

impl<'a> State<'a> {
    pub async fn new(window: winit::window::Window) -> Self {
        let size = window.inner_size();

        //--------------------------------------------------------------------//
        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        // # Safety
        // The surface needs to live as long as the window that created it.
        // State owns the window, so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = match instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::None,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
        {
            Some(adapter) => adapter,
            None => {
                cfg_if::cfg_if! {
                    if #[cfg(target_arch = "wasm32")] {
                        panic!("No adapter found")
                    } else {
                        instance
                            .enumerate_adapters(wgpu::Backends::all())
                            .find(|adapter| {
                                // Check if this adapter supports our surface
                                adapter.is_surface_supported(&surface)
                            })
                            .unwrap()
                    }
                }
            }
        };
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    // features: wgpu::Features::POLYGON_MODE_LINE,
                    features: wgpu::Features::empty(),
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let depth_texture = texture::Texture::create_depth_texture(&device, &config, "depth_texture");
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let font = include_bytes!("Assistant-Medium.ttf");
        let brush = wgpu_text::BrushBuilder::using_font_bytes(font)
            .unwrap()
            .build(&device, config.width, config.height, config.format);
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let camera = camera::Camera::new(&config);

        let camera_buffer = device.create_buffer_init(
            &wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[*camera.uniform()]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            }
        );

        let camera_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
            label: Some("camera_bind_group_layout"),
        });

        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                }
            ],
            label: Some("camera_bind_group"),
        });
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            multisampled: false,
                            view_dimension: wgpu::TextureViewDimension::D2,
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                ],
                label: Some("texture_bind_group_layout"),
            });
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let terrain_shader = device.create_shader_module(wgpu::include_wgsl!("terrain.wgsl"));
        let sub_shader = device.create_shader_module(wgpu::include_wgsl!("sub.wgsl"));

        let terrain_sub_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });
        let terrain_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Terrain Render Pipeline"),
            layout: Some(&terrain_sub_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &terrain_shader,
                entry_point: "vs_main",
                buffers: &[draw::VertColor::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &terrain_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        let sub_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Sub Render Pipeline"),
            layout: Some(&terrain_sub_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &sub_shader,
                entry_point: "vs_main",
                buffers: &[draw::VertColor::desc(), draw::Instance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &sub_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                // MODEL triangles are not wound correctly for backface culling
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        let fish_shader = device.create_shader_module(wgpu::include_wgsl!("fish.wgsl"));

        let fish_render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                    &texture_bind_group_layout
                ],
                push_constant_ranges: &[],
            });
        let fish_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fish Render Pipeline"),
            layout: Some(&fish_render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &fish_shader,
                entry_point: "vs_main",
                buffers: &[draw::VertTex::desc(), draw::InstanceTime::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fish_shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                // cull_mode: Some(wgpu::Face::Back),
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState {
                count: 1, 
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let fps_counter = timer::FpsCounter::new();
        let fpses = Vec::new();
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let seed = (instant::now().round() % u32::MAX as f64) as u32;
        println!("Seed: {}", seed);
        let perlin = noise::Perlin::new(seed);

        let sub = sub::Sub::new(&device, &perlin);
        
        let mut world = world::World::new();
        world.update_nearby(&sub);
        world.build_full(&perlin, &device);

        let boid_manager = boid::BoidManager::new(&sub, &perlin, &device, &queue, &texture_bind_group_layout);
        //--------------------------------------------------------------------//

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            brush,
            terrain_render_pipeline,
            sub_render_pipeline,
            fish_render_pipeline,
            depth_texture,
            camera,
            camera_buffer,
            camera_bind_group,
            fps_counter,
            fpses,
            perlin,
            sub,
            world,
            boid_manager,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");

            self.brush.resize_view(self.config.width as f32, self.config.height as f32, &self.queue);
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.sub.process_events(event)
    }

    pub fn update(&mut self) {
        let delta = self.fps_counter.update();
        self.fpses.push(self.fps_counter.fps() as f32);

        let old_len = self.fpses.len();
        let average_fps: f32 = self.fpses.iter().sum::<f32>() / old_len as f32;
        let length = (FPSES_TO_KEEP * average_fps).round() as usize;

        let mut new_fpses = Vec::with_capacity(length);
        let start = (old_len as i32 - length as i32).max(0) as usize;
        for i in start..old_len {
            new_fpses.push(self.fpses[i]);
        }
        self.fpses = new_fpses;

        self.sub.update(&self.queue, delta as f32);
        self.sub.update_camera(&mut self.camera, delta as f32);

        self.world.update(&self.sub, &self.perlin, &self.device);

        self.boid_manager.update(&self.queue, &self.perlin, &self.sub, &self.world, delta as f32);

        self.queue.write_buffer(&self.camera_buffer, 0, bytemuck::cast_slice(&[*self.camera.uniform()]));
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(consts::CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                occlusion_query_set: None,
                timestamp_writes: None,
            });


            //----------------------------------------------------------------//
            render_pass.set_pipeline(&self.terrain_render_pipeline);

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for pos in self.world.chunks_to_render() {
                let chunk = self.world.get_chunk(*pos).unwrap();
                render_pass.set_vertex_buffer(0, chunk.verts_buffer_slice());
                render_pass.draw(0..chunk.num_verts() as u32, 0..1);
            }
            //----------------------------------------------------------------//

            //----------------------------------------------------------------//
            render_pass.set_pipeline(&self.sub_render_pipeline);

            render_pass.set_vertex_buffer(0, self.sub.verts_buffer_slice());
            render_pass.set_vertex_buffer(1, self.sub.inst_buffer_slice());
            render_pass.draw(0..self.sub.num_verts() as u32, 0..1);

            render_pass.set_vertex_buffer(0, self.sub.prop_verts_buffer_slice());
            render_pass.set_vertex_buffer(1, self.sub.prop_inst_buffer_slice());
            render_pass.draw(0..self.sub.num_prop_verts() as u32, 0..1);
            //----------------------------------------------------------------//

            //----------------------------------------------------------------//
            render_pass.set_pipeline(&self.fish_render_pipeline);

            for species in &boid::ALL_SPECIES {
                render_pass.set_bind_group(1, self.boid_manager.diffuse_bind_group(*species), &[]);

                render_pass.set_vertex_buffer(0, self.boid_manager.verts_buffer_slice(*species));
                render_pass.set_vertex_buffer(1, self.boid_manager.inst_buffer_slice(*species));

                render_pass.draw(0..self.boid_manager.num_verts(*species) as u32, 0..self.boid_manager.num_inst(*species) as u32);
            }
            //----------------------------------------------------------------//
        }
        {
            let mut brush_render_pass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });

            let scale = self.size.width.min(self.size.height) as f32;
            let font_size = scale * TEXT_SIZE;
            let text_spacing = scale * TEXT_SPACING;

            let min_fps = self.fpses.clone().into_iter().reduce(f32::min).unwrap();
            let pos = self.sub.pos();
            let bearing = self.sub.bearing();

            let fps_text = format!("FPS: {:3.0}", self.fps_counter.fps());
            let min_text = format!("99% FPS: {:3.0}", min_fps);
            let pos_text = format!("POS: {:.0} {:.0} {:.0}", pos.x, pos.y, pos.z);
            let bearing_text = format!("BEARING: {:.3} {:.3} {:.3}", bearing.x, bearing.y, bearing.z);
            let generate_text = format!("GENERATE: {}", self.world.generate_count());
            let render_text = format!("RENDER: {}", self.world.render_count());
            let total_text = format!("TOTAL: {}", self.world.total_count());

            let texts = vec![fps_text, min_text, pos_text, bearing_text, generate_text, render_text, total_text];
            let overall_text = texts.join("\n");

            let selection = wgpu_text::glyph_brush::Section::default()
                .add_text(wgpu_text::glyph_brush::Text::new(&overall_text)
                    .with_scale(font_size)
                    .with_color([236.0 / 255.0, 239.0 / 255.0, 244.0 / 255.0, 1.0])
                )
                .with_layout(
                    wgpu_text::glyph_brush::Layout::default()
                        .v_align(wgpu_text::glyph_brush::VerticalAlign::Top)
                        .line_breaker(wgpu_text::glyph_brush::BuiltInLineBreaker::AnyCharLineBreaker),
                )
                .with_screen_position((text_spacing, text_spacing))
                .to_owned();


            let _ = self.brush.queue(&self.device, &self.queue, vec![&selection]);

            self.brush.draw(&mut brush_render_pass);
            // self.brush.draw(&mut render_pass);
        }
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        // submit will accept anything that implements IntoIter
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        //--------------------------------------------------------------------//

        Ok(())
    }

    pub fn window(&self) -> &winit::window::Window {
        &self.window
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }
}
