use std::io::Error;
fn main() -> Result<(), Error>
{
	// Create test folder, if applicable
	 {
		const TEST_RESULT_FOLDER : &str = "test_results";
		if !std::fs::exists(TEST_RESULT_FOLDER)?
		{
			std::fs::create_dir(TEST_RESULT_FOLDER)?;
		}
	}

	Ok(())
}