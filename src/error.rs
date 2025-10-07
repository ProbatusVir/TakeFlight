use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Error
{
	InitFail,
	IOError(std::io::Error),
	Custom(&'static str),
	LocalIPError,
	ImageError(image::ImageError),
	TFLiteC(tflitec::Error),
	H264Error(openh264::Error),
	Infallible(std::convert::Infallible)
}

impl From<std::convert::Infallible> for Error
{
	fn from(value: std::convert::Infallible) -> Self {
		Error::Infallible(value)
	}
}

impl From<openh264::Error> for Error
{
	fn from(value: openh264::Error) -> Self {
		Error::H264Error(value)
	}
}

impl From<tflitec::Error> for Error
{
	fn from(value: tflitec::Error) -> Self {
		Error::TFLiteC(value)
	}
}

impl From<std::io::Error> for Error
{
	fn from(value: std::io::Error) -> Self {
		Error::IOError(value)
	}
}

impl From<local_ip_address::Error> for Error
{
	fn from(_value: local_ip_address::Error) -> Self {
		Error::LocalIPError
	}
}

impl From<image::ImageError> for Error
{
	fn from(value: image::ImageError) -> Self {
		Error::ImageError(value)
	}
}


impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self
		{
			Error::InitFail => { "Failed to initialize drone!".fmt(f) }
			Error::IOError(e) => { e.fmt(f) }
			Error::Custom(msg) => { msg.fmt(f) }
			Error::LocalIPError => { "Failed to acquire local IP".fmt(f) }
			Error::ImageError(e) => { e.fmt(f) }
			Error::TFLiteC(e) => { e.fmt(f) }
			Error::H264Error(e) => { e.fmt(f) }
			Error::Infallible(e) => { e.fmt(f) }
		}
	}
}

impl std::error::Error for Error {}

