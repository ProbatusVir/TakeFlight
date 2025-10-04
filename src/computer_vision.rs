use image::{DynamicImage, EncodableLayout, Rgb32FImage};
use tflitec::interpreter;
use tflitec::interpreter::{Interpreter, Options};
use tflitec::model::Model;
use tflitec::tensor::{Shape, Tensor};
use crate::Error;

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

struct HandLandmarker<'a>
{
	model: Model<'a>,
	instance : Interpreter<'a>,
}

struct HandLandmarkerBuilder<'a>
{
	model : Option<Model<'a>>,
	interpreter : Option<Interpreter<'a>>,
}

const NUM_BATCHES: usize = 1;
const WIDTH : usize = 224;
const HEIGHT: usize = 224;
const BIT_DEPTH : usize = 3;


impl<'a> HandLandmarker<'a>
{
	pub fn new(model_path : &str) -> Result<Self, Error>
	{
		let model = Model::new(model_path)?;

		Self::initialize_hand_landmarker_model(model)
	}

	/*#[allow(dead_code)]
	pub fn from_bytes(buffer : &[u8]) -> Result<Self, Error>
	{
		let model = Model::from_bytes(buffer)?;

		Self::initialize_hand_landmarker_model(model)
	}*/

	// FIXME: This should not return the unit value.
	// 	I'll have to see if consuming the value is good or not.
	pub fn run_model(&mut self, input : Rgb32FImage) -> Result<Vec<Tensor>, Error>
	{
		debug_assert_eq!(input.as_bytes().len(), NUM_BATCHES * WIDTH * HEIGHT * BIT_DEPTH * size_of::<f32>(), "Image dimensions did not match expected size");
		let input_tensor = self.instance.input(0)?;
		input_tensor.set_data(&input.into_vec())?;

		self.instance.invoke()?;

		let output =
			{
				let num_outputs = self.instance.output_tensor_count();
				let mut output = Vec::with_capacity(num_outputs); // I think there's only 4 values that can be returned here
				for i in 0..num_outputs
				{
					output.push(self.instance.output(i)?);
				}

				output
			};

		Ok(output)
	}

	fn initialize_hand_landmarker_model(model : Model<'a>) -> Result<Self, Error>
	{
		let input_shape = Shape::new(vec![NUM_BATCHES, WIDTH, HEIGHT, BIT_DEPTH]);

		let mut builder = HandLandmarkerBuilder { model :  Some(model), interpreter : None};

		// Do instance thing
		{
			let instance = Interpreter::new(&builder.model, Some(Options::default()))?;
			instance.resize_input(0, input_shape)?;
			instance.allocate_tensors()?;

			builder.interpreter = Some(instance);
		}

		let new_landmarker = Self { model : builder.model, instance: builder.interpreter.unwrap() };
		/*new_landmarker.instance = Some(
			{
				let instance = Interpreter::new(&new_landmarker.model, Some(Options::default()))?;
				instance.resize_input(0, input_shape)?;
				instance.allocate_tensors()?;
			});
*/
		Ok(new_landmarker)
	}

}