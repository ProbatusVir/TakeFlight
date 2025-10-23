mod cv_base;
pub mod hand_landmarker;

pub use hand_landmarker::HandLandmarker;

#[cfg(test)]
mod tests;


pub(crate) use anyhow::Error;
pub(crate) use cv_base::CVBase;
pub(crate) use tflitec::tensor::Tensor;
pub(crate) use tflitec as tf;

#[derive(Copy, Clone)]
pub struct Coord3D
{
	x : f32,
	y : f32,
	z : f32,
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