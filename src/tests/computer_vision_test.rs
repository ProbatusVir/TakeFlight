use crate::computer_vision::HandLandmarker as Landmarker;
use crate::tests::computer_vision_test::HandLandmarkIndices::Handedness;
use crate::tests::computer_vision_test::HandLandmarkIndices::Presence;
use crate::tests::get_mut_pixel;
use crate::tf::interpreter::Interpreter;
use crate::tf::interpreter::Options;
use crate::tf::model::Model;
use crate::tf::tensor::Shape;
use crate::Error;
use image::imageops::{CatmullRom, FilterType};
use image::{EncodableLayout, Rgb32FImage};
use rstest::{fixture, rstest};
use std::fs::File;
use std::io::Write;

#[repr(usize)]
enum HandLandmarkIndices
{
	ScreenSpace	= 0,
	Presence	= 1,
	Handedness	= 2,
	WorldSpace	= 3,
}

const PRESENCE_THRESHOLD : f32 = 0.5;

fn load_image_data(path : &str) -> Result<Rgb32FImage, Error>
{
	// Load in test data
	let mut image = image::open(path)?;
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
fn hand_detector_runs() -> Result<(), Error>
{
	let input_image = load_image_data("src/tests/test_data/open_palm.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;
	let _output = instance.run_model(input_image)?;

	Ok(())
}

#[rstest]
fn hand_detector_output() -> Result<(), Error>
{
	// Arrange model and put the image in the input.
	let input_image = load_image_data("src/tests/test_data/open_palm.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;

	// Act
	let _output = instance.run_model(input_image)?;
	let output = &_output[0];

	// Assert
	assert_eq!(*output.shape(), Shape::new(vec![1, 63]));

	Ok(())
}

#[rstest]
fn peek_at_hand_data() -> Result<(), Error>
{
	let input_image = load_image_data("src/tests/test_data/open_palm.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;

	// Act
	let output = instance.run_model(input_image)?;
	output.into_iter().for_each(|x| { dbg!(x); });

	Ok(())
}

#[rstest]
fn peek_two_hands_data() -> Result<(), Error>
{
	// Arrange
	let input_image = load_image_data("src/tests/test_data/open_palm.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;

	// Act
	let output = instance.run_model(input_image)?;
	let num_outputs = output.len();

	// Assert (and peek)
	assert_eq!(8, num_outputs);
	output.into_iter().for_each(|x| { dbg!(x); });

	Ok(())
}

// This test requires manual review
#[rstest]
fn color_based_on_index() -> Result<(), Error>
{
	// Arrange model and put the image in the input.
	let input_image = load_image_data("src/tests/test_data/open_palm.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;

	// Act
	let output = instance.run_model(input_image)?;

	// Act -- color the hands
	let screen_coordinates = &output[0];

	let mut output_image = image::open("src/tests/test_data/open_palm.png")?.resize_exact(224, 224, CatmullRom).into_rgb8();

	for point in screen_coordinates.data::<f32>().chunks_exact(3)
	{
		// Values are 0-244 for x & y, and just all real numbers for z???
		let x = point[0] as usize;
		let y = point[1] as usize;
		let _z = point[2];
		let pixel = get_mut_pixel(&mut output_image, x, y, 224, 3);
		pixel.swap_with_slice(&mut [0, 0, 0]);
	}

	let mut file = File::create("test_results/colored_hand.f32rgb")?;
	file.write_all(&output_image.as_bytes())?;


	Ok(())
}

#[rstest]
fn determine_handedness() -> Result<(), Error>
{
	// Arrange
	let input_image = load_image_data("src/tests/test_data/open_palm.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;

	// Act
	let _output = instance.run_model(input_image)?;
	let output = &_output[Handedness as usize];

	dbg!(output.data::<f32>()[0]);

	Ok(())
}

#[rstest]
//#[case("src/tests/test_data/blank.png", false)]
//#[case("src/tests/test_data/open_palm.png", true)]
fn determine_hand_presence_false() -> Result<(), Error>
{

	// Arrange
	let input_image = load_image_data("src/tests/test_data/blank.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;

	// Act
	let _output = instance.run_model(input_image)?;
	let output = &_output[Presence as usize];

	assert_eq!(PRESENCE_THRESHOLD < output.data::<f32>()[0], false);

	Ok(())
}

#[rstest]
fn determine_hand_presence_true() -> Result<(), Error>
{
	// Arrange
	let input_image = load_image_data("src/tests/test_data/open_palm.png")?;
	let mut instance = Landmarker::from_path("src/model/hand_landmarks_detector.tflite")?;

	// Act
	let _output = instance.run_model(input_image)?;
	let output = &_output[Presence as usize];

	assert_eq!(PRESENCE_THRESHOLD > output.data::<f32>()[0], false);

	Ok(())
}