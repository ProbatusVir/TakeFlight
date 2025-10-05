mod mock_camera;
mod stream_tests;
mod conversion_tests;
mod camera_conversion_test;
mod drone_flight_tests;
mod computer_vision_test;

const TEST_PATH : &str = "test_results/";

/// width is measured in pixels, and stride is the size of each pixel in bytes.
fn get_mut_pixel<T>(image : &mut [T], x : usize, y : usize, width : usize, stride : usize) -> &mut [T]
{
	let row_first_pixel = y * width;
	let index = (row_first_pixel + x) * stride;
	&mut image[index..index + stride]
}

