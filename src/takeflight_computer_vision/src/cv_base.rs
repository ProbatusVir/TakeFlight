use std::marker::PhantomPinned;
use std::pin::Pin;
use crate::Error;
use tflitec::interpreter::{Interpreter, Options};
use tflitec::model::Model;
use tflitec::tensor::{Shape, Tensor};


pub(crate) struct CVBase<'a>
{
	model		: Model<'static>,			// Has the independent lifetime
	instance	: Option<Interpreter<'a>>,	// Instance should NEVER be touched.
	_pin		: PhantomPinned,			// This is just a compiler hint, it doesn't exist at runtime.
}

impl<'a> CVBase<'a>
{

	pub fn from_path(model_path : &str, input_shape : Shape) -> Result<Pin<Box<Self>>, Error>
	{
		let model = Model::new(model_path)?;

		Self::init(model, input_shape)
	}

	#[allow(dead_code)]
	pub fn from_bytes(buffer : &'static [u8], input_shape : Shape) -> Result<Pin<Box<Self>>, Error>
	{
		let model = Model::from_bytes(buffer)?;

		Self::init(model, input_shape)
	}

	fn init(model : Model<'static>, input_shape : Shape) -> Result<Pin<Box<Self>>, Error>
	{

		/*let result = CVBaseBuilder {
			model,
			instance_builder : move |model : &Model| {
				let instance = Interpreter::new(&model, Some(Options::default())).unwrap();
				instance.resize_input(0, input_shape).unwrap();
				instance.allocate_tensors().unwrap();
				instance
			},
		}.build();*/

		let mut result = Box::pin(Self {
			model,
			instance: None,
			_pin: PhantomPinned,
		});

		unsafe {
			let model_ptr = &result.model as *const Model;
			let mut_ref_to_self = result.as_mut();
			let inner = mut_ref_to_self.get_unchecked_mut();

			let instance = Interpreter::new(&*model_ptr, Some(Options::default())).unwrap();

			inner.instance = Some(instance)
		}

		let instance = result.borrow_instance()?;
		instance.resize_input(0, input_shape).unwrap();
		instance.allocate_tensors().unwrap();


		Ok(result)
	}

	fn borrow_instance(&self) -> Result<&Interpreter, Error> {
		match &self.instance {
			Some(instance) => { Ok(&instance) }
			None => { Err(Error::msg("There was no instance for this model to get."))? }
		}
	}

	pub fn input(&self, idx : usize) -> Result<Tensor<'_>, Error>
	{

		Ok(self.borrow_instance()?.input(idx)?)
	}

	pub fn invoke(&self) -> Result<(), Error>
	{
		Ok(self.borrow_instance()?.invoke()?)
	}

	pub fn output_tensor_count(&self) -> usize
	{
		// FIXME: This is not good.
		self.borrow_instance().unwrap().output_tensor_count()
	}

	pub fn output(&self, idx : usize) -> Result<Tensor<'_>, Error>
	{
		Ok(self.borrow_instance()?.output(idx)?)
	}

}
