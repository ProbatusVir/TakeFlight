use ffmpeg_next::software::scaling::Flags;
use ffmpeg_next::format::Pixel;
use std::fs::File;
use std::io::Write;
use image::{ RgbImage};
use ffmpeg_next as ffmpeg;
use ffmpeg_next::{codec, Rescale};
use ffmpeg_next::format::format;

pub fn raw_h264_to_rgb(image_buffer : Box<[u8]>) -> Option<RgbImage>
{
	
	/*let codec = ffmpeg::codec::decoder::find(ffmpeg::codec::Id::H264).unwrap();
	let mut context = ffmpeg::codec::Context::new();
	context.set_parameters(codec.parameters()).unwrap(); // Or whatever gets ye Parameters
	let decoder = ffmpeg::codec::decoder::Decoder(context);
	let mut video_decoder = decoder.video().unwrap();

	let packet = ffmpeg::Packet::copy(&image_buffer);

	video_decoder.send_packet(&packet).unwrap();

	let mut frame = ffmpeg::frame::video::Video::empty();
	while video_decoder.receive_frame(&mut frame).is_ok() {
		println!("Got frame: {}x{}", frame.width(), frame.height());
	}

	let width = frame.width();
	let height = frame.height();

	RgbImage::from_vec(width, height, frame.data(0).into())
	*/

	match try_h264_to_rgb(&image_buffer)
	{
		Ok(image ) => { Some(image) }
		Err(e) => { dbg!(e); None } // we generally don't care about a conversion error, all we care about is if there's an image or not.
	}

	//Ok(frames)


	/*

	let mut decoder = match openh264::decoder::Decoder::new() {
		Ok(de) => { de }
		Err(e) => { None? }
	};

	let decoder_result = decoder.decode(&image_buffer);
	match decoder_result
	{
		Ok(decoded_option) =>
			{
				/*
				let decoded = decoded_option.unwrap();
				let (w,h) = decoded.dimensions();
				self.logger.info_from_string(format!("We successfully decoded frame {frame_number}, {w}x{h}"))?;
				let mut file = File::create(format!("test_results/frame{frame_number}.rgb"))?;
				decoded.write_rgb8(&mut file_buffer);
				file.write_all(&file_buffer)?;*/

				// Send the image to the client, if possible.
				let decoded = decoded_option?;
				let (w,h) = decoded.dimensions();
				let mut decoded_image = image::RgbImage::new(w as u32, h as u32);
				decoded.write_rgb8(&mut decoded_image);
				Some(decoded_image)
			}
		Err(_) =>
			{ 	//We actually don't really care about this error.
				//self.logger.error_from_string(format!("Received malformed video frame {frame_number}"))?;
				//File::create(format!("test_results/malformed{frame_number}"))?.write_all(&image_buffer)?
				None
			}
	}*/
}

fn try_h264_to_rgb(image_buffer: &[u8]) -> Result<RgbImage, ffmpeg::Error>
{
	ffmpeg::init()?;

	let codec = ffmpeg::codec::decoder::find(codec::Id::H264).ok_or(ffmpeg::Error::DecoderNotFound)?;
	let mut decoder = codec::context::Context::new_with_codec(codec).decoder().video()?;

	let packet = ffmpeg::Packet::copy(&image_buffer);
	decoder.send_packet(&packet)?;

	let mut decoded = ffmpeg::util::frame::Video::empty();


	// This will put YUV420 into decoded.
	decoder.receive_frame(&mut decoded)?;

	// The scaler also handles YUV->RGB conversion
	let scaler = ffmpeg::software::scaling::Context::get(
		Pixel::YUV420P,
		decoded.width(),
		decoded.height(),
		Pixel::RGB24,
		decoded.width(),
		decoded.height(),
		Flags::BILINEAR,
	);

	let mut rgb_frame = ffmpeg::util::frame::Video::empty();
	scaler?.run(&decoded, &mut rgb_frame)?;

	RgbImage::from_vec(decoded.width(), decoded.height(), rgb_frame.data(0).to_vec()).ok_or(ffmpeg::Error::InvalidData)
}


/*
let mut image_buffer = Vec::new();
						image_buffer.extend_from_slice(&self.sps.unwrap());
						image_buffer.extend_from_slice(&self.pps.unwrap());
						image_buffer.extend_from_slice(&self.idr);
						image_buffer.extend_from_slice(&self.frame_buffer);


						let mut decoder = openh264::decoder::Decoder::new()?;
						let decoder_result = decoder.decode(&image_buffer);
						match decoder_result
						{
							Ok(decoded_option) =>
								{
									/*
									let decoded = decoded_option.unwrap();
									let (w,h) = decoded.dimensions();
									self.logger.info_from_string(format!("We successfully decoded frame {frame_number}, {w}x{h}"))?;
									let mut file = File::create(format!("test_results/frame{frame_number}.rgb"))?;
									decoded.write_rgb8(&mut file_buffer);
									file.write_all(&file_buffer)?;*/

									// Send the image to the client, if possible.
									// TODO: I think this can be optimized for space if we initialize our image once, etc. etc.
									let decoded = decoded_option.unwrap();
									let (w,h) = decoded.dimensions();
									let mut decoded_image = image::RgbImage::new(w as u32, h as u32);
									decoded.write_rgb8(&mut decoded_image);
									self.image = Some(decoded_image.into());

									let now = SystemTime::now();
									if now.duration_since(self.last_frame_sent_time)? >= *self.frame_time
									{
										match self.send_image(Png)
										{
											Err(Error::NoVideoSource) => { self.logger.info("Tello didn't consider itself a valid video source?")? }
											Err(Error::NoVideoTarget) => { self.logger.info("No valid video destination.")? }
											Ok(_) => { self.logger.info("Video sent")?; }
											e => { self.logger.warn("Some error occurred while Tello was sending video...")?; e? }
										}
									}
								}
							Err(_) =>
								{ 	//We actually don't really care about this error.
									//self.logger.error_from_string(format!("Received malformed video frame {frame_number}"))?;
									//File::create(format!("test_results/malformed{frame_number}"))?.write_all(&image_buffer)?
								}
						}
 */