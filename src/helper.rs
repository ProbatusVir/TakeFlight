
#[allow(dead_code)]
// https://softpixel.com/~cwright/programming/colorspace/yuv/
pub fn rgb_pixel_to_yuv_pixel(r_ : u8, g_ : u8, b_: u8) -> [u8; 3] {
	let r = r_ as f32;
	let g = g_ as f32;
	let b = b_ as f32;
	let y = r *  0.299000 + g *  0.587000 + b *  0.114000;
	let u = r * -0.168736 + g * -0.331264 + b *  0.500000 + 128.0;
	let v = r *  0.500000 + g * -0.418688 + b * -0.081312 + 128.0;

	[y as u8, u as u8, v as u8]
}

#[allow(dead_code)]
pub fn rgb_pixel_to_yuv_pixel_in_place(pixel : &mut [u8]) {
	let r = pixel[0] as f32;
	let g = pixel[1] as f32;
	let b = pixel[2] as f32;
	pixel[0] = (r *  0.299000 + g *  0.587000 + b *  0.114000) as u8;
	pixel[1] = (r * -0.168736 + g * -0.331264 + b *  0.500000 + 128.0) as u8;
	pixel[2] = (r *  0.500000 + g * -0.418688 + b * -0.081312 + 128.0) as u8;

}

#[allow(dead_code)]
pub fn yuv_pixel_to_rgb_pixel(y_ : u8, u_ : u8, v_: u8) -> [u8; 3]
{
	let y = y_ as f32;
	let u = u_ as f32;
	let v = v_ as f32;
	let r = y + 1.4075 * (v - 128.0);
	let g = y - 0.3455 * (u - 128.0) - (0.7169 * (v - 128.0));
	let b = y + 1.7790 * (u - 128.0);

	[r as u8, g as u8, b as u8]
}

#[allow(dead_code)]
pub fn yuv_pixel_to_rgb_pixel_in_place(pixel : &mut [u8])
{
	let y = pixel[0] as f32;
	let u = pixel[1] as f32;
	let v = pixel[2] as f32;
	pixel[0] = (y + 1.4075 * (v - 128.0)) as u8;
	pixel[1] = (y - 0.3455 * (u - 128.0) - (0.7169 * (v - 128.0))) as u8;
	pixel[2] = (y + 1.7790 * (u - 128.0)) as u8;

}

#[allow(dead_code)]
// Might change to u32 or something, depending on usage...
pub fn rgb_image_to_yuv_image(image : &mut [u8])
{
	// The length of an rgb image (in bytes) should be some multiple of 3
	// This will give us a more helpful error during debug.
	debug_assert_eq!(image.len() % 3, 0);

	for pixel in image.chunks_exact_mut(3)
	{
		rgb_pixel_to_yuv_pixel_in_place(pixel);
	}
}

#[allow(dead_code)]
pub fn yuv_image_to_rgb_image(image : &mut [u8])
{
	// The length of an rgb image (in bytes) should be some multiple of 3
	// This will give us a more helpful error during debug.
	debug_assert_eq!(image.len() % 3, 0);

	for pixel in image.chunks_exact_mut(3)
	{
		yuv_pixel_to_rgb_pixel_in_place(pixel);
	}
}