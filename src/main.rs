mod helper;
mod video_stream;
mod drone_interface;
mod error;
#[cfg(test)]
mod tests;


use std::io::Error;

fn main() -> Result<(), Error> {
	println!("Hello, world!");
	Ok(())
}
