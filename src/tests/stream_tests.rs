use crate::tests::TEST_PATH;
use std::fs::File;
use std::io::{Error, Write};
use openh264::formats::YUVSource;
use crate::tests::mock_camera::MockCamera;
use rstest::rstest;

#[rstest]
fn test_mock_camera_init() -> Result<(), Error>
{
	MockCamera::new()?;

	Ok(())
}

#[rstest]
fn mock_camera_can_capture() -> Result<(), Error>
{
	let mut cam = MockCamera::new()?;
	cam.snapshot()?;

	Ok(())
}

// Can't test inverse, because this is a lossy conversion.
#[rstest]
fn camera_to_video() -> Result<(), Error>
{
	// Initialize camera parameters
	let mut cam = MockCamera::new()?;
	let _ = cam.snapshot()?;
	cam.encode_existing_image()?;

	Ok(())
}

#[rstest]
fn camera_to_video_to_file() -> Result<(), Error>
{
	let mut cam = MockCamera::new()?;
	let _ = cam.snapshot()?;
	let packet = cam.encode_existing_image()?;
	let latest_image = cam.decoder.decode(&packet).map_err(|_| Error::other("Could not decode encoded packet."))?;

	// If there's no image, the test fails.
	if latest_image.is_none()
	{
		Err(Error::other("There was no latest frame???"))?
	}
	let latest_image = latest_image.unwrap();
	// Prepare to write to a file

	let mut rgb_buffer = vec![0;latest_image.rgb8_len()];
	latest_image.write_rgb8(&mut rgb_buffer);

	let mut file = File::create(format!("{}{}", TEST_PATH, "last_video_frame.rgb"))?;
	file.write_all(&mut rgb_buffer)?;

	Ok(())
}