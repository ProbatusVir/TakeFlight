mod cv_base;
pub mod hand_landmarker;

pub use hand_landmarker::HandLandmarker;

pub mod hand_identifier;
pub mod geometry;

pub use anyhow::Error;
pub(crate) use cv_base::CVBase;
pub(crate) use tflitec::tensor::Tensor;


pub trait ComputerVision
{
	const NUM_BATCHES	: usize;
	const WIDTH			: usize;
	const HEIGHT		: usize;
	const BIT_DEPTH		: usize;

	fn output(&self, idx : usize) -> Result<Tensor<'_>, Error>;
	fn invoke(&self) -> Result<(), Error>;
	fn input(&self, idx : usize) -> Result<Tensor<'_>, Error>;

	fn width() -> usize { Self::WIDTH }
	fn height() -> usize { Self::HEIGHT }
}