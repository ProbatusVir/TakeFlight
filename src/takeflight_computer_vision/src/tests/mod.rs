mod computer_vision_test;

/// width is measured in pixels, and stride is the size of each pixel in bytes.
fn get_mut_pixel<T>(image : &mut [T], x : usize, y : usize, width : usize, stride : usize) -> &mut [T]
{
	let row_first_pixel = y * width;
	let index = (row_first_pixel + x) * stride;
	&mut image[index..index + stride]
}