use crate::{camera, chunk, consts, texture, timer, sub};

pub struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,

    // vertex_buffer: wgpu::Buffer,
    // index_buffer: wgpu::Buffer,

    // instances: Vec<draw::Instance>,
    // instance_buffer: wgpu::Buffer,

    depth_texture: texture::Texture,

    camera: camera::Camera,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,

    fps_counter: timer::FpsCounter,

    perlin: noise::Perlin,

    chunks: Vec<chunk::Chunk>,

    sub: sub::Sub,

    // The window must be declared after the surface so
    // it gets dropped after it as the surface contains
    // unsafe references to the window's resources.
    window: winit::window::Window,
}

impl State {
    // Creating some of the wgpu types requires async code
    pub async fn new(window: winit::window::Window) -> Self {
        use wgpu::util::DeviceExt;

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
                power_preference: wgpu::PowerPreference::HighPerformance,
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
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web, we'll have to disable some.
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
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result in all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
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
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[
                    &camera_bind_group_layout,
                ],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[chunk::Vert::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
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
                // Setting this to anything other than Fill requires Features::NON_FILL_POLYGON_MODE
                // polygon_mode: wgpu::PolygonMode::Line,
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
        // let vertex_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Vertex Buffer"),
        //         contents: bytemuck::cast_slice(draw::VERTICES),
        //         usage: wgpu::BufferUsages::VERTEX,
        //     }
        // );
        // let index_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Index Buffer"),
        //         contents: bytemuck::cast_slice(draw::INDICES),
        //         usage: wgpu::BufferUsages::INDEX,
        //     }
        // );
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        // let instances = (0..draw::NUM_INSTANCES_PER_ROW).flat_map(|y| {
        //     (0..draw::NUM_INSTANCES_PER_ROW).map(move |x| {
        //         let position = cgmath::Vector3 { x: x as f32, y: y as f32, z: 0.0 } - draw::INSTANCE_DISPLACEMENT;

        //         let rotation = if position.is_zero() {
        //             // this is needed so an object at (0, 0, 0) won't get scaled to zero
        //             // as Quaternions can affect scale if they're not created correctly
        //             cgmath::Quaternion::from_axis_angle(cgmath::Vector3::unit_z(), cgmath::Deg(0.0))
        //         } else {
        //             cgmath::Quaternion::from_axis_angle(position.normalize(), cgmath::Deg(45.0))
        //         };

        //         draw::Instance {
        //             position, rotation,
        //         }
        //     })
        // }).collect::<Vec<_>>();

        // let instance_data = instances.iter().map(draw::Instance::to_raw).collect::<Vec<_>>();
        // let instance_buffer = device.create_buffer_init(
        //     &wgpu::util::BufferInitDescriptor {
        //         label: Some("Instance Buffer"),
        //         contents: bytemuck::cast_slice(&instance_data),
        //         usage: wgpu::BufferUsages::VERTEX,
        //     }
        // );
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let fps_counter = timer::FpsCounter::new();
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let seed = (instant::now().round() % u32::MAX as f64) as u32;
        println!("Seed: {}", seed);
        let perlin = noise::Perlin::new(seed);
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let mut chunks = Vec::new();

        for x in -1..=1 {
            for y in -1..=1 {
                for z in -1..=1 {
                    let chunk = chunk::Chunk::new([x, y, z], &perlin, &device);
                    chunks.push(chunk);
                }
            }
        }
        //--------------------------------------------------------------------//

        //--------------------------------------------------------------------//
        let sub = sub::Sub::new();
        //--------------------------------------------------------------------//

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            // vertex_buffer,
            // index_buffer,
            // instances,
            // instance_buffer,
            depth_texture,
            camera,
            camera_buffer,
            camera_bind_group,
            fps_counter,
            perlin,
            chunks,
            sub,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            self.depth_texture = texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
        }
    }

    pub fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        // self.camera.process_events(event)
        self.sub.process_events(event)
    }

    pub fn update(&mut self) {
        let delta = self.fps_counter.update();
        println!("FPS: {:5.0}", self.fps_counter.fps());

        self.sub.update(delta as f32);
        self.sub.update_camera(&mut self.camera);

        // self.camera.update(delta);

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


            render_pass.set_pipeline(&self.render_pipeline);

            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            for chunk in self.chunks.iter() {
                render_pass.set_vertex_buffer(0, chunk.buffer_slice());
                render_pass.draw(0..chunk.num_verts() as u32, 0..1);
            }
            
            // render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            // render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
            // render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // render_pass.draw_indexed(0..draw::INDICES.len() as u32, 0, 0..self.instances.len() as _);
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
