mod helper;
mod video_stream;
#[cfg(test)]
mod tests;


use std::io::Error;

fn main() -> Result<(), Error> {
	println!("Hello, world!");
	Ok(())
}
