mod cv_base;
pub mod hand_landmarker;

use anyhow::Error;
use cv_base::CVBase;
use tflitec::tensor::Tensor;
pub use hand_landmarker::HandLandmarker;

const PRESENCE_THRESHOLD : f32 = 0.3;

#[repr(usize)]
enum HandLandmarkIndices
{
	ScreenSpace	= 0,
	Presence	= 1,
	Handedness	= 2,
	WorldSpace	= 3,
}

#[allow(dead_code)]
// These acronyms are anatomical, and I lack better words for them
pub enum DigitIndices
{
	Wrist = 0,
	ThumbCMC,
	ThumbMCP,
	ThumbIP,
	ThumbTip,
	IndexFingerMCP = 5,
	IndexFingerPIP,
	IndexFingerDIP,
	IndexFingerTip,
	MiddleFingerMCP = 9,
	MiddleFingerPIP,
	MiddleFingerDIP,
	MiddleFingerTip,
	RingFingerMCP=13,
	RingFingerPIP,
	RingFingerDIP,
	RingFingerTIP,
	PinkyMCP = 17,
	PinkyPIP,
	PinkyDIP,
	PinkyTip,
}


pub trait ComputerVision
{
	const NUM_BATCHES	: usize;
	const WIDTH			: usize;
	const HEIGHT		: usize;
	const BIT_DEPTH		: usize;

	fn output(&self, idx : usize) -> Result<Tensor<'_>, Error>;
	fn invoke(&self) -> Result<(), Error>;
	fn input(&self, idx : usize) -> Result<Tensor<'_>, Error>;
}