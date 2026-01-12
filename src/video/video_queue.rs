use std::default::Default;
use std::sync::mpsc::TryRecvError;
use std::option::Option;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ffi::c_void;
use crate::logger::Logger;
use crate::video::decode::raw_h264_to_rgb;
use crate::{Error, InternalSignal};
use image::codecs::png::PngEncoder;
use image::{DynamicImage, ExtendedColorType, ImageBuffer, ImageEncoder, Rgb, Rgb32FImage, RgbImage};
use mio::{ Token };
use mio_wakeq::WakeQSender;
use std::fmt::Debug;
use std::sync::{Arc, Mutex};
use std::thread;
use image::imageops::CatmullRom;
use takeflight_computer_vision as tfcv;
use takeflight_computer_vision::{ComputerVision, HandLandmarker};

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

// TODO: an option Vec<u8> would be really cool for all the image allocations, we can make it into an imagebuffer, and go between rgb8 and rgbf32 all for the price of rgbf32 if we can guarantee that we'll put it back right.
struct VideoQueueThreadInfo
{
	receiver		: std::sync::mpsc::Receiver<VideoTaskFull>,
	sender			: WakeQSender<InternalSignal>,
	curr_src		: Arc<Mutex<Option<Token>>>,
	logger			: Logger,
	model			: tfcv::hand_landmarker::HandLandmarker,

	/// I don't like how memory expensive HS is, but it does have speed + more efficient allocation scheme.
	///
	/// The token will be the origin, and the usize will be index from _chrono_stack.
	_sorting_set	: HashSet<Token>,
	/// The events as they came in. Most recent events are at the end, logically.
	_chrono_stack	: VecDeque<VideoTaskFull>,
	/// The top of _chrono_stack is at the bottom of the _sorted_dequeue,
	/// consequently, when processing this structure, we want to start at
	/// the front.
	_sorted_dequeue	: VecDeque<VideoTaskFull>,
	/// This keeps track of who currently needs attention in the batch.
	_batch_set: HashSet<Token>
}

type TryInsertionError = ();


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

	pub fn shutdown(&self) -> Result<(), Error> {
		let vtf = VideoTaskFull { task: VideoTask::ShutDown, image_data: Box::new([]), origin : Token(0) };
		self.sender.send(vtf).map_err(|_| { Error::Custom("Failed to send shutdown to the video stream thread. Did thread crash?") })
	}

	pub fn computer_vision(&self, origin : Token, curr_src : Option<Token>, frame_type : FrameType, image_data : Box<[u8]>) -> Result<(), Error> {
		self.send_to_queue(origin, curr_src, image_data, VideoTask::CV(frame_type))
	}

	/// Start the work thread
	/// May incorporate the logger.
	pub fn start_work_thread(curr_src : Arc<Mutex<Option<Token>>>, logger: Logger, internal_signaller : WakeQSender<InternalSignal>) -> Result<(Self, thread::JoinHandle<Result<(), Error>>), Error>
	{
		let (queue, queue_receiver) = Self::new();

		let thread_handle = std::thread::Builder::new()
			.name("Video".into())
			.spawn(|| {
					let mut thread_stuff = VideoQueueThreadInfo {
						receiver: queue_receiver,
						sender: internal_signaller,
						curr_src,
						logger,
						model : HandLandmarker::new()?,
						_sorting_set: Default::default(),
						_chrono_stack: Default::default(),
						_sorted_dequeue: Default::default(),
						_batch_set: Default::default(),
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
	fn do_work(&mut self) -> Result<(), Error>
	{
		self.logger.info("Starting working queue!")?;

		loop {
			// get networking events
			self._sorting_set.clear();
			self.drain_held_events_into_unsorted_buffer();
			self.get_all_current_events()?;
			
			if self._batch_set.is_empty()
			{
				self.fill_batch_set();
			}

			// dispatch the incoming message.
			match self._sorted_dequeue.pop_front() {
				Some(message) => { 
					match self.dispatch_incoming_message(message) {
						Ok(()) => {} // Literally nothing to do
						Err(Error::Shutdown) => { break }
						Err(e) => { Err(e)? }
					}
				}
				None => { todo!("This is an unreachable arm.") }
			};


		} // loop

		self.logger.warn("Shutting down!")?;
		Ok(())
	} // work
	
	fn fill_batch_set(&mut self) {
		
		for item_of_unique_origin in &self._sorted_dequeue
		{
			debug_assert!(!self._batch_set.contains(&item_of_unique_origin.origin));
			self._batch_set.insert(item_of_unique_origin.origin);
		}
	}

	fn dispatch_incoming_message(&mut self, incoming_message : VideoTaskFull) -> Result<(), Error>
	{
		match incoming_message.task {
			VideoTask::Encode(to) => {
				todo!("Have not implemented encoding within the actual worker thread yet!");
				Err(Error::Custom("What the heck"))
			}
			VideoTask::Decode(from) => {
				todo!("Have not implemented decoding within the actual worker thread yet!");
				Err(Error::Custom("What the heck"))
			}
			VideoTask::Transcode(from, to) => { self.internal_transcode(incoming_message, from, to) }
			VideoTask::ShutDown => { Err(Error::Shutdown) }
			VideoTask::CV(frame_type) => { self.internal_cv(incoming_message, frame_type) }
		} // match incoming_message.task
	}

	/// This should retain ordering of the initial set, which means pop_front -> push_front
	///
	/// When re-sorted, these events will have the lowest priority.
	fn drain_held_events_into_unsorted_buffer(&mut self) {
		loop // !self._sorted_dequeue.is_empty()
		{
			let element = match self._sorted_dequeue.pop_front() {
				Some(element) => { element }
				None => { break }
			};

			self._chrono_stack.push_front(element);
		} // loop
	}

	fn get_all_current_events(&mut self) -> Result<(), Error>
	{
		// do a blocking read here to avoid busy cycles.
		if self._sorted_dequeue.is_empty()
		{
			let first_message = self.blocking_try_to_get_new_message()?;
			self.handle_incoming_message(first_message);
		}

		let mut video_task : Option<VideoTaskFull>; // assigned before it's ever read.
		while {
			video_task = self.try_to_get_new_message()?;
			video_task.is_some()
		} {
			match video_task {
				Some(inner_task) => { self.handle_incoming_message(inner_task) }
				None => { self.logger.error("Unreachable execution path discovered in VideoQueueThreadInfo::get_all_current_events")? } // this should be an unreachable arm.
			}
		}

		self.cull_work_events();

		Ok(())
	}
	
	fn handle_incoming_message(&mut self, task : VideoTaskFull)
	{
		self._chrono_stack.push_back(task);
	}
	

	/// This isn't very scalable.
	/// TODO: Needs a lot of work.
	/// TODO: Find out if we need to clear the _sorting_set at the start.
	fn cull_work_events(&mut self) {

		while !self._chrono_stack.is_empty() {
			let last_task = self._chrono_stack.pop_back();
			match last_task {
				// we'll check if the source has emitted a more recent
				// event; If it has, then we do nothing (discard this item)
				Some(task) => {
					// make it known that this exists in the other set.
					let task_origin = task.origin;
					self.try_insert_to_sorted_stack(task).unwrap_or_default();
					self._sorting_set.insert(task_origin);
				} // Some(task)
				None => { todo!("This is an unreachable branch.") }
			}// match last_task
		} // while !is_empty()
	} // fn cull_work_events

	fn try_insert_to_sorted_stack(&mut self, task : VideoTaskFull) -> Result<(), TryInsertionError> {
		// If it contains the source, do nothing
		// otherwise, insert the source
		if !self._sorting_set.contains(&task.origin) {
			self._sorted_dequeue.push_back(task);
			Ok(())
		} else {
			Err(())
		}
	}

	fn try_to_get_new_message(&self) -> Result<Option<VideoTaskFull>, Error>
	{
		match self.receiver.try_recv()
		{
			Ok(message) => { Ok(Some(message)) }
			Err(e) => {
				match e
				{
					TryRecvError::Empty => { Ok(None) }
					TryRecvError::Disconnected => { Err(e)? }
				}
			}
		}
	}

	fn blocking_try_to_get_new_message(&self) -> Result<VideoTaskFull, Error>
	{
		match self.receiver.recv() {
			Ok(message) => {
				Ok(message)
			}
			Err(e) => {
				Err(TryRecvError::Disconnected)?
			}
		}
	}


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

	fn internal_cv(&mut self, message : VideoTaskFull, frame_type: FrameType) -> Result<(), Error> {
		let (width, height, image) = match frame_type
		{
			FrameType::TelloH264 => { todo!("Haven't implemented CV from TelloH264") }
			FrameType::Png => { todo!("Haven't implemented CV from PNG") }
			FrameType::Rgb(width, height) => { (width, height, message.image_data) }
			FrameType::H264 => { todo!("Haven't implemented CV from H264") }
		};

		//RgbImage and ImageBuffer<Rgb<u8>, Vec<u8>> are equivalent.
		let mut image =
			match RgbImage::from_raw(width, height, image.into_vec()) { // why is there literally no documentation on Box<T> into_vec method???
				None => { self.logger.error("Invalid image given to cv.")?; return Ok(()) } // maybe I should add an error type for this...
				Some(image) => { DynamicImage::ImageRgb8(image).into_rgb32f() }
			};
		// FIXME: we're gonna make the scaling its own dang function!
		// FIXME: implement more proper logic
		image = DynamicImage::ImageRgb32F(image).resize_exact(HandLandmarker::WIDTH as u32, HandLandmarker::HEIGHT as u32, CatmullRom).into_rgb32f();

		let output = self.model.run_model(image)?;
		let digits = HandLandmarker::get_digits(&output);
		let digits_down_array = HandLandmarker::digits_down(&digits);
		let mut num_digits_down = 0;
		// hideous one-liner for counting all the digits.
		digits_down_array.iter().for_each(|digit| { if *digit { num_digits_down += 1 } });



		Ok(())
	}
}