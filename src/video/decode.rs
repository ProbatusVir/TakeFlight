use image::RgbImage;
use openh264::formats::YUVSource;

pub fn raw_h264_to_rgb(image_buffer : Box<[u8]>) -> Option<RgbImage>
{
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
				let decoded = decoded_option.unwrap();
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
	}
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