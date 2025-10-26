mod shared;

use std::process::Output;
use rstest::rstest;
use takeflight_computer_vision::hand_identifier::HandIdentifier;
use shared::{load_image_data, OPEN_PALM, BLANK};
use takeflight_computer_vision::{Error, HandLandmarker};
use takeflight_computer_vision::hand_identifier::_IdentifierComponent::{Boxes, Confidences};
use crate::shared::TWO_HANDS;

const MODEL_PATH : &str = "model/hand_detector.tflite";


/// The first tensor is the data
///
/// first four:
///
/// dx:	offset from the center
///
/// dy:	offset from the center
///
/// w:	width across center of rectangle (the edges will be dx ± w/2)
///
/// h:	height across center of rectangle (the edges will be dy ± h/2)
///
/// There will be, additionally, 7 points of x,y coordinates: see [this](https://github.com/aashish2000/hand_tracking/blob/master/src/hand_tracker.py#L130) for more information
///
/// The second tensor are the probabilities, where >0.5 is good enough probability to be considered a candidate.

#[rstest]
fn identifier_run() -> Result<(), Error>
{
	// Arrange
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(OPEN_PALM)?;

	// Act / Assert
	let output = instance.run_model(input_image)?;

	Ok(())
}

#[rstest]
fn identifier_final_one_hand() -> Result<(), Error>
{
	// Arrange
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(OPEN_PALM)?;

	let output = instance.run_model(input_image)?;

	// Act
	let hands = HandIdentifier::get_hand_boxes(&output);

	// Assert
	assert_eq!(hands.len(), 1);

	Ok(())
}

#[rstest]
fn identifier_final_two_hands() -> Result<(), Error>
{
	// Arrange
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(TWO_HANDS)?;

	let output = instance.run_model(input_image)?;

	// Act
	let hands = HandIdentifier::get_hand_boxes(&output);

	// Assert
	assert_eq!(hands.len(), 2);

	Ok(())
}




/*#[rstest]
fn peek_identifier_0() -> Result<(), Error>
{
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(OPEN_PALM)?;

	let output = instance.run_model(input_image)?;
	assert_eq!(output[0].data::<f32>().len(), 2016 * 18);
	let mut i = 0;
	for bound_box in output[0].data::<f32>().chunks_exact(18)
	{
		if output[1].data::<f32>()[i] > 0.5
		{
			println!("dx: {}\tdy: {}\t w: {}\t h: {}, c: {}", bound_box[0], bound_box[1], bound_box[2], bound_box[3], output[1].data::<f32>()[i])
		}

		i += 1;
	}

	panic!()
}
#[rstest]
fn peek_identifier_1() -> Result<(), Error>
{
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(OPEN_PALM)?;

	let output = instance.run_model(input_image)?;
	assert_eq!(output[1].data::<f32>().len(), 2016);

	dbg!(output[1].data::<f32>());
	panic!()
}

#[rstest]
fn peek_culled() -> Result<(), Error>
{
	// Arrange
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(OPEN_PALM)?;

	let output = instance.run_model(input_image)?;
	assert_eq!(output[Boxes as usize].data::<f32>().len(), 2016 * 18);

	// Act
	let indices = HandIdentifier::_cull_hand_boxes_by_confidence(&output);

	// Assert
	let (chunks, remainder) = output[Boxes as usize].data::<f32>().as_chunks::<18>();
	assert_eq!(remainder.len(), 0); // It is not possible for this to be untrue, given our check up top.

	for idx in indices
	{
		let bound_box = chunks[idx];
		println!("dx: {}\tdy: {}\t w: {}\t h: {}, c: {}", bound_box[0], bound_box[1], bound_box[2], bound_box[3], output[Confidences as usize].data::<f32>()[idx])
	}

	panic!()
}

#[rstest]
fn peek_sorted() -> Result<(), Error>
{
	// Arrange
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(OPEN_PALM)?;

	let output = instance.run_model(input_image)?;
	assert_eq!(output[Boxes as usize].data::<f32>().len(), 2016 * 18);
	let mut indices = HandIdentifier::_cull_hand_boxes_by_confidence(&output);

	// Act
	HandIdentifier::_sort_hand_boxes(&output, &mut indices);

	// Assert
	let (chunks, remainder) = output[Boxes as usize].data::<f32>().as_chunks::<18>();
	assert_eq!(remainder.len(), 0); // It is not possible for this to be untrue, given our check up top.

	for idx in indices
	{
		let bound_box = chunks[idx];
		println!("dx: {}\tdy: {}\t w: {}\t h: {}, c: {}", bound_box[0], bound_box[1], bound_box[2], bound_box[3], output[Confidences as usize].data::<f32>()[idx])
	}

	panic!()
}

#[rstest]
fn peek_final_one_hand() -> Result<(), Error>
{
	// Arrange
	let mut instance = HandIdentifier::from_path(MODEL_PATH)?;
	let input_image = load_image_data::<_, HandIdentifier>(OPEN_PALM)?;

	let output = instance.run_model(input_image)?;
	assert_eq!(output[Boxes as usize].data::<f32>().len(), 2016 * 18);
	let mut indices = HandIdentifier::_cull_hand_boxes_by_confidence(&output);
	HandIdentifier::_sort_hand_boxes(&output, &mut indices);

	// Act
	HandIdentifier::_cull_hand_boxes_by_iou(&output, &mut indices);

	// Assert
	let (chunks, remainder) = output[Boxes as usize].data::<f32>().as_chunks::<18>();
	assert_eq!(remainder.len(), 0); // It is not possible for this to be untrue, given our check up top.

	for idx in indices
	{
		let bound_box = chunks[idx];
		println!("dx: {}\tdy: {}\t w: {}\t h: {}, c: {}", bound_box[0], bound_box[1], bound_box[2], bound_box[3], output[Confidences as usize].data::<f32>()[idx])
	}

	panic!()
}*/