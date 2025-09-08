use std::io::{Error, Write};
use crate::helper::{rgb_image_to_yuv_image, yuv_image_to_rgb_image};
use std::fs::File;
use std::fs;
use rstest::rstest;
use crate::tests::mock_camera::MockCamera;
use crate::tests::TEST_PATH;

// This is taking a picture through YOUR camera.
#[rstest]
fn manual_test() -> Result<(), Error>
{
	// create the directory if it doesn't exist
	fs::create_dir(TEST_PATH).unwrap_or_default();

	// initialize camera instance, capture image
	let mut camera = MockCamera::new()?;
	let image = camera.snapshot()?;


	//	This may highlight a concept that might seem a little odd. This is called 'shadowing'
	//	Two variables can be declared with the same name (and the same or different type)
	//	Once this 'shadowing has occurred, the former variable can not be referenced by name, only
	//	if something else owns the value. Otherwise, it is destroyed when the other variable is created.
	//	* Should a variable be declared in an inner-scope while a variable of the same name exists in the
	//		encompassing scope, the name IS still valid in the outer scope, but the outer variable becomes
	//		inaccessible in the inner scope (by name)
	let mut file = File::create(format!("{}{}", TEST_PATH, "camera.rgb"))?;
	file.write_all(image)?;

	let mut transformed_image = image.clone();
	rgb_image_to_yuv_image(&mut transformed_image);
	let mut file = File::create(format!("{}{}", TEST_PATH, "camera.yuv"))?;
	file.write_all(&transformed_image)?;

	yuv_image_to_rgb_image(&mut transformed_image);
	let mut file = File::create(format!("{}{}", TEST_PATH, "camera_transformed.rgb"))?;
	file.write_all(&transformed_image)?;

	Ok(())
}