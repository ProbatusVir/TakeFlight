use std::process::{Command, Stdio};
use std::io::Error;

/// This will build the JavaScript.
/// Both fail if the JavaScript doesn't build.
fn main() -> Result<(), Error>
{
	const FRONTEND_DIRECTORY : &str = "Frontend/takeofftestapp"; // we are using the system-agnostic path

	#[cfg(debug_assertions)]
	let mut react_child = Command::new("cmd")
		.args(["/C", "npm run dev"])
		.current_dir(FRONTEND_DIRECTORY)
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.spawn()?;

	#[cfg(not(debug_assertions))]
	let mut _react_child = Command::new("cmd")
		.args(["/C", "npm run build"])
		.current_dir(FRONTEND_DIRECTORY)
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.spawn()?;

	Ok(())
}
