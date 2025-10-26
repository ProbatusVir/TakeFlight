use std::io::Error;

/// This will build the JavaScript.
/// Both fail if the JavaScript doesn't build.
fn main() -> Result<(), Error>
{
	// Create test folder, if applicable
	#[cfg(test)]
	{
		const TEST_RESULT_FOLDER : &str = "test_results";
		if !std::fs::exists(TEST_RESULT_FOLDER)?
		{
			std::fs::create_dir(TEST_RESULT_FOLDER)?;
		}
	}
	Ok(())
}
