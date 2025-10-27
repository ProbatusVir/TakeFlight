use image::EncodableLayout;
use crate::CVBase;
use std::fmt::{Debug, Formatter};
use std::path::Path;
use image::Rgb32FImage;
use tflitec::model::Model;
use tflitec::tensor::{Shape, Tensor};
use crate::{cv_base, ComputerVision};
use HandLandmarkIndices::Presence;
use crate::Error;
use crate::geometry::{Coord2D, Coord3D};
use crate::hand_landmarker::DigitIndices::{Index, Middle, Pinky, Ring, Thumb};
use crate::hand_landmarker::HandLandmarkIndices::{Handedness, ScreenSpace, WorldSpace};
use crate::hand_landmarker::PointIndices::{IndexFingerMCP, MiddleFingerMCP, PinkyMCP, RingFingerMCP, ThumbCMC};

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
pub enum PointIndices
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

pub enum DigitIndices
{
	Thumb,
	Index,
	Middle,
	Ring,
	Pinky,
}

#[allow(dead_code)]
impl<'a> HandLandmarker
{
	pub fn new() -> Result<Self, Error>
	{
		let input_shape = Shape::new(vec![Self::NUM_BATCHES, Self::WIDTH, Self::HEIGHT, Self::BIT_DEPTH]);
		let path = Path::new("model/hand_landmarks_detector.tflite");
		let base = CVBase::from_path(path.to_str().unwrap(), input_shape)?;
		Ok(Self { base })
	}


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

	/*pub fn hand_screen_coords(tensor : &Vec<Tensor<'_>>) -> [Coord3D;21]
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

	}*/

	pub fn hand_world_coords(tensor : &Vec<Tensor<'_>>) -> [Coord3D<f32>;21]
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


	/// Each finger has 4 points
	/// 0 is the radix
	/// 3 is the tip
	pub fn get_digits(tensors: &Vec<Tensor<'_>>) -> [[Coord3D<f32>;4];5]
	{
		// x, y, c
		let (raw_data, remainder) = tensors[ScreenSpace as usize].data::<f32>().as_chunks::<3>();
		debug_assert_eq!(remainder.len(), 0);
		debug_assert_eq!(raw_data.len(), 21);
		let sc: Vec<Coord3D<f32>> = raw_data.into_iter().map(|coord| { Coord3D { x: coord[0], y: coord[1], z: coord[2]} } ).collect();

		const P : usize = PinkyMCP as usize;
		const R : usize = RingFingerMCP as usize;
		const M : usize = MiddleFingerMCP as usize;
		const I : usize = IndexFingerMCP as usize;
		const T : usize = ThumbCMC as usize;


		// Don't wanna complicate things too much, so instead of using all the indices as they appear on the enum, we'll just use basic offsets...
		[
			[sc[T + 0], sc[T + 1], sc[T + 2], sc[T + 3]],
			[sc[I + 0], sc[I + 1], sc[I + 2], sc[I + 3]],
			[sc[M + 0], sc[M + 1], sc[M + 2], sc[M + 3]],
			[sc[R + 0], sc[R + 1], sc[R + 2], sc[R + 3]],
			[sc[P + 0], sc[P + 1], sc[P + 2], sc[P + 3]],
		]

	}

	/// Counts the fingers that aren't the thumb on a hand.
	pub fn digits_down(fingies : &[[Coord3D<f32>;4];5]) -> [bool;4]
	{
		let mut result = [false, false, false, false];
		for (idx, finger) in [fingies[Index as usize], fingies[Middle as usize], fingies[Ring as usize], fingies[Pinky as usize]].iter().enumerate()
		{
			result[idx] = finger[0].y > finger[3].y;
		}

		result
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