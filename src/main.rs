mod helper;
mod video_stream;
mod drone_interface;
mod error;
#[cfg(test)]
mod tests;
#[cfg(debug_assertions)]
pub(crate) mod debug_utils;

mod computer_vision;
mod video;
use tflitec as tf;

use crate::drone_interface::drone_pro;
use error::Error;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;


fn main() -> Result<(), Error> {
	println!("Hello, world!");

	let mut application_status = Command::new("cmd")
		.args(["/C", "start http://localhost:5173"])
		.spawn()?;

	// We will be implementing the TakeFlight server backend here. Since the process is spawned we can do our anything here



	println!("React application exited with status: {}", application_status.wait()?);

	Ok(())

}
