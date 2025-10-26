extern crate sdl3 as sdl;

use std::thread::sleep;
use sdl3::rect::Rect;
use sdl3::pixels::Color;
use std::path::Path;
use std::time::Duration;
use image::imageops::CatmullRom;
use image::{GenericImage, Pixel, RgbImage};
use takeflight_computer_vision as cv;
use cv::{Error, ComputerVision};
use takeflight_computer_vision::HandLandmarker;

fn main() -> Result<(), Error>
{
	// initialize SDL
	let sdl_context = sdl::init()?;
	let video_sub = sdl_context.video()?;
	let width = cv::HandLandmarker::WIDTH;
	let height = cv::HandLandmarker::HEIGHT;

	let window = video_sub.window("Draw Fingers Example", width as u32, height as u32)
	.build()?;
	let mut canvas = window.into_canvas();
	canvas.present();

	// Load image
	let image = image::open("tests/test_data/open_palm.png")?.resize_exact(HandLandmarker::WIDTH as u32, HandLandmarker::HEIGHT as u32, CatmullRom);
	let hand_image = image.clone().into_rgb32f();
	Path::new("model/hand_detector.tflite");
	let mut landmarker = HandLandmarker::new()?;
	let output = landmarker.run_model(hand_image.clone())?;

	// Get fingers
	let fingers = HandLandmarker::get_digits(&output);

	// Plot everything
	let mut image = image.into_rgb8();
	let image_width = image.width();
	for (idx, finger) in fingers.iter().enumerate()
	{
		let color = match idx {
			0 => (0xFF, 0x00, 0x00),
			1 => (0x00, 0xFF, 0x00),
			2 => (0xFF, 0x00, 0xFF),
			3 => (0xFF, 0x00, 0xFF),
			4 => (0xFF, 0xFF, 0x00),
			_ => (0x00, 0x00, 0x00),
		};

		for i in 0..finger.len() - 1
		{
			let point_1 = finger[i];
			let point_2 = finger[i + 1];
			plot_line(&mut image,
					Coord2D { x: point_1.x as usize, y: point_1.y as usize},
					Coord2D { x: point_2.x as usize, y: point_2.y as usize},
					image_width as usize,
					color,
			);
		}
	}

	render_image(&mut canvas, &image)
}

struct Coord2D<T>
{
	x : T,
	y : T
}


fn get_pixel_index(x : usize, y : usize, row_width : usize, pixel_depth : usize ) -> usize
{
	(row_width * y + x) * pixel_depth
}

// DDA line plot as found here: https://www.geeksforgeeks.org/computer-graphics/dda-line-generation-algorithm-computer-graphics/
// adapted from C to Rust
fn plot_line(canvas: &mut [u8], start: Coord2D<usize>, end : Coord2D<usize>, row_width : usize, color : (u8,u8,u8))
{
	const PIXEL_DEPTH : usize = 3;
	let dx = end.x as i32 - start.x as i32;
	let dy = end.y as i32 - start.y as i32;

	let steps = ((if dx.abs() > dy.abs() {dx} else {dy}) as f32).abs(); // floats can't be compared

	let x_increment = dx as f32 / steps;
	let y_increment = dy as f32 / steps;

	let mut x = start.x as f32;
	let mut y = start.y as f32;

	for _ in 0 ..(steps as usize).saturating_sub(1) //This -1 is required in every situation. We sat_sub so that if we have 0 steps, we don't underflow.
	{
		x += x_increment;
		y += y_increment;

		let index = get_pixel_index(x as usize, y as usize, row_width, 3);

		let pixel = &mut canvas[index..index + PIXEL_DEPTH];
		pixel[0] = color.0;
		pixel[1] = color.1;
		pixel[2] = color.2;
	}
}

fn render_image(canvas : &mut sdl3::render::Canvas<sdl3::video::Window>, im : &RgbImage) -> Result<(), Error>
{
	let image_width = im.width();
	for (idx, pixel) in im.pixels().enumerate()
	{
		let y = (idx as u32 / image_width) as i32;
		let x = (idx as u32 % image_width) as i32;

		let pixel_color = pixel.to_rgb();

		canvas.set_draw_color(Color::RGB(pixel_color.0[0], pixel_color.0[1], pixel_color.0[2]));
		canvas.fill_rect(Rect::new(x, y , 1, 1))?;
	}

	canvas.present();

	sleep(Duration::from_secs(10));

	Ok(())
}