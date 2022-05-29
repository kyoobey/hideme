


use winit::{
	dpi::{ PhysicalSize, PhysicalPosition },
	window::Window
};
use wgpu;
use wgpu::util::DeviceExt;

mod image_texture;



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
	position: [f32; 2],
}

impl Vertex {
	fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
		wgpu::VertexBufferLayout {
			array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
			step_mode: wgpu::VertexStepMode::Vertex,
			attributes: &[
				wgpu::VertexAttribute {
					offset: 0,
					shader_location: 0,
					format: wgpu::VertexFormat::Float32x2
				}
			]
		}
	}
}

const VERTICES: [Vertex; 4] = [
	Vertex { position: [-1.0, -1.0] },
	Vertex { position: [-1.0,  1.0] },
	Vertex { position: [ 1.0,  1.0] },
	Vertex { position: [ 1.0, -1.0] }
];

const INDICES: &[u16] = &[
	2, 1, 0,
	3, 2, 0
];



#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
	top_left: [f32; 2],
	bottom_right: [f32; 2],
}

impl Uniforms {
	fn new() -> Self {
		Self {
			top_left: [0.0; 2],
			bottom_right: [0.0; 2],
		}
	}

	fn update(
		&mut self,
		top_left: PhysicalPosition<i32>,
		bottom_right: PhysicalPosition<i32>,
		monitor_size: PhysicalSize<u32>
	) {
		let monitor_size = [ monitor_size.width as f32, monitor_size.height as f32 ];
		self.top_left = [ top_left.x as f32 / monitor_size[0], top_left.y as f32 / monitor_size[1] ];
		self.bottom_right = [ bottom_right.x as f32 / monitor_size[0], bottom_right.y as f32 / monitor_size[1] ];
	}
}



pub struct RenderState {
	draw_shader: bool,

	surface: wgpu::Surface,
	device: wgpu::Device,
	queue: wgpu::Queue,
	config: wgpu::SurfaceConfiguration,
	pub window_size: winit::dpi::PhysicalSize<u32>,
	render_pipeline: wgpu::RenderPipeline,

	vertex_buffer: wgpu::Buffer,
	// num_vertices: u32, // not needed, keep for future
	index_buffer: wgpu::Buffer,
	num_indices: u32,

	clear_color: wgpu::Color,
	// background_texture: image_texture::ImageTexture, // texture does not need to be stored for now, might need for future
	background_bind_group: wgpu::BindGroup,

	uniforms: Uniforms,
	uniform_buffer: wgpu::Buffer,
	uniform_bind_group: wgpu::BindGroup
}

impl RenderState {

	pub async fn new(
		window: &Window,
		color: (f64, f64, f64, f64),
		background_bytes: &Vec<u8>,
		draw_shader: bool,
	) -> Self {
		let window_size = window.inner_size();

		// let instance = wgpu::Instance::new(wgpu::Backends::all());
		let instance = wgpu::Instance::new(if cfg!(windows) { wgpu::Backends::DX12 } else { wgpu::Backends::all() });
		let surface = unsafe { instance.create_surface(&window) };
		let adapter = instance.request_adapter(
			&wgpu::RequestAdapterOptions {
				// power_preference: wgpu::PowerPreference::HighPerformance,
				power_preference: wgpu::PowerPreference::LowPower,
				compatible_surface: Some(&surface),
				force_fallback_adapter: false
			}
		).await.unwrap();

		let (device, queue) = adapter.request_device(
			&wgpu::DeviceDescriptor {
				features: wgpu::Features::empty(),
				limits: wgpu::Limits::downlevel_webgl2_defaults().using_resolution(adapter.limits()),
				label: None
			},
			None
		).await.unwrap();

		let config = wgpu::SurfaceConfiguration {
			usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
			format: surface.get_preferred_format(&adapter).unwrap(),
			width: window_size.width,
			height: window_size.height,
			present_mode: wgpu::PresentMode::Fifo
		};
		surface.configure(&device, &config);

		let clear_color = wgpu::Color {
			r: color.0,
			g: color.1,
			b: color.2,
			a: color.3,
		};

		let background_texture = image_texture::ImageTexture::from_bytes(&device, &queue, background_bytes, "background texture")
			.expect("[Error] Couldn't read background image");

		let background_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Texture {
							multisampled: false,
							view_dimension: wgpu::TextureViewDimension::D2,
							sample_type: wgpu::TextureSampleType::Float { filterable: true }
						},
						count: None
					},
					wgpu::BindGroupLayoutEntry {
						binding: 1,
						visibility: wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
						count: None
					}
				],
				label: Some("background_texture_bind_group_layout")
			}
		);

		let background_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &background_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: wgpu::BindingResource::TextureView(&background_texture.view)
					},
					wgpu::BindGroupEntry {
						binding: 1,
						resource: wgpu::BindingResource::Sampler(&background_texture.sampler)
					}
				],
				label: Some("background_bind_group")
			}
		);


		let uniforms = Uniforms::new();

		let uniform_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Uniform Buffer"),
				contents: bytemuck::cast_slice(&[uniforms]),
				usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST
			}
		);

		let uniform_bind_group_layout = device.create_bind_group_layout(
			&wgpu::BindGroupLayoutDescriptor {
				entries: &[
					wgpu::BindGroupLayoutEntry {
						binding: 0,
						visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
						ty: wgpu::BindingType::Buffer {
							ty: wgpu::BufferBindingType::Uniform,
							has_dynamic_offset: false,
							min_binding_size: None
						},
						count: None
					}
				],
				label: Some("uniform_bind_group_layout")
			}
		);

		let uniform_bind_group = device.create_bind_group(
			&wgpu::BindGroupDescriptor {
				layout: &uniform_bind_group_layout,
				entries: &[
					wgpu::BindGroupEntry {
						binding: 0,
						resource: uniform_buffer.as_entire_binding()
					}
				],
				label: Some("uniform_bind_group")
			}
		);


		let shader = device.create_shader_module(
			&wgpu::ShaderModuleDescriptor {
				label: Some("shader"),
				source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into())
			}
		);

		let vertices = VERTICES;
		let vertex_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Vertex Buffer"),
				contents: bytemuck::cast_slice(&vertices),
				usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST
			}
		);
		// let num_vertices = vertices.len() as u32;

		let index_buffer = device.create_buffer_init(
			&wgpu::util::BufferInitDescriptor {
				label: Some("Index Buffer"),
				contents: bytemuck::cast_slice(INDICES),
				usage: wgpu::BufferUsages::INDEX
			}
		);
		let num_indices = INDICES.len() as u32;

		let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
			label: Some("Render Pipeline Layout"),
			bind_group_layouts: &[
				&background_bind_group_layout,
				&uniform_bind_group_layout
			],
			push_constant_ranges: &[]
		});

		let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
			label: Some("Render Pipeline"),
			layout: Some(&render_pipeline_layout),
			vertex: wgpu::VertexState {
				module: &shader,
				entry_point: "vs_main",
				buffers: &[
					Vertex::desc()
				]
			},
			fragment: Some(wgpu::FragmentState {
				module: &shader,
				entry_point: "fs_main",
				targets: &[wgpu::ColorTargetState {
					format: config.format,
					blend: Some(wgpu::BlendState::REPLACE),
					write_mask: wgpu::ColorWrites::ALL
				}]
			}),
			primitive: wgpu::PrimitiveState {
				topology: wgpu::PrimitiveTopology::TriangleList,
				strip_index_format: None,
				front_face: wgpu::FrontFace::Ccw,
				cull_mode: Some(wgpu::Face::Back),
				polygon_mode: wgpu::PolygonMode::Fill,
				unclipped_depth: false,
				conservative: false
			},
			depth_stencil: None,
			multisample: wgpu::MultisampleState {
				count: 1,
				mask: 10,
				alpha_to_coverage_enabled: false
			},
			multiview: None
		});


		Self {
			draw_shader,
			surface, device, queue, config, window_size,
			// vertex_buffer, num_vertices, index_buffer, num_indices,
			vertex_buffer, index_buffer, num_indices, // num_vertices not needed for now
			// clear_color, background_texture, background_bind_group,
			clear_color, background_bind_group,
			uniforms, uniform_buffer, uniform_bind_group,
			render_pipeline
		}
	}



	pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
		let output = self.surface.get_current_texture()?;
		let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
		let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
			label: Some("Render Encoder")
		});

		{
			let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
				label: Some("Render Pass"),
				color_attachments: &[
					wgpu::RenderPassColorAttachment {
						view: &view,
						resolve_target: None,
						ops: wgpu::Operations {
							load: wgpu::LoadOp::Clear(self.clear_color),
							store: true
						}
					}
				],
				depth_stencil_attachment: None
			});

			if self.draw_shader {
				render_pass.set_pipeline(&self.render_pipeline);
				render_pass.set_bind_group(0, &self.background_bind_group, &[]);
				render_pass.set_bind_group(1, &self.uniform_bind_group, &[]);
				render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
				render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
				render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
			}
		}

		self.queue.submit(std::iter::once(encoder.finish()));
		output.present();

		Ok(())
	}



	pub fn update(&mut self, window: &Window) {
		let top_left = window.outer_position().ok().unwrap();
		let window_size = window.inner_size();
		let bottom_right = PhysicalPosition::new(top_left.x + window_size.width as i32, top_left.y + window_size.height as i32);
		let monitor_size = window.current_monitor().expect("[Error] Couldn't get current monitor").size();

		self.uniforms.update(top_left, bottom_right, monitor_size);
		self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[self.uniforms]));

		self.config.width = window_size.width;
		self.config.height = window_size.height;
		// self.surface.configure(&self.device, &self.config); // slows down the program a lot lol
	}



	// for future
	// pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
	// 	if new_size.width > 0 && new_size.height > 0 {
	// 		self.window_size = new_size;
	// 		self.config.width = new_size.width;
	// 		self.config.height = new_size.height;
	// 		self.surface.configure(&self.device, &self.config);
	// 	}
	// }

}


