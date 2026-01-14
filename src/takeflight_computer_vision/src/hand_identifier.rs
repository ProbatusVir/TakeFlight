use std::pin::Pin;
use crate::geometry::{BoundBox, IndexBoundBox};
use crate::hand_identifier::_IdentifierComponent::{Boxes, Confidences};
use crate::{CVBase, ComputerVision};
use anyhow::Error;
use image::{EncodableLayout, Rgb32FImage};
use itertools::Itertools;
use tflitec::tensor::{Shape, Tensor};

pub struct HandIdentifier
{
	base : Pin<Box<CVBase<'static>>>
}

pub enum _IdentifierComponent
{
	Boxes,
	Confidences,
}

impl HandIdentifier
{
	pub const PRESENCE_THRESHOLD : f32 = 0.5;
	pub const MAX_BOXES : usize = 2016;

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

	/// Right now this returns the values specifically for a 192x192, it may make sense in
	/// the future to normalize the coordinates from 0-1.
	pub fn get_hand_boxes(tensors : &[Tensor<'_>]) -> Vec<BoundBox<f32>>
	{

		let mut candidates = Self::_cull_hand_boxes_by_confidence(tensors);
		Self::_sort_hand_boxes(tensors, &mut candidates);
		Self::_cull_hand_boxes_by_iou(tensors, &mut candidates);
		/*
		// 18 is the number of floats in a detection -- we'll have a better definition later...
		for detection in detections.chunks_exact(18)
		{
			let x = detection[0];
			let y = detection[1];
			let w = detection[2];
			let h = detection[3];

			candidates.push(BoundBox { x, y, w, h});
		}
		*/

		let (raw_data, remaining) = tensors[Boxes as usize].data::<f32>().as_chunks::<18>();
		debug_assert_eq!(remaining.len(), 0);

		candidates.iter().map(|idx| {
			let raw_box = raw_data[*idx];
			BoundBox { x : raw_box[0], y : raw_box[1], w : raw_box[2], h : raw_box[3]}
		}).collect()

	}

	// TODO: Find a way to optimize the HandIdentifier internal algorithms...

	/// This assumes that values such as NaN are not included in the range of values.
	///
	/// Do this after culling!
	///
	/// These values are sorted from least likely to most likely
	pub fn _sort_hand_boxes(tensors : &[Tensor<'_>], valid_indices : &mut Vec<usize>)
	{
		let confidences = tensors[Confidences as usize].data::<f32>();
		valid_indices.sort_by(|left : &usize, right : &usize| { confidences[*left].total_cmp(&confidences[*right]) } );
		valid_indices.reverse();
	}

	/// Returns the indices of all eligible hand boxes
	pub fn _cull_hand_boxes_by_confidence(tensors : &[Tensor<'_>]) -> Vec<usize>
	{
		let confidences = tensors[Confidences as usize].data::<f32>();
		let mut indices = Vec::new();
		// only add what meets our criteria to the output, looks reasonable to me!
		for (idx, c) in confidences.iter().enumerate()
		{
			if *c > Self::PRESENCE_THRESHOLD
			{
				indices.push(idx);
			}
		}

		indices
	}

	pub fn _cull_hand_boxes_by_iou(tensors : &[Tensor<'_>], indices : &mut Vec<usize>)
	{
		let (chunks, remainder) = tensors[Boxes as usize].data::<f32>().as_chunks::<18>();
		debug_assert_eq!(remainder.len(), 0);

		// Initialize all the bounding boxes!
		let mut boxes = Vec::new();
		{
			for idx in indices.iter()
			{
				let raw = chunks[*idx];
				let bound_box = BoundBox::<f32> { x: raw[0], y: raw[1], w: raw[2], h: raw[3] };
				boxes.push(IndexBoundBox { bound_box, idx : *idx }); // This looks awful, but the first elements are compared when comparing tuples -- and since we want the boxes compared, it must be this way...
			}
		}




		let deduplicated = boxes.iter().dedup().collect::<Vec<_>>();
		indices.clear();
		for indexed_box in deduplicated
		{
			indices.push(indexed_box.idx);
		}
	}


}


impl ComputerVision for HandIdentifier
{
	const NUM_BATCHES: usize = 1;
	const WIDTH : usize = 192;
	const HEIGHT: usize = 192;
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

