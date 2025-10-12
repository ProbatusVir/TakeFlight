mod pixel_tests
{
	use crate::helper;
	use crate::helper::rgb_pixel_to_yuv_pixel_in_place;
	use crate::helper::yuv_image_to_rgb_image;
	use rstest::rstest;

	#[rstest]
	fn rgb_to_yuv_test()
	{
		assert_eq!(helper::rgb_pixel_to_yuv_pixel(114, 157, 3), [126, 58, 119]);
	}

	#[rstest]
	fn yuv_to_rgb_test()
	{
		assert_eq!(helper::yuv_pixel_to_rgb_pixel(126, 58, 119), [113, 156, 1]);
	}

	#[rstest]
	fn rgb_to_yuv_in_place_test()
	{
		let mut pixel = [114, 157, 3];
		rgb_pixel_to_yuv_pixel_in_place(&mut pixel);
		assert_eq!(pixel, [126, 58, 119]);
	}

	#[rstest]
	fn yuv_to_rgb_in_place_test()
	{
		let mut pixel = [126, 58, 119];
		yuv_image_to_rgb_image(&mut pixel);
		assert_eq!(pixel, [113, 156, 1]);
	}
}

