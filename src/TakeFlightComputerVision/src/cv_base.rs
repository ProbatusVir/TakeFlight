use tflitec::interpreter::{Interpreter, Options};
use tflitec::model::Model;
use ouroboros;
use tflitec::tensor::{Shape, Tensor};
use crate::Error;

#[ouroboros::self_referencing]
pub(crate) struct CVBase<'a>
{
	model		: Model<'a>,
	#[borrows(model)]
	#[covariant]
	instance	: Interpreter<'this>,
}

impl<'a> CVBase<'a>
{

	pub fn from_path(model_path : &str, input_shape : Shape) -> Result<Self, Error>
	{
		let model = Model::new(model_path)?;

		Self::init(model, input_shape)
	}

	#[allow(dead_code)]
	pub fn from_bytes(buffer : &'static [u8], input_shape : Shape) -> Result<Self, Error>
	{
		let model = Model::from_bytes(buffer)?;

		Self::init(model, input_shape)
	}

	fn init(model : Model<'a>, input_shape : Shape) -> Result<Self, Error>
	{

		let result = CVBaseBuilder {
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

	pub fn input(&self, idx : usize) -> Result<Tensor<'_>, Error>
	{
		Ok(self.borrow_instance().input(idx)?)
	}

	pub fn invoke(&self) -> Result<(), Error>
	{
		Ok(self.borrow_instance().invoke()?)
	}

	pub fn output_tensor_count(&self) -> usize
	{
		self.borrow_instance().output_tensor_count()
	}

	pub fn output(&self, idx : usize) -> Result<Tensor<'_>, Error>
	{
		Ok(self.borrow_instance().output(idx)?)
	}

}
