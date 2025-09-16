mod tello;

use crate::error::Error;

/// The unit corresponds to a centimeter, for now; even if the precision of the drone is not matched to the centimeter.
type Unit = u64;
pub trait Drone
{
	/// The init will establish whatever internal state is necessary.
	/// This must include whatever network operations are necessary
	/// for communicating with the drone. This may ***NOT*** include
	/// takeoff.
	fn init() -> Result<Self, Error> where Self: Sized;

	/// The drone will reach an operational height.
	fn takeoff() -> Result<(), Error>;
	/// The drone will immediately cease all motor activity, this
	/// should also power down the drone and disconnect all network
	/// activity to the drone.

	fn emergency_land() -> Result<(), Error>;

	/// The drone will attempt to land stably. No guarantees on
	/// precisely where the drone will land.
	fn graceful_land() -> Result<(), Error>;

	/// Will move the drone x centimeters in the direction
	/// it is facing.
	fn forward(x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters opposite of the
	/// direction it is facing.
	fn backward(x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters left of the
	/// direction it is facing.
	fn left(x : Unit) -> Result<(), Error>;

	/// Will move the drone x centimeters right of the
	/// direction it is facing.
	fn right(x : Unit) -> Result<(), Error>;

	/// The drone's yaw may be adjusted in radians.
	/// A negative radian will result in a counter
	/// clockwise rotation.
	fn clockwise_rot(rads: f32) -> Result<(), Error>;

	/// The drone's yaw may be adjusted in radians.
	/// A negative radian will result in a clockwise
	/// rotation.
	fn cclockwise_rot(rads : f32) -> Result<(), Error>;

}