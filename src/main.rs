


#[macro_use]
extern crate log;
use simple_logger;

use clappers;
use itertools::Itertools;
use winit::{
	dpi::{ PhysicalSize, PhysicalPosition },
	event::*,
	event_loop::{ ControlFlow, EventLoop },
	window::WindowBuilder
};
use wallpaper;

use std::str::FromStr;
use std::fs::File;
use std::io::Read;

mod render_state;



fn main() {


	let clappers = clappers::Clappers::build()
		.set_flags(vec![
			"h|help",
			"b|background"
		])
		.set_singles(vec![
			"c|color",
			"v|loglevel"
		])
		.parse();

	if clappers.get_flag("help") {
		println!("\
			A small utility to hide portions of your screen \n\n\
			usage: hideme [arguments]\n\
			Arguments:\n\
				\t-h|--help		Print this help message\n\
				\t-c|--color <r,g,b,a>	The color value of the window\n\
				\t-v|--loglevel <n>	Changes the log level 0 (lowest) to 3 (highest)\n\
				\t-b|--background	Flag for setting the background to desktop wallper (use with conjunction with color arguments to get a tinted background)
		");
		std::process::exit(0);
	}


	simple_logger::SimpleLogger::new()
		.with_level(match clappers.get_single("loglevel").parse()
			.unwrap_or_else(|_err| 0) {
			0 => log::LevelFilter::Error,
			1 => log::LevelFilter::Warn,
			2 => log::LevelFilter::Info,
			3 => log::LevelFilter::Debug,
			_ => {
				println!("[Warning] Log Level should be in the range 0..3");
				log::LevelFilter::Error
			}
		})
		.init()
		.expect("Couldn't initialize logger");


	let (r, g, b, a) = clappers.get_single("color")
		.split(",")
		.map(|n| f64::from_str(n).unwrap_or_else(|_err| 0_f64))
		.collect_tuple().unwrap_or_else(|| {
			warn!("Something went wrong while parsing the color arguments");
			(1_f64, 0_f64, 0_f64, 1_f64)
		});

	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("hideme")
		.with_inner_size(PhysicalSize::new(1280_i32, 720_i32))
		.with_decorations(false)
		.with_transparent(true)
		.with_always_on_top(true)
		.build(&event_loop)
		.unwrap();


	let background_bytes = if clappers.get_flag("background") {
		let mut f = File::open(
			&wallpaper::get()
				.ok()
				.expect("[Error] Couldn't read wallpaper file, check permissions")
				.as_str()
		).ok().expect("[Error] Couldn't open wallpaper file, check permissions");
		let mut buf = Vec::new();
		f.read_to_end(&mut buf).ok().expect("[Error] Couldn't read bytes from wallpaper file");
		buf
	} else {
		include_bytes!("black_1920x1080.png").to_vec()
	};



	let mut render_state = pollster::block_on(
		render_state::RenderState::new(
			&window,
			(r, g, b, a),
			&background_bytes,
			clappers.get_flag("background"),
		)
	);


	let mut mouse_pressed = false;
	let mut last_mouse_position = PhysicalPosition::new(0.0, 0.0);
	let mut resizing_window = false;
	let mouse_padding = 100.0;

	event_loop.run(move |event, _, control_flow| match event {
		Event::RedrawRequested(window_id) if window_id == window.id() => {
			match render_state.render() {
				Ok(_) => {},
				// Err(wgpu::SurfaceError::Lost) => render_state.resize(render_state.window_size), // just copied from another place, prob will remove later
				Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
				Err(e) => error!("{:?}", e)
			}
		},
		Event::RedrawEventsCleared => {
			window.request_redraw();
		},

		Event::WindowEvent {
			event: WindowEvent::MouseInput {
				button: MouseButton::Left,
				state,
				..
			}, ..
		} => match state == ElementState::Pressed {
			true => mouse_pressed = true,
			false => {
				resizing_window = false;
				mouse_pressed = false;
			}
		}
		Event::WindowEvent {
			event: WindowEvent::CursorMoved { position, ..}, ..
		} => {
			last_mouse_position = position;
		},
		Event::DeviceEvent {
			event: DeviceEvent::MouseMotion { delta },
			..
		} => if mouse_pressed {
			let window_size = window.inner_size();
			if !resizing_window
			&& mouse_padding < last_mouse_position.x && last_mouse_position.x < (window_size.width - mouse_padding as u32).into()
			&& mouse_padding < last_mouse_position.y && last_mouse_position.y < (window_size.height - mouse_padding as u32).into() {
				let top_left_position = window.outer_position().ok().unwrap_or_else(|| {
					error!("Couldn't read window position");
					PhysicalPosition::new(0, 0)
				});
				window.set_outer_position(
					PhysicalPosition::new(
						top_left_position.x as i32 + delta.0 as i32,
						top_left_position.y as i32 + delta.1 as i32
					)
				);
			} else {
				resizing_window = true;
				window.set_inner_size(
					PhysicalSize::new(
						window_size.width as i32 + delta.0 as i32,
						window_size.height as i32+ delta.1 as i32
					)
				);
			}
			render_state.update(&window);
		},

		Event::WindowEvent {
			event: WindowEvent::CloseRequested,
			..
		} | Event::WindowEvent {
			event: WindowEvent::KeyboardInput {
				input: KeyboardInput {
					state: ElementState::Pressed,
					virtual_keycode: Some(VirtualKeyCode::Escape),
					..
				},
				..
			},
			..
		} => *control_flow = ControlFlow::Exit,
		_ => ()
	});

}


