use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc};
use std::thread::{JoinHandle};
use nokhwa::NokhwaError;
use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{CameraIndex, RequestedFormat};
use nokhwa::utils::RequestedFormatType::AbsoluteHighestFrameRate;
use crate::logger::Logger;
use crate::Error;

pub struct Camera
{
	camera	: nokhwa::Camera,
	buffer	: Vec<u8>,
}

impl Camera
{
	pub fn new() -> Result<Self, NokhwaError>
	{
		let format = RequestedFormat::new::<RgbFormat>(AbsoluteHighestFrameRate);
		let mut camera = nokhwa::Camera::new(CameraIndex::Index(0), format)?; // just get whatever camera is available.
		camera.open_stream()?;
		let buffer = Vec::new();

		Ok(Self
		{
			buffer,
			camera,
		})
	}

	pub fn get_rgb(&mut self) -> Result<&[u8], NokhwaError>
	{
		let resolution = self.camera.resolution();
		let total_size = (resolution.width_x * resolution.height_y * 3) as usize;
		self.buffer.resize(total_size, 0); // make sure that the buffer is always just as long as the data. No deallocation. No unnecessary writes.

		self.camera.write_frame_to_buffer::<RgbFormat>(&mut self.buffer)?;

		Ok(&self.buffer)
	}

	pub fn buffer(&self) -> &[u8]
	{
		&self.buffer
	}

	pub fn resolution(&self) -> (u32, u32)
	{
		let resolution = self.camera.resolution();
		(resolution.width(), resolution.height())
	}
}

struct CameraThreadInfo {
	take_pictures		: Arc<AtomicBool>,
	continue_running	: Arc<AtomicBool>,
	logger				: Logger,
}

pub(crate) struct CameraThread
{
	_thread	: Arc<JoinHandle<Result<(), Error>>>,
	info	: Arc<CameraThreadInfo>,
}

impl CameraThread {
	pub fn spawn(logger : Logger, take_pictures : Arc<AtomicBool>, continue_running : Arc<AtomicBool>) -> Result<Arc<Self>, Error>
	{
		let info = Arc::new(CameraThreadInfo {
			take_pictures,
			continue_running,
			logger,
		});
		let info_clone = info.clone();
		let _thread = Arc::new(std::thread::Builder::new()
			.name("Camera".into())
			.spawn(move || { info_clone.do_work() })?);

		Ok(Arc::new(Self {
			info,
			_thread,
		}))
	}
}

impl CameraThreadInfo {
	/// When killing this thread, you may need to set take_pictures notify this thread
	///
	/// # Arguments
	///
	/// * `logger`: Logger
	/// * `take_pictures`: A(n atomic) boolean for the server to indicate that pictures should be taken.
	/// * `continue_running`: A(n atomic) boolean meant to clean up this thread.
	///
	/// returns: A crate error if the camera fails.
	///
	/// # Examples
	///
	/// ```
	///
	/// ```
	pub(crate) fn do_work(&self) -> Result<(), Error>
	{
		const FRAMERATE: usize = 30;
		// create the camera
		self.logger.info("Starting camera thread!")?;

		// FIXME:	Good for identifying an error, but not for actually recovering the error...
		// 			Maybe the thread should implement clone where it clones the previous'
		//			`Arcs`, but spawns a new thread??? That requires the server to think a lil'
		//			harder, but might pay off in the long run. Thread failure will not
		//			invalidate these arcs, since they live longer than the main loop, so it's
		//			always an option...
		//			We'll figure out a good way to do this eventually.
		let mut camera = match Camera::new()
		{
			Ok(camera) => {camera}
			Err(e) => { self.logger.error_from_string(format!("Could not instantiate camera. {e:?}"))?; Err(e)?}
		};

		let (width, height) = camera.resolution();
		self.logger.info("Camera started!")?;

		// We don't need strong guarantees on order-of-operations.
		while self.continue_running.load(Relaxed) {
			let (w, h) = camera.resolution();
			let image = camera.get_rgb()?;

		} // while continue_running

		Ok(())
	} // do_work
} // impl CameraThreadInfo