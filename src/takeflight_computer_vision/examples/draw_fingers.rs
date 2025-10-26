extern crate sdl3 as sdl;
use takeflight_computer_vision::{Error};

fn main() -> Result<(), Error>
{
	let sdl_context = sdl3::init()?;

	Ok(())
}