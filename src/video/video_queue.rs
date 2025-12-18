use std::sync::{Arc, Mutex};
use std::thread;
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use crate::Error;
use mio::{Interest, Poll, Token};
use crate::logger::Logger;
use crate::video::decode::{ raw_h264_to_rgb};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameType
{
	H264(),
	TelloH264(),
	Png(),
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VideoTask
{
	/// Should take an RGB image.
	Encode(FrameType),
	/// Should produce an RGB image.
	Decode(FrameType),
	/// From(FrameType) -> To(FrameType)
	Transcode(FrameType, FrameType),

	ShutDown,
}

#[derive(Debug)]
pub(crate) struct VideoTaskFull
{
	pub task		: VideoTask,
	pub image_data	: Box<[u8]>,
	pub origin		: Token,
}


/// This is strictly between sources of video and the Video Queue
/// Since the Video Queue has one purpose, it's fine for it to block.
#[derive(Clone, Debug)]
pub(crate) struct VideoQueue
{
	sender: std::sync::mpsc::Sender<VideoTaskFull>
}

impl VideoQueue
{
	/// Sealed.
	fn new() -> (Self, std::sync::mpsc::Receiver<VideoTaskFull>)
	{
		let (sender, receiver) = std::sync::mpsc::channel();
		(Self { sender }, receiver)
	}

	pub fn encode(&self, origin : Token, curr_src : Option<Token>, frame_type : FrameType, image_data : Box<[u8]>) -> Result<(), Error>
	{
		self.send_to_queue(origin, curr_src, image_data, VideoTask::Encode(frame_type))
	}

	pub fn decode(&self, origin : Token, curr_src : Option<Token>, frame_type : FrameType, image_data : Box<[u8]>) -> Result<(), Error>
	{
		self.send_to_queue(origin, curr_src, image_data, VideoTask::Decode(frame_type))
	}
	pub fn transcode(&self, origin : Token, curr_src : Option<Token>, from : FrameType, to : FrameType, image_data : Box<[u8]>) -> Result<(), Error>
	{
		self.send_to_queue(origin, curr_src, image_data, VideoTask::Transcode(from, to))
	}

	pub fn shutdown(&self) -> Result<(), Error>
	{
		let vtf = VideoTaskFull { task: VideoTask::ShutDown, image_data: Box::new([]), origin : Token(0) };
		self.sender.send(vtf).map_err(|_| { Error::Custom("Failed to send shutdown to the video stream thread. Did thread crash?") })
	}

	/// Start the work thread
	/// May incorporate the logger.
	pub fn start_work_thread(poll : &Poll, curr_src : Arc<Mutex<Option<Token>>>, logger: Logger) -> Result<(Self, mio_channel::Receiver<(Token, Box<[u8]>)>, thread::JoinHandle<Result<(), Error>>), Error>
	{
		let (queue, queue_receiver) = Self::new();
		let (sender_to_server, mut server_receiver) = mio_channel::channel::<(Token, Box<[u8]>)>();

		poll.registry().register(&mut server_receiver, crate::VIDEO_QUEUE, Interest::READABLE)?;

		let thread_handle = std::thread::Builder::new()
			.name("Video".into())
			.spawn(|| Self::work(queue_receiver, sender_to_server, curr_src, logger))?;

		Ok((queue, server_receiver, thread_handle))
	}

	/// A producer will send an image to the work queue
	///
	/// The work queue will process the image, and send the result to the server
	///
	/// This requires two sets of producers and consumers. We'll call these pairs A and B
	/// The producers own A_{Sender}, and the server owns B_{Receiver}, the other two are used for IO with the queue.
	/// This may seem like a complicated setup, but it's just a fat pointer being moved around, so minimal allocations are necessary.
	//  The parameters reflect the flow of this method.
	fn work(receiver: std::sync::mpsc::Receiver<VideoTaskFull>, sender : mio_channel::Sender<(Token, Box<[u8]>)>, curr_src : Arc<Mutex<Option<Token>>>, logger : Logger) -> Result<(), Error>
	{
		logger.info("Starting working queue!")?;

		loop {
			// We do not need to match the token, since we only ever expect one activity.
			let incoming_message = match receiver.recv() {
				Ok(message) => { message }
				Err(error) => { continue; } // opaque error.
			};

			dbg!("Received a message in video_queue");

			let frame = match incoming_message.task {
				VideoTask::Encode(to) => { todo!("Have not implemented encoding within the actual worker thread yet!") }
				VideoTask::Decode(from) => { todo!("Have not implemented decoding within the actual worker thread yet!") }
				VideoTask::Transcode(from, to) => {
					match (from, to)
					{
						(FrameType::TelloH264(), FrameType::Png()) => {
							let upgraded_h264 = raw_h264_to_rgb(incoming_message.image_data);
							match upgraded_h264 {
								Some(image) => {
									let mut png_buffer = Vec::new();
									let mut png_encoder = PngEncoder::new(&mut png_buffer);
									let (width, height) = image.dimensions();
									png_encoder.write_image(&*image, width, height, ExtendedColorType::Rgb8)?;
									sender.send((incoming_message.origin, png_buffer.into()))
										// FIXME: determine how we want to handle this.
										.unwrap_or(()); // If it fails to send, that's not big deal, for now...
								}
								None => { /* noop */ } // failed to transcode.
							}

						}
						_ => todo!("Not sure how to transcode this yet.")
					}

				}
				VideoTask::ShutDown => { break }
			}; // match incoming_message
		} // loop

		logger.warn("Shutting down!")?;
		Ok(())
	} // work

	#[inline(always)]
	fn is_current(origin : Token, curr_src : Token) -> bool
	{
		origin == curr_src
	}

	/// We only send to the queue if we are the current source of video.
	///
	/// If there is no current source of video, nothing must happen.
	///
	/// If there is a source of video, and we are not it, nothing must happen
	///
	/// If there is a source of video, and we are it, we must process the video.
	fn send_to_queue(&self, origin : Token, curr_src : Option<Token>, image_data : Box<[u8]>, task : VideoTask) -> Result<(), Error>
	{
		match curr_src
		{
			Some(token) => {
				if Self::is_current(origin, token)
				{
					let full_video_task = VideoTaskFull::new(origin, task, image_data);
					self.sender.send(full_video_task).map_err(|_| Error::Custom("Failed to send message to video queue!"))
				}
				// If this is not the current source of video, do nothing.
				else {
					Ok(())
				}
			}
			// If there is no current source of video, do nothing.
			None => { Ok(()) }
		} // match curr_src
	} // send_to_queue
} // VideoQueue


impl VideoTaskFull
{
	/// Sealed
	fn new(origin : Token, task : VideoTask, image_data : Box<[u8]>) -> Self
	{
		Self
		{
			origin,
			task,
			image_data
		}
	}
}