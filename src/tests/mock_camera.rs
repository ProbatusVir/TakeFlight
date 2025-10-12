use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat, RequestedFormatType};
use openh264::decoder::Decoder;
use openh264::encoder::Encoder;
use openh264::formats::{RgbSliceU8, YUVBuffer};
use std::io::Error;

pub(crate) struct Resolution
{
	pub w : usize,
	pub h : usize,
}

/// This is not meant for production.
pub(crate) struct MockCamera {
	pub camera	: nokhwa::Camera,
	pub encoder	: Encoder,
	pub decoder	: Decoder,
	image_buffer: Vec<u8>
}

impl MockCamera {
	pub fn new() -> Result<Self, Error> {
		// yuyv represents 2 pixels next to each other horizontally, where the u and v are shared but not the y. Should be 4 pixels wide.
		let mut camera = nokhwa::Camera::new(
			CameraIndex::Index(0),
			RequestedFormat::new::<RgbFormat>(RequestedFormatType::AbsoluteHighestFrameRate)
		).unwrap();
		camera.set_frame_rate(30).map_err(|e| Error::other(e))?;

		camera.open_stream().map_err(|_| Error::other("Could not open camera stream!!!"))?;

		let encoder = Encoder::new().map_err(|_| Error::other("Could not initialize encoder!!!"))?;
		let decoder = Decoder::new().map_err(|_| Error::other("Could not initialize decoder!!!"))?;
		let image_buffer = {
			let res = camera.resolution();
			vec![0; (res.height() * res.width() * 3) as usize]
		};
		Ok(Self { camera, encoder, decoder, image_buffer})
	}

	// For some reason we can't decode into YUYV, so we'll use RGB, since it's easy to work with.
	pub fn snapshot(&mut self) -> Result<&'_ Vec<u8>, Error>
	{
		self.camera.write_frame_to_buffer::<RgbFormat>(&mut self.image_buffer).map_err(|e| Error::other(e))?;

		Ok(&self.image_buffer)
	}

	pub fn resolution(&self) -> Resolution
	{
		let res = self.camera.resolution();
		Resolution {w : res.width() as usize, h: res.height() as usize }
	}

	pub fn encode_existing_image(&mut self) -> Result<Vec<u8>, Error>
	{
		let image = &self.image_buffer;
		let cam_res = self.resolution();
		let image = RgbSliceU8::new(&image, (cam_res.w, cam_res.h));
		let mut yuv = YUVBuffer::new(cam_res.w, cam_res.h);
		yuv.read_rgb8(image);

		let encoded_image = self.encoder.encode(&yuv).map_err(|_| Error::other("Could not encode image!"))?;

		let mut encoded_vec = Vec::new();
		encoded_image.write_vec(&mut encoded_vec);

		Ok(encoded_vec)
	}
}