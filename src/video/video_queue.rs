use std::ffi::c_void;
use crate::logger::Logger;
use crate::video::decode::raw_h264_to_rgb;
use crate::{Error, InternalSignal};
use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
use mio::{ Token };
use mio_wakeq::WakeQSender;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;
use takeflight_computer_vision as tfcv;
use takeflight_computer_vision::HandLandmarker;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameType
{
	TelloH264,
	Png,
	Rgb(u32, u32), // width and height


	#[allow(dead_code)]
	H264,
}


#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VideoTask
{
	#[allow(dead_code)]
	/// Should take an RGB image.
	Encode(FrameType),
	#[allow(dead_code)]
	/// Should produce an RGB image.
	Decode(FrameType),
	/// From(FrameType) -> To(FrameType)
	Transcode(FrameType, FrameType),

	/// Any type of image will do
	/// This is meant for images
	/// to be fed to the model to
	/// control the drone.
	CV(FrameType),

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

struct VideoQueueThreadInfo
{
	receiver	: std::sync::mpsc::Receiver<VideoTaskFull>,
	sender		: WakeQSender<InternalSignal>,
	curr_src	: Arc<Mutex<Option<Token>>>,
	logger		: Logger,
	model		: Option<tfcv::hand_landmarker::HandLandmarker>,
}

impl VideoQueue
{
	/// Sealed.
	fn new() -> (Self, std::sync::mpsc::Receiver<VideoTaskFull>)
	{
		let (sender, receiver) = std::sync::mpsc::channel();
		(Self { sender }, receiver)
	}

	#[allow(dead_code)]
	pub fn encode(&self, origin : Token, curr_src : Option<Token>, frame_type : FrameType, image_data : Box<[u8]>) -> Result<(), Error>
	{
		self.send_to_queue(origin, curr_src, image_data, VideoTask::Encode(frame_type))
	}

	#[allow(dead_code)]
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
	pub fn start_work_thread(curr_src : Arc<Mutex<Option<Token>>>, logger: Logger, internal_signaller : WakeQSender<InternalSignal>) -> Result<(Self, thread::JoinHandle<Result<(), Error>>), Error>
	{
		let (queue, queue_receiver) = Self::new();

		let thread_handle = std::thread::Builder::new()
			.name("Video".into())
			.spawn(|| {
					let thread_stuff = VideoQueueThreadInfo {
						receiver: queue_receiver,
						sender: internal_signaller,
						curr_src,
						logger,
						model : None // FIXME: No.
					};
					thread_stuff.do_work()
				})?;

		Ok((queue, thread_handle))
	}


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
	} // fn new
} // impl VideoTaskFull

impl VideoQueueThreadInfo
{
	/// A producer will send an image to the work queue
	///
	/// The work queue will process the image, and send the result to the server
	///
	/// This requires two sets of producers and consumers. We'll call these pairs A and B
	/// The producers own A_{Sender}, and the server owns B_{Receiver}, the other two are used for IO with the queue.
	/// This may seem like a complicated setup, but it's just a fat pointer being moved around, so minimal allocations are necessary.
	//  The parameters reflect the flow of this method.
	fn do_work(&self) -> Result<(), Error>
	{
		self.logger.info("Starting working queue!")?;

		loop {
			// We do not need to match the token, since we only ever expect one activity.
			let incoming_message = match self.receiver.recv() {
				Ok(message) => { message }
				Err(_) => { continue; } // opaque error. We can't properly handle it, since we don't know its severity.
			};

			let frame = match incoming_message.task {
				VideoTask::Encode(to) => { todo!("Have not implemented encoding within the actual worker thread yet!"); Err(Error::Custom("What the heck")) }
				VideoTask::Decode(from) => { todo!("Have not implemented decoding within the actual worker thread yet!"); Err(Error::Custom("What the heck")) }
				VideoTask::Transcode(from, to) => { self.internal_transcode(incoming_message, from, to) }
				VideoTask::ShutDown => { break }
				VideoTask::CV(frame_type) => { self.internal_cv(incoming_message, frame_type) }
			}; // match incoming_message
		} // loop
		self.logger.warn("Shutting down!")?;
		Ok(())
	} // work

	fn internal_transcode(&self, message : VideoTaskFull, from : FrameType, to : FrameType) -> Result<(), Error> {
		match (from, to)
		{
			(FrameType::TelloH264, FrameType::Png) => {
				let upgraded_h264 = raw_h264_to_rgb(message.image_data);
				match upgraded_h264 {
					Some(image) => {
						let mut png_buffer = Vec::new();
						let png_encoder = PngEncoder::new(&mut png_buffer);
						let (width, height) = image.dimensions();
						png_encoder.write_image(&*image, width, height, ExtendedColorType::Rgb8)?;
						self.sender.send_event(InternalSignal::FromVideoQueue((message.origin, png_buffer.into())))
							// FIXME: determine how we want to handle this.
							.unwrap_or(()); // If it fails to send, that's not big deal, for now...
					}
					None => { /* noop */ } // failed to transcode.
				}

			}
			_ => todo!("Not sure how to transcode this yet.")
		}
		Ok(())
	}

	fn internal_cv(&self, message : VideoTaskFull, frame_type: FrameType) -> Result<(), Error> {
		let (width, height, image) = match frame_type
		{
			FrameType::TelloH264 => { todo!("Haven't implemented CV from TelloH264") }
			FrameType::Png => { todo!("Haven't implemented CV from PNG") }
			FrameType::Rgb(width, height) => { (width, height, message.image_data) }
			FrameType::H264 => { todo!("Haven't implemented CV from H264") }
		};

		//self.model.run_model()?;

		Ok(())
	}
}