use image::EncodableLayout;
use crate::{CVBase, Coord3D};
use std::fmt::{Debug, Formatter};
use image::Rgb32FImage;
use tflitec::tensor::{Shape, Tensor};
use crate::{cv_base, ComputerVision};
use HandLandmarkIndices::Presence;
use crate::Error;
use crate::hand_landmarker::HandLandmarkIndices::{Handedness, ScreenSpace, WorldSpace};

pub struct HandLandmarker
{
	base : cv_base::CVBase<'static>,
}

const PRESENCE_THRESHOLD : f32 = 0.3;

#[derive(Debug)]
pub enum Hand
{
	Left,
	Right,
}

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


#[allow(dead_code)]
impl<'a> HandLandmarker
{
	pub fn from_path(model_path : &str) -> Result<Self, Error>
	{
		let input_shape = Shape::new(vec![Self::NUM_BATCHES, Self::WIDTH, Self::HEIGHT, Self::BIT_DEPTH]);
		let base = CVBase::from_path(model_path, input_shape)?;
		Ok(Self { base })
	}

	#[allow(dead_code)]
	pub fn from_bytes(buffer : &'static [u8]) -> Result<Self, Error>
	{
		let input_shape = Shape::new(vec![Self::NUM_BATCHES, Self::WIDTH, Self::HEIGHT, Self::BIT_DEPTH]);
		let base = CVBase::from_bytes(buffer, input_shape)?;
		Ok(Self { base })
	}

	/*pub fn from_shared_buffer(buffer : Arc<[u8]>)
	{
		Self::initialize_hand_landmarker_model(buffer)
	}*/

	// FIXME: This should not return the unit value.
	// 	I'll have to see if consuming the value is good or not.
	pub fn run_model(&mut self, input : Rgb32FImage) -> Result<Vec<Tensor<'_>>, Error>
	{
		debug_assert_eq!(input.as_bytes().len(), Self::NUM_BATCHES * Self::WIDTH * Self::HEIGHT * Self::BIT_DEPTH * size_of::<f32>(), "Image dimensions did not match expected size");

		let input_tensor = self.base.input(0)?;

		input_tensor.set_data(&input.into_vec())?;

		self.base.invoke()?;

		let output =
			{
				let num_outputs = self.base.output_tensor_count();
				let mut output = Vec::with_capacity(num_outputs); // I think there's only 4 values that can be returned here
				for i in 0..num_outputs
				{
					output.push(self.base.output(i)?);
				}

				output
			};

		Ok(output)
	}

	pub fn hand_present(tensor : &Vec<Tensor<'_>>) -> bool
	{
		tensor[Presence as usize].data::<f32>()[0] >= PRESENCE_THRESHOLD
	}

	pub fn hand_screen_coords(tensor : &Vec<Tensor<'_>>) -> [Coord3D;21]
	{
		let poi =  tensor[ScreenSpace as usize].data::<f32>();
		debug_assert_eq!(poi.len() % 3, 0);

		let mut result= [Coord3D {x : 0.0, y : 0.0, z : 0.0};21];
		let mut i = 0;
		for point in poi.chunks_exact(3)
		{
			result[i] = Coord3D {
				x: point[0],
				y: point[1],
				z: point[2],
			};
			i += 1;
		}

		result

	}

	pub fn hand_world_coords(tensor : &Vec<Tensor<'_>>) -> [Coord3D;21]
	{
		let poi =  tensor[WorldSpace as usize].data::<f32>();
		debug_assert_eq!(poi.len() % 3, 0);

		let mut result= [Coord3D {x : 0.0, y : 0.0, z : 0.0};21];
		let mut i = 0;
		for point in poi.chunks_exact(3)
		{
			result[i] = Coord3D {
				x: point[0],
				y: point[1],
				z: point[2],
			};
			i += 1;
		}

		result

	}

	pub fn handedness(tensor : &Vec<Tensor<'_>>) -> Hand
	{
		if tensor[Handedness as usize].data::<f32>()[0] <= 0.5 { Hand::Left } else { Hand::Right }
	}


}


impl Debug for HandLandmarker
{
	fn fmt(&self, _f: &mut Formatter<'_>) -> std::fmt::Result {
		dbg!("Debug is not implemented for HandLandmarker!");
		Ok(())
	}
}

impl ComputerVision for HandLandmarker
{
	const NUM_BATCHES: usize = 1;
	const WIDTH : usize = 224;
	const HEIGHT: usize = 224;
	const BIT_DEPTH : usize = 3;

	fn output(&self, idx: usize) -> Result<Tensor<'_>, Error> {
		Ok(self.base.output(idx)?)
	}

	fn invoke(&self) -> Result<(), Error> {
		Ok(self.base.invoke()?)
	}

	fn input(&self, idx: usize) -> Result<Tensor<'_>, Error> {
		Ok(self.base.input(idx)?)
	}
}