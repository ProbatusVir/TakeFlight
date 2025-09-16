use std::fmt;
use std::fmt::{Display, Formatter};
use crate::error::Error::{IOError, LocalIPError};

#[derive(Debug)]
pub enum Error
{
	InitFail,
	IOError(std::io::Error),
	Custom(&'static str),
	LocalIPError,
}

impl From<std::io::Error> for Error
{
	fn from(value: std::io::Error) -> Self {
		IOError(value)
	}
}

impl From<local_ip_address::Error> for Error
{
	fn from(_value: local_ip_address::Error) -> Self {
		LocalIPError
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self
		{
			Error::InitFail => { "Failed to initialize drone!".fmt(f) }
			Error::IOError(e) => { e.fmt(f) }
			Error::Custom(msg) => { msg.fmt(f) }
			LocalIPError => { "Failed to acquire local IP".fmt(f) }
		}
	}
}

impl std::error::Error for Error {}

