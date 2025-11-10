/*use crate::Error;
use rstest::rstest;

mod tello_drone
{
	use super::rstest;
	use super::Error;
	use crate::drone_interface::tello::drone::Drone as TelloDrone;
	use crate::drone_interface::Drone;
	use std::f32::consts::PI;
	use std::thread::sleep;
	use std::time::Duration;
	#[rstest]
	fn drone_init_test() -> Result<(), Error>
	{
		TelloDrone::init()?;

		Ok(())
	}

	#[rstest]
	fn simple_planned_flight_test() -> Result<(), Error>
	{
		let mut drone = TelloDrone::init()?;
		sleep(Duration::from_secs(1));
		drone.takeoff()?;

		// Box flight, each section is 25 centimeters
		drone.forward(25)?;
		drone.left(25)?;
		drone.backward(25)?;
		drone.right(25)?;

		drone.clockwise_rot(6.0 * PI)?;	// Three rotations clockwise
		drone.cclockwise_rot(6.0 * PI)?;	// Three rotations counterclockwise

		drone.graceful_land()?;

		Ok(())
	}
}*/