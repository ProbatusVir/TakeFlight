mod helper;
mod video_stream;
mod drone_interface;
mod error;
#[cfg(test)]
mod tests;

use std::fs::File;
use std::io::{Read};
use image::EncodableLayout;
use image::imageops::FilterType;
use error::Error;

fn main() -> Result<(), Error> {
	println!("Hello, world!");

	
	/* The seven gestures supported are:
		* Open palm
		* Victory (peace)
		* Closed fist
		* Pointing up
		* Thumbs up
		* Thumbs down
		* Love (rock)
	*/

	// https://storage.googleapis.com/mediapipe-assets/Model%20Card%20Hand%20Tracking%20(Lite_Full)%20with%20Fairness%20Oct%202021.pdf
	// Hand detector model: 192x192x3 (rgb float [0.0, 1.0])
	const TENSOR_SIZE : usize = 2016 * 18;
	let mut model = opencv::dnn::Model::new("src/model/hand_detector.tflite", "")?;
	let im = image::open("src/tests/test_data/open_palm.png")?
		.resize_exact(192, 192, FilterType::CatmullRom)
		.into_rgb32f();


	let mut output_buffer = [0.0f32;TENSOR_SIZE];
	let mut output_tensor = //Mat::from_slice_mut(output_buffer.as_mut_slice())?;
	Mat::new_rows_cols_with_data(2016, 18, &output_buffer)?;

	let raw_mat = Mat::from_slice(&im)?;
	let input_tensor = raw_mat.reshape(3, 192)?;
	let mut output_for_realsies = output_tensor.clone_pointee();

	model.predict(&input_tensor, &mut output_for_realsies)?;

	Ok(())
}









