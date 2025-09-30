use crate::tf::tensor::Shape;
use crate::tf::interpreter::Interpreter;
use crate::tf::interpreter::Options;
use crate::tf::model::Model;
use image::imageops::FilterType;
use image::{EncodableLayout, Rgb32FImage};
use rstest::{fixture, rstest};
use crate::Error;


#[fixture]
fn load_image_data() -> Result<Rgb32FImage, Error>
{
	// Load in test data
	let mut image = image::open("src/tests/test_data/open_palm.png")?;
	Ok(
		image.resize_exact(224, 224, FilterType::CatmullRom).into_rgb32f()
	)
}

// This looks super ugly. The reference of Model is gonna live equally as long as the model will, and the interpreter will also live equally as long.
//#[fixture]
fn setup_interpreter<'a>(model: &'a Model<'a>) -> Result<Interpreter<'a>, Error>
{
	let model_options = Some(Options::default()); // the only thing configurable here is thread count, let's defer to the library on this.
	let mut interpreter = Interpreter::new(&model, model_options)?;

	// Set up input
	//let input_shape = tf::tensor::Shape::new(vec![1, 192, 192, 3]);
	let mut input_shape = Shape::new(vec![1, 224, 224, 3]); // 1 image with dimension 224x224, with a 3 elements per pixel.
	interpreter.resize_input(0, input_shape)?;
	interpreter.allocate_tensors()?;

	Ok(interpreter)
}

#[rstest]
fn hand_detector_runs(load_image_data : Result<Rgb32FImage, Error>) -> Result<(), Error>
{
	// Arrange model and put the image in the input.
	let model = Model::new("src/model/hand_landmarks_detector.tflite")?;
	let instance = setup_interpreter(&model)?;
	let input_tensor = instance.input(0)?;
	let input_image = load_image_data?;
	input_tensor.set_data(&input_image.as_bytes())?;

	// Act
	instance.invoke()?;

	Ok(())
}

#[rstest]
fn hand_detector_output(load_image_data : Result<Rgb32FImage, Error>) -> Result<(), Error>
{
	// Arrange model and put the image in the input.
	let model = Model::new("src/model/hand_landmarks_detector.tflite")?;
	let instance = setup_interpreter(&model)?;
	let input_tensor = instance.input(0)?;
	let input_image = load_image_data?;
	input_tensor.set_data(&input_image.as_bytes())?;

	// Act
	instance.invoke()?;

	// Assert
	let output = instance.output(0)?;
	assert_eq!(*output.shape(), Shape::new(vec![1, 63]));

	Ok(())
}

#[rstest]
fn looksie_at_hand_data(load_image_data : Result<Rgb32FImage, Error>) -> Result<(), Error>
{
	// Arrange model and put the image in the input.
	let model = Model::new("src/model/hand_landmarks_detector.tflite")?;
	let instance = setup_interpreter(&model)?;
	let input_tensor = instance.input(0)?;
	let input_image = load_image_data?;
	input_tensor.set_data(&input_image.as_bytes())?;

	// Act
	instance.invoke()?;

	let num_outputs = instance.output_tensor_count();
	for i in 0..num_outputs
	{
		let output = instance.output(i)?;
		dbg!(output.shape());
	}

	Ok(())
}