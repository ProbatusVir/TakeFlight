use std::fmt;
use std::fmt::{Display, Formatter};

pub type Result<T> = core::result::Result<T, Error>;
#[derive(Debug)]
pub enum Error
{
	IOError(std::io::Error),
	Custom(&'static str),								// 16 bytes
	LocalIPError,
	ImageError(Box<image::ImageError>),					// 64 bytes
	H264Error(Box<openh264::Error>),					// 88 bytes
	Infallible(std::convert::Infallible),
	MutexError,
	PoisonError,
	AddrParseError(std::net::AddrParseError),
	RTPTypeNotImplemented(u8),
	AnyhowError(anyhow::Error),
	NoVideoTarget,
	NoVideoSource,
	ParseIntError(std::num::ParseIntError),
	SqliteError(Box<rusqlite::Error>),					// 64 bytes
	SerdeJSON(serde_json::Error),
	FromUtf8Error(Box<std::string::FromUtf8Error>),		// 40 bytes
	SystemTimeError(Box<std::time::SystemTimeError>),	// 16 bytes
	TryFromSliceError(std::array::TryFromSliceError),
	NokhwaError(Box<nokhwa::NokhwaError>),				// 72 bytes
	TryRecvError(std::sync::mpsc::TryRecvError),
}

impl From<std::sync::mpsc::TryRecvError> for Error
{
	fn from(value: std::sync::mpsc::TryRecvError) -> Self { Error::TryRecvError(value) }
}

impl From<std::array::TryFromSliceError> for Error
{
	fn from(value: std::array::TryFromSliceError) -> Self { Error::TryFromSliceError(value) }
}

impl From<std::time::SystemTimeError> for Error
{
	fn from(value: std::time::SystemTimeError) -> Self { Error::SystemTimeError(Box::new(value)) }
}


impl From<std::string::FromUtf8Error> for Error
{
	fn from(value: std::string::FromUtf8Error) -> Self { Error::FromUtf8Error(Box::new(value)) }
}

impl From<serde_json::Error> for Error
{
	fn from(value: serde_json::Error) -> Self { Error::SerdeJSON(value) }
}

impl From<rusqlite::Error> for Error
{
	fn from(value: rusqlite::Error) -> Self { Error::SqliteError(Box::new(value)) }
}

impl From<std::num::ParseIntError> for Error
{
	fn from(value: std::num::ParseIntError) -> Self { Error::ParseIntError(value) }
}


impl From<anyhow::Error> for Error
{
	fn from(value: anyhow::Error) -> Self {
		Error::AnyhowError(value)
	}
}

impl From<openh264::Error> for Error
{
	fn from(value: openh264::Error) -> Self {
		Error::H264Error(Box::new(value))
	}
}

impl From<std::convert::Infallible> for Error
{
	fn from(value: std::convert::Infallible) -> Self {
		Error::Infallible(value)
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
		Error::ImageError(Box::new(value))
	}
}

impl<T> From<std::sync::LockResult<T>> for Error
{
	fn from(_value : std::sync::LockResult<T>) -> Self { Error::MutexError }
}

impl<T> From<std::sync::PoisonError<T>> for Error
{
	fn from(_value : std::sync::PoisonError<T>) -> Self { Error::PoisonError }
}

impl From<std::net::AddrParseError> for Error
{
	fn from(value : std::net::AddrParseError) -> Self { Error::AddrParseError(value) }
}

impl From<nokhwa::NokhwaError> for Error {
	fn from(value : nokhwa::NokhwaError) -> Self { Error::NokhwaError(Box::new(value)) }
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		match self
		{
			Error::IOError(e) => { e.fmt(f) }
			Error::Custom(msg) => { msg.fmt(f) }
			Error::LocalIPError => { "Failed to acquire local IP".fmt(f) }
			Error::ImageError(e) => { e.fmt(f) }
			Error::H264Error(e) => { e.fmt(f) }
			Error::Infallible(e) => { e.fmt(f) }
			Error::MutexError => { "Failed to acquire lock!".fmt(f) }
			Error::PoisonError => { "Failed to acquire lock!".fmt(f) }
			Error::AddrParseError(e) => { e.fmt(f) }
			Error::RTPTypeNotImplemented(value) => { format!("RTP type {value} not implemented!").fmt(f) }
			Error::AnyhowError(e) => { e.fmt(f) }
			Error::NoVideoSource => { "Server instance did not have a video source!".fmt(f) }
			Error::NoVideoTarget => { "Server instance did not have a video target!".fmt(f) }
			Error::ParseIntError(e) => { e.fmt(f) }
			Error::SqliteError(e) => { e.fmt(f) }
			Error::SerdeJSON(e) => { e.fmt(f) }
			Error::FromUtf8Error(e) => { e.fmt(f) }
			Error::SystemTimeError(e) => { e.fmt(f) }
			Error::TryFromSliceError(e) => { e.fmt(f) }
			Error::NokhwaError(e) => { e.fmt(f) }
			Error::TryRecvError(e) => { e.fmt(f) }
		}
	}
}

impl std::error::Error for Error {}

