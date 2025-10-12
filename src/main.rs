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
    let frontend_dir = r"Frontend\\takeofftestapp"; // Windows path
    let mut react_child = Command::new("cmd")
        .args(["/C", "npm run dev"])
        .current_dir(frontend_dir)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    thread::sleep(Duration::from_secs(3));
    let _ = Command::new("cmd")
        .args(["/C", "start http://localhost:5173"])
        .spawn();
    let status = react_child.wait()?;
    println!("React dev server exited with status: {status}");

    Ok(())

}
