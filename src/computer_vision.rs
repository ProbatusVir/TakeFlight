use crate::Error;
use image::{EncodableLayout, Rgb32FImage};
use tflitec::interpreter::{Interpreter, Options};
use tflitec::model::Model;
use tflitec::tensor::{Shape, Tensor};

// These acronyms are anatomical, and I lack better words for them
enum DigitIndices
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

const NUM_BATCHES: usize = 1;
const WIDTH : usize = 224;
const HEIGHT: usize = 224;
const BIT_DEPTH : usize = 3;


#[ouroboros::self_referencing]
pub struct HandLandmarker<'a>
{
	model		: Model<'a>,
	#[borrows(model)]
	#[covariant]
	instance	: Interpreter<'this>,
}

//An implementation function with lifetimes that finds the bytes and path of the model(hand)
impl<'a> HandLandmarker<'a>
{
	pub fn from_path(model_path : &str) -> Result<Self, Error>
	{
		let model = Model::new(model_path)?;

		Self::initialize_hand_landmarker_model(model)
	}

	#[allow(dead_code)]
	pub fn from_bytes(buffer : &'a [u8]) -> Result<Self, Error>
	{
		let model = Model::from_bytes(buffer)?;

		Self::initialize_hand_landmarker_model(model)
	}

	/*pub fn from_shared_buffer(buffer : Arc<[u8]>)
	{
		Self::initialize_hand_landmarker_model(buffer)
	}*/

	// FIXME: This should not return the unit value.
	// 	I'll have to see if consuming the value is good or not.
	pub fn run_model(&mut self, input : Rgb32FImage) -> Result<Vec<Tensor<'_>>, Error>
	{
		//checking the image constraints do not go out of expected dimensions
		debug_assert_eq!(input.as_bytes().len(), NUM_BATCHES * WIDTH * HEIGHT * BIT_DEPTH * size_of::<f32>(), "Image dimensions did not match expected size");

		let instance = self.borrow_instance();

		let input_tensor = instance.input(0)?;
		input_tensor.set_data(&input.into_vec())?;

		instance.invoke()?;

		let output =
			{
				let num_outputs = instance.output_tensor_count();
				let mut output = Vec::with_capacity(num_outputs); // I think there's only 4 values that can be returned here
				for i in 0..num_outputs
				{
					output.push(instance.output(i)?);
				}

				output
			};

		Ok(output)
	}
//Creating the hand marker image
	fn initialize_hand_landmarker_model(model : Model<'a>) -> Result<Self, Error>
	{
		//shape of our currently predefined image size
		let input_shape = Shape::new(vec![NUM_BATCHES, WIDTH, HEIGHT, BIT_DEPTH]);
		//Creation of interpreter using the model, then resizing it and allocating tensors once created
		let result = HandLandmarkerBuilder {
			model,
			instance_builder : move |model : &Model| {
				let instance = Interpreter::new(&model, Some(Options::default())).unwrap();
				instance.resize_input(0, input_shape).unwrap();
				instance.allocate_tensors().unwrap();
				instance
			},
		}.build();

		Ok(result)
	}

}
