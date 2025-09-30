mod helper;
mod video_stream;
mod drone_interface;
mod error;
#[cfg(test)]
mod tests;

use tflitec as tf;

use error::Error;
fn main() -> Result<(), Error> {
	println!("Hello, world!");
	Ok(())
}
