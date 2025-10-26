use std::path::Path;
use anyhow::Error;
use const_format::concatcp;
use image::imageops::FilterType;
use image::Rgb32FImage;
use tflitec::model::Model;
use takeflight_computer_vision::ComputerVision;

pub const TEST_DATA : &str = "tests/test_data/";
pub const OPEN_PALM : &str = concatcp!(TEST_DATA, "open_palm.png");
pub const TWO_HANDS : &str = concatcp!(TEST_DATA, "two_hands.jpg");
pub const BLANK : &str = concatcp!(TEST_DATA, "blank.png");

pub fn load_image_data<P, CV>(path : P) -> Result<Rgb32FImage, Error>
where
	P	: AsRef<Path>,
	CV	: ComputerVision
{
	// Load in test data
	let mut image = image::open(path)?;
	Ok(
		image.resize_exact(CV::WIDTH as u32, CV::HEIGHT as u32, FilterType::CatmullRom).into_rgb32f()
	)
}


/// width is measured in pixels, and stride is the size of each pixel in bytes.
pub fn get_mut_pixel<T>(image : &mut [T], x : usize, y : usize, width : usize, stride : usize) -> &mut [T]
{
	let row_first_pixel = y * width;
	let index = (row_first_pixel + x) * stride;
	&mut image[index..index + stride]
}